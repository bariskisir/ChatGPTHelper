//! Screen capture and scan command handlers.

use super::CmdResult;
use crate::app::state::AppState;
use crate::app::view::{AppViewState, ManualInput, ScanInput};
use crate::domain::{ScanKind, SelectionArea};
use crate::infra::{screen, screen_selector};
use base64::Engine;
use chrono::Utc;
use serde::Serialize;
use std::{thread, time::Duration};
use tauri::{AppHandle, Manager, State};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CapturedArea {
    area: SelectionArea,
    image_data_url: String,
}

/// Submits manual text or pasted image input for a ChatGPT response.
#[tauri::command]
pub fn submit_manual_input(
    input: ManualInput,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> CmdResult<AppViewState> {
    state
        .submit_manual_input(input, app_handle)
        .map_err(|e| e.to_string())
}

/// Submits OCR text or a cropped image scan for a ChatGPT response.
#[tauri::command]
pub fn submit_scan(
    input: ScanInput,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> CmdResult<AppViewState> {
    state
        .submit_scan(input, app_handle)
        .map_err(|e| e.to_string())
}

/// Returns or starts the most recently saved scan area for the requested kind.
#[tauri::command]
pub fn repeat_scan(kind: ScanKind, state: State<'_, AppState>) -> CmdResult<Option<SelectionArea>> {
    state.repeat_scan(kind).map_err(|e| e.to_string())
}

/// Opens the native selector or reuses a previous area and returns a PNG data URL.
#[tauri::command]
pub fn select_screen_area(
    kind: ScanKind,
    previous_area: Option<SelectionArea>,
    confirm_previous: Option<bool>,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> CmdResult<Option<CapturedArea>> {
    log::info!("Starting native {kind:?} screen selection");
    let window = app_handle
        .get_webview_window("main")
        .ok_or_else(|| "Main window was not found.".to_owned())?;
    let always_on_top = state
        .view_state()
        .map_err(|e| e.to_string())?
        .settings
        .always_on_top;
    let selection_style = match kind {
        ScanKind::Text => screen_selector::SelectionStyle::text(),
        ScanKind::Image => screen_selector::SelectionStyle::image(),
    };

    window.hide().map_err(|e| e.to_string())?;
    thread::sleep(Duration::from_millis(400));

    let initial_selection = previous_area.map(|area| screen_selector::RelativeSelection {
        left: area.left,
        top: area.top,
        width: area.width,
        height: area.height,
    });
    let selection_result = if let Some(initial_selection) = initial_selection {
        if confirm_previous.unwrap_or(true) {
            screen_selector::select_region(selection_style, Some(initial_selection))
                .map_err(|e| e.to_string())
        } else {
            let image = screen::capture_primary_screen_image().map_err(|e| e.to_string())?;
            let selection = relative_to_pixels(initial_selection, image.width, image.height);
            Ok(Some(screen_selector::SelectedScreenRegion {
                image,
                selection,
            }))
        }
    } else {
        screen_selector::select_region(selection_style, None).map_err(|e| e.to_string())
    };

    window
        .set_always_on_top(always_on_top)
        .map_err(|e| e.to_string())?;
    window.show().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())?;

    let Some(selected) = selection_result? else {
        return Ok(None);
    };
    let area = pixels_to_area(
        selected.selection,
        selected.image.width,
        selected.image.height,
    );
    let image_data_url = crop_to_png_data_url(&selected.image, selected.selection)?;
    Ok(Some(CapturedArea {
        area,
        image_data_url,
    }))
}

/// Converts normalized selection ratios into pixel bounds.
fn relative_to_pixels(
    area: screen_selector::RelativeSelection,
    image_width: u32,
    image_height: u32,
) -> screen_selector::PixelSelection {
    let left_ratio = clamp_ratio(area.left);
    let top_ratio = clamp_ratio(area.top);
    let mut width_ratio = clamp_ratio(area.width);
    let mut height_ratio = clamp_ratio(area.height);
    if left_ratio + width_ratio > 1.0 {
        width_ratio = (1.0 - left_ratio).max(0.0);
    }
    if top_ratio + height_ratio > 1.0 {
        height_ratio = (1.0 - top_ratio).max(0.0);
    }
    let left =
        ((left_ratio * image_width as f64).round() as u32).min(image_width.saturating_sub(1));
    let top =
        ((top_ratio * image_height as f64).round() as u32).min(image_height.saturating_sub(1));
    let width = ((width_ratio * image_width as f64).round() as u32)
        .max(1)
        .min(image_width - left);
    let height = ((height_ratio * image_height as f64).round() as u32)
        .max(1)
        .min(image_height - top);
    screen_selector::PixelSelection {
        left,
        top,
        width,
        height,
    }
}

/// Constrains a stored area ratio into selector bounds.
fn clamp_ratio(value: f64) -> f64 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// Converts pixel-based selection bounds into normalized ratios.
fn pixels_to_area(
    selection: screen_selector::PixelSelection,
    image_width: u32,
    image_height: u32,
) -> SelectionArea {
    SelectionArea {
        left: selection.left as f64 / image_width as f64,
        top: selection.top as f64 / image_height as f64,
        width: selection.width as f64 / image_width as f64,
        height: selection.height as f64 / image_height as f64,
        saved_at: Utc::now(),
    }
    .normalized()
}

/// Crops a screen image selection and encodes it as a PNG data URL.
fn crop_to_png_data_url(
    image: &screen::ScreenImage,
    selection: screen_selector::PixelSelection,
) -> CmdResult<String> {
    let sw = image.width as usize;
    let sh = image.height as usize;
    let left = (selection.left as usize).min(sw - 1);
    let top = (selection.top as usize).min(sh - 1);
    let cw = (selection.width as usize).max(1).min(sw - left);
    let ch = (selection.height as usize).max(1).min(sh - top);
    let rgba = crop_rgba(&image.rgba, sw, sh, left, top, cw, ch)?;
    encode_png_data_url(cw as u32, ch as u32, &rgba)
}

/// Copies selected RGBA rows out of the full screen capture buffer.
fn crop_rgba(
    rgba: &[u8],
    sw: usize,
    sh: usize,
    left: usize,
    top: usize,
    cw: usize,
    ch: usize,
) -> CmdResult<Vec<u8>> {
    let stride = sw * 4;
    if rgba.len() < stride * sh {
        return Err("Capture image pixel data was incomplete.".to_owned());
    }
    let mut cropped = Vec::with_capacity(cw * ch * 4);
    for row in top..top + ch {
        let start = row * stride + left * 4;
        let end = start + cw * 4;
        cropped.extend_from_slice(&rgba[start..end]);
    }
    Ok(cropped)
}

/// Encodes RGBA pixels into a base64 PNG data URL.
fn encode_png_data_url(width: u32, height: u32, rgba: &[u8]) -> CmdResult<String> {
    let mut png_bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut png_bytes, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
        writer.write_image_data(rgba).map_err(|e| e.to_string())?;
    }
    Ok(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(png_bytes)
    ))
}
