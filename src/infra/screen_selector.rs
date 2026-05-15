//! Native Windows screen region selector.

use super::screen::{ScreenImage, capture_primary_screen_image};
use anyhow::{Result, anyhow};

#[derive(Clone, Copy, Debug)]
pub struct PixelSelection {
    pub left: u32,
    pub top: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct RelativeSelection {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug)]
pub struct SelectedScreenRegion {
    pub image: ScreenImage,
    pub selection: PixelSelection,
}

#[derive(Clone, Copy, Debug)]
pub struct SelectionStyle {
    pub border: RgbColor,
    pub fill: RgbaColor,
}

#[derive(Clone, Copy, Debug)]
pub struct RgbColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct RgbaColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl SelectionStyle {
    /// Returns the selection style used for OCR text scans.
    pub const fn text() -> Self {
        Self {
            border: RgbColor {
                red: 15,
                green: 159,
                blue: 143,
            },
            fill: RgbaColor {
                red: 15,
                green: 159,
                blue: 143,
                alpha: 36,
            },
        }
    }

    /// Returns the selection style used for image scans.
    pub const fn image() -> Self {
        Self {
            border: RgbColor {
                red: 244,
                green: 197,
                blue: 66,
            },
            fill: RgbaColor {
                red: 244,
                green: 197,
                blue: 66,
                alpha: 56,
            },
        }
    }
}

/// Lets the user select a native screen region.
#[cfg(target_os = "windows")]
pub fn select_region(
    style: SelectionStyle,
    initial_selection: Option<RelativeSelection>,
) -> Result<Option<SelectedScreenRegion>> {
    windows_selector::select_region(style, initial_selection)
}

/// Lets the user select a native screen region.
#[cfg(not(target_os = "windows"))]
pub fn select_region(
    _style: SelectionStyle,
    _initial_selection: Option<RelativeSelection>,
) -> Result<Option<SelectedScreenRegion>> {
    Err(anyhow!(
        "Native screen selection is currently implemented for Windows only."
    ))
}

#[cfg(target_os = "windows")]
mod windows_selector {
    use super::*;
    use std::ffi::c_void;
    use std::mem::size_of;
    use windows::{
        Win32::{
            Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
            Graphics::Gdi::{
                AC_SRC_ALPHA, AC_SRC_OVER, AlphaBlend, BI_RGB, BITMAPINFO, BITMAPINFOHEADER,
                BLENDFUNCTION, BeginPaint, CreateCompatibleDC, CreateDIBSection, CreatePen,
                DIB_RGB_COLORS, DeleteDC, DeleteObject, EndPaint, GetStockObject, HDC, HGDIOBJ,
                HOLLOW_BRUSH, InvalidateRect, PS_SOLID, Rectangle, SRCCOPY, SelectObject,
                StretchDIBits, UpdateWindow,
            },
            System::LibraryLoader::GetModuleHandleW,
            UI::Input::KeyboardAndMouse::{ReleaseCapture, SetCapture},
            UI::WindowsAndMessaging::{
                CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, CreateWindowExW, DefWindowProcW,
                DestroyWindow, DispatchMessageW, GWLP_USERDATA, GetMessageW, GetWindowLongPtrW,
                IDC_CROSS, LoadCursorW, MSG, RegisterClassW, SW_SHOW, SetForegroundWindow,
                SetWindowLongPtrW, ShowWindow, TranslateMessage, WINDOW_EX_STYLE, WINDOW_STYLE,
                WM_CLOSE, WM_DESTROY, WM_KEYDOWN, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE,
                WM_NCCREATE, WM_PAINT, WM_RBUTTONDOWN, WNDCLASSW, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
                WS_POPUP, WS_VISIBLE,
            },
        },
        core::{PCWSTR, w},
    };

    const MIN_SELECTION_SIZE: i32 = 5;
    const VK_ESCAPE_KEY: usize = 27;
    const VK_RETURN_KEY: usize = 13;
    const VK_SPACE_KEY: usize = 32;

    #[derive(Clone, Copy, Debug)]
    struct Point {
        x: i32,
        y: i32,
    }

    struct SelectorState {
        image: Option<ScreenImage>,
        bgra: Vec<u8>,
        width: i32,
        height: i32,
        drag_start: Option<Point>,
        drag_current: Option<Point>,
        selection: Option<PixelSelection>,
        style: SelectionStyle,
        done: bool,
        cancelled: bool,
    }

