//! Window, clipboard, and external-link command handlers.

use super::CmdResult;
use crate::app::state::AppState;
use crate::app::view::AppViewState;
use crate::infra::clipboard;
use tauri::{AppHandle, Manager, State};

/// Copies text through the native clipboard command path.
#[tauri::command]
pub fn copy_text_to_clipboard(text: String, app_handle: AppHandle) -> CmdResult<()> {
    if text.is_empty() {
        return Ok(());
    }
    #[cfg(target_os = "windows")]
    {
        let window = app_handle
            .get_webview_window("main")
            .ok_or_else(|| "Main window was not found.".to_owned())?;
        let hwnd = window.hwnd().map_err(|e| e.to_string())?;
        clipboard::write_text(&text, hwnd).map_err(|e| e.to_string())
    }
    #[cfg(not(target_os = "windows"))]
    {
        clipboard::write_text(&text).map_err(|e| e.to_string())
    }
}

/// Persists the always-on-top window setting.
#[tauri::command]
pub fn set_always_on_top(
    enabled: bool,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> CmdResult<AppViewState> {
    let view = state
        .set_always_on_top(enabled)
        .map_err(|e| e.to_string())?;
    let window = app_handle
        .get_webview_window("main")
        .ok_or_else(|| "Main window was not found.".to_owned())?;
    window
        .set_always_on_top(enabled)
        .map_err(|e| e.to_string())?;
    Ok(view)
}

/// Opens the developer website in the default browser.
#[tauri::command]
pub fn open_developer_site(state: State<'_, AppState>) -> CmdResult<()> {
    state.open_developer_site().map_err(|e| e.to_string())
}

/// Opens the source repository in the default browser.
#[tauri::command]
pub fn open_source_site(state: State<'_, AppState>) -> CmdResult<()> {
    state.open_source_site().map_err(|e| e.to_string())
}