    /// Lets the user select a native screen region.
    pub fn select_region(
        style: SelectionStyle,
        initial_selection: Option<RelativeSelection>,
    ) -> Result<Option<SelectedScreenRegion>> {
        let image = capture_primary_screen_image()?;
        let width = image.width as i32;
        let height = image.height as i32;
        if width <= 0 || height <= 0 {
            return Err(anyhow!("Screen capture returned an invalid size."));
        }

        let bgra = rgba_to_bgra(&image.rgba);
        let class_name = w!("ChatGptHelperNativeSelector");
        let instance = unsafe { GetModuleHandleW(None) }?;
        register_window_class(instance.into(), class_name)?;
        let initial_selection = initial_selection
            .and_then(|area| relative_to_selection(area, image.width, image.height));

        let mut state = Box::new(SelectorState {
            image: Some(image),
            bgra,
            width,
            height,
            drag_start: None,
            drag_current: None,
            selection: initial_selection,
            style,
            done: false,
            cancelled: false,
        });
        let state_ptr = state.as_mut() as *mut SelectorState;

        let hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE(WS_EX_TOPMOST.0 | WS_EX_TOOLWINDOW.0),
                class_name,
                w!("Select capture area"),
                WINDOW_STYLE(WS_POPUP.0 | WS_VISIBLE.0),
                state
                    .image
                    .as_ref()
                    .map(|img| img.bounds.left)
                    .unwrap_or(CW_USEDEFAULT),
                state
                    .image
                    .as_ref()
                    .map(|img| img.bounds.top)
                    .unwrap_or(CW_USEDEFAULT),
                width,
                height,
                None,
                None,
                Some(instance.into()),
                Some(state_ptr.cast()),
            )
        }?;

        unsafe {
            let _ = ShowWindow(hwnd, SW_SHOW);
            let _ = SetForegroundWindow(hwnd);
            let _ = InvalidateRect(Some(hwnd), None, false);
        }

        std::mem::forget(state);

        let mut message = MSG::default();
        loop {
            let status = unsafe { GetMessageW(&mut message, None, 0, 0) };
            if status.0 == -1 {
                unsafe {
                    let _ = Box::from_raw(state_ptr);
                }
                return Err(anyhow!("Could not read selector window messages."));
            }
            if status.0 == 0 {
                break;
            }
            unsafe {
                let _ = TranslateMessage(&message);
                DispatchMessageW(&message);
                if (*state_ptr).done {
                    break;
                }
            }
        }

        let mut state = unsafe { Box::from_raw(state_ptr) };

        if state.cancelled {
            return Ok(None);
        }
        let selection = state
            .selection
            .ok_or_else(|| anyhow!("No screen area was selected."))?;
        let image = state
            .image
            .take()
            .ok_or_else(|| anyhow!("Screen capture was not available."))?;
        Ok(Some(SelectedScreenRegion { image, selection }))
    }

    /// Registers the Win32 overlay window class used by the selector.
    fn register_window_class(instance: HINSTANCE, class_name: PCWSTR) -> Result<()> {
        let cursor = unsafe { LoadCursorW(None, IDC_CROSS)? };
        let window_class = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(selector_wnd_proc),
            hInstance: instance,
            hCursor: cursor,
            lpszClassName: class_name,
            ..Default::default()
        };
        let atom = unsafe { RegisterClassW(&window_class) };
        if atom == 0 {
            // RegisterClassW returns 0 if the class already exists; that is fine for this selector.
        }
        Ok(())
    }

    /// Handles Win32 messages for the native selection overlay.
    unsafe extern "system" fn selector_wnd_proc(
        hwnd: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if message == WM_NCCREATE {
            let createstruct =
                lparam.0 as *const windows::Win32::UI::WindowsAndMessaging::CREATESTRUCTW;
            let state_ptr = unsafe { (*createstruct).lpCreateParams as *mut SelectorState };
            unsafe {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, state_ptr as isize);
            }
            return unsafe { DefWindowProcW(hwnd, message, wparam, lparam) };
        }

        let state_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut SelectorState };
        if state_ptr.is_null() {
            return unsafe { DefWindowProcW(hwnd, message, wparam, lparam) };
        }
        let state = unsafe { &mut *state_ptr };

        match message {
            WM_PAINT => {
                paint(hwnd, state);
                LRESULT(0)
            }
            WM_LBUTTONDOWN => {
                let point = message_point(lparam, state.width, state.height);
                state.drag_start = Some(point);
                state.drag_current = Some(point);
                state.selection = None;
                unsafe {
                    let _ = SetCapture(hwnd);
                    let _ = InvalidateRect(Some(hwnd), None, false);
                    let _ = UpdateWindow(hwnd);
                }
                LRESULT(0)
            }
            WM_MOUSEMOVE => {
                if state.drag_start.is_some() {
                    state.drag_current = Some(message_point(lparam, state.width, state.height));
                    unsafe {
                        let _ = InvalidateRect(Some(hwnd), None, false);
                        let _ = UpdateWindow(hwnd);
                    }
                }
                LRESULT(0)
            }
            WM_LBUTTONUP => {
                if let Some(start) = state.drag_start.take() {
                    let end = message_point(lparam, state.width, state.height);
                    state.drag_current = Some(end);
                    unsafe {
                        let _ = ReleaseCapture();
                    }
                    if let Some(selection) =
                        selection_from_points(start, end, state.width, state.height)
                    {
                        state.selection = Some(selection);
                        state.done = true;
                        unsafe {
                            DestroyWindow(hwnd).ok();
                        }
                    } else {
                        state.selection = None;
                        state.drag_current = None;
                        unsafe {
                            let _ = InvalidateRect(Some(hwnd), None, false);
                            let _ = UpdateWindow(hwnd);
                        }
                    }
                }
                LRESULT(0)
            }
            WM_RBUTTONDOWN | WM_CLOSE => {
                state.cancelled = true;
                state.done = true;
                unsafe {
                    DestroyWindow(hwnd).ok();
                }
                LRESULT(0)
            }
            WM_KEYDOWN => {
                match wparam.0 {
                    VK_ESCAPE_KEY => {
                        state.cancelled = true;
                        state.done = true;
                        unsafe {
                            DestroyWindow(hwnd).ok();
                        }
                    }
                    VK_RETURN_KEY | VK_SPACE_KEY => {
                        if state.selection.is_some() {
                            state.done = true;
                            unsafe {
                                DestroyWindow(hwnd).ok();
                            }
                        }
                    }
                    _ => {}
                }
                LRESULT(0)
            }
            WM_DESTROY => {
                if !state.done {
                    state.cancelled = true;
                    state.done = true;
                }
                LRESULT(0)
            }
            _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
        }
    }

    /// Paints the captured screen and active selection overlay.
    fn paint(hwnd: HWND, state: &SelectorState) {
        unsafe {
            let mut paint = Default::default();
            let hdc = BeginPaint(hwnd, &mut paint);
            draw_capture(hdc, state);
            if let Some(selection) = active_selection(state) {
                draw_selection(hdc, selection, state.style);
            }
            let _ = EndPaint(hwnd, &paint);
        }
    }

    /// Draws the captured screen image into the selector window.
    unsafe fn draw_capture(hdc: HDC, state: &SelectorState) {
        let mut bitmap_info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: state.width,
                biHeight: -state.height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        unsafe {
            StretchDIBits(
                hdc,
                0,
                0,
                state.width,
                state.height,
                0,
                0,
                state.width,
                state.height,
                Some(state.bgra.as_ptr().cast()),
                &mut bitmap_info,
                DIB_RGB_COLORS,
                SRCCOPY,
            );
        }
    }

    /// Draws the highlighted selection outline and fill.
    unsafe fn draw_selection(hdc: HDC, selection: PixelSelection, style: SelectionStyle) {
        let left = selection.left as i32;
        let top = selection.top as i32;
        let right = (selection.left + selection.width) as i32;
        let bottom = (selection.top + selection.height) as i32;
        let width = right - left;
        let height = bottom - top;
        unsafe {
            draw_translucent_fill(hdc, left, top, width, height, style.fill);

            let hollow = GetStockObject(HOLLOW_BRUSH);
            let old_brush = SelectObject(hdc, hollow);

            let color_pen = CreatePen(PS_SOLID, 2, colorref(style.border));
            let old_pen = SelectObject(hdc, HGDIOBJ(color_pen.0));
            let _ = Rectangle(hdc, left, top, right, bottom);
            let _ = SelectObject(hdc, old_pen);
            let _ = DeleteObject(HGDIOBJ(color_pen.0));

            let _ = SelectObject(hdc, old_brush);
        }
    }

    /// Draws the translucent fill used inside the selected area.
    unsafe fn draw_translucent_fill(
        hdc: HDC,
        left: i32,
        top: i32,
        width: i32,
        height: i32,
        color: RgbaColor,
    ) {
        if width <= 0 || height <= 0 || color.alpha == 0 {
            return;
        }

        let bitmap_info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: 1,
                biHeight: 1,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits: *mut c_void = std::ptr::null_mut();
        let bitmap = match unsafe {
            CreateDIBSection(Some(hdc), &bitmap_info, DIB_RGB_COLORS, &mut bits, None, 0)
        } {
            Ok(bitmap) => bitmap,
            Err(_) => return,
        };
        if bits.is_null() {
            unsafe {
                let _ = DeleteObject(HGDIOBJ(bitmap.0));
            }
            return;
        }

        let alpha = color.alpha as u16;
        let pixel = bits.cast::<u8>();
        unsafe {
            *pixel.add(0) = premultiply(color.blue, alpha);
            *pixel.add(1) = premultiply(color.green, alpha);
            *pixel.add(2) = premultiply(color.red, alpha);
            *pixel.add(3) = color.alpha;
        }

        let memory_dc = unsafe { CreateCompatibleDC(Some(hdc)) };
        if memory_dc.is_invalid() {
            unsafe {
                let _ = DeleteObject(HGDIOBJ(bitmap.0));
            }
            return;
        }
        unsafe {
            let old_bitmap = SelectObject(memory_dc, HGDIOBJ(bitmap.0));
            let blend = BLENDFUNCTION {
                BlendOp: AC_SRC_OVER as u8,
                BlendFlags: 0,
                SourceConstantAlpha: 255,
                AlphaFormat: AC_SRC_ALPHA as u8,
            };
            let _ = AlphaBlend(hdc, left, top, width, height, memory_dc, 0, 0, 1, 1, blend);
            let _ = SelectObject(memory_dc, old_bitmap);
            let _ = DeleteObject(HGDIOBJ(bitmap.0));
            let _ = DeleteDC(memory_dc);
        }
    }

    /// Converts a stored relative area into a selector pixel selection.
    fn relative_to_selection(
        area: RelativeSelection,
        image_width: u32,
        image_height: u32,
    ) -> Option<PixelSelection> {
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
        if width < MIN_SELECTION_SIZE as u32 || height < MIN_SELECTION_SIZE as u32 {
            return None;
        }
        Some(PixelSelection {
            left,
            top,
            width,
            height,
        })
    }

    /// Constrains a stored selector ratio into screen bounds.
    fn clamp_ratio(value: f64) -> f64 {
        if value.is_finite() {
            value.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Premultiplies a color channel by alpha for GDI blending.
    fn premultiply(channel: u8, alpha: u16) -> u8 {
        ((channel as u16 * alpha + 127) / 255) as u8
    }

    /// Converts an RGB color into a Win32 COLORREF value.
    fn colorref(color: RgbColor) -> COLORREF {
        COLORREF(color.red as u32 | ((color.green as u32) << 8) | ((color.blue as u32) << 16))
    }

    /// Returns the drag preview or completed selection.
    fn active_selection(state: &SelectorState) -> Option<PixelSelection> {
        if let (Some(start), Some(end)) = (state.drag_start, state.drag_current) {
            visual_selection_from_points(start, end, state.width, state.height)
        } else {
            state.selection
        }
    }

    /// Converts a Win32 mouse message into a clamped selector point.
    fn message_point(lparam: LPARAM, width: i32, height: i32) -> Point {
        let x = (lparam.0 as u32 & 0xffff) as i16 as i32;
        let y = ((lparam.0 as u32 >> 16) & 0xffff) as i16 as i32;
        Point {
            x: x.clamp(0, width),
            y: y.clamp(0, height),
        }
    }

    /// Builds a completed pixel selection from drag endpoints.
    fn selection_from_points(
        start: Point,
        end: Point,
        width: i32,
        height: i32,
    ) -> Option<PixelSelection> {
        let left = start.x.min(end.x).clamp(0, width);
        let top = start.y.min(end.y).clamp(0, height);
        let right = start.x.max(end.x).clamp(0, width);
        let bottom = start.y.max(end.y).clamp(0, height);
        let selection_width = right - left;
        let selection_height = bottom - top;
        if selection_width < MIN_SELECTION_SIZE || selection_height < MIN_SELECTION_SIZE {
            return None;
        }
        Some(PixelSelection {
            left: left as u32,
            top: top as u32,
            width: selection_width as u32,
            height: selection_height as u32,
        })
    }

    /// Builds a visible drag-preview selection from drag endpoints.
    fn visual_selection_from_points(
        start: Point,
        end: Point,
        width: i32,
        height: i32,
    ) -> Option<PixelSelection> {
        let left = start.x.min(end.x).clamp(0, width);
        let top = start.y.min(end.y).clamp(0, height);
        let mut right = start.x.max(end.x).clamp(0, width);
        let mut bottom = start.y.max(end.y).clamp(0, height);
        if right == left {
            right = (right + 1).min(width);
        }
        if bottom == top {
            bottom = (bottom + 1).min(height);
        }
        Some(PixelSelection {
            left: left as u32,
            top: top as u32,
            width: (right - left).max(1) as u32,
            height: (bottom - top).max(1) as u32,
        })
    }

    /// Converts captured RGBA pixels into BGRA order for Win32 drawing.
    fn rgba_to_bgra(rgba: &[u8]) -> Vec<u8> {
        let mut bgra = Vec::with_capacity(rgba.len());
        for pixel in rgba.chunks_exact(4) {
            bgra.extend_from_slice(&[pixel[2], pixel[1], pixel[0], 255]);
        }
        bgra
    }
}
