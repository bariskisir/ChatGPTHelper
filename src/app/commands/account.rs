//! Account and settings command handlers exposed to the frontend.

use super::CmdResult;
use crate::app::state::AppState;
use crate::app::view::{AppViewState, SettingsInput};
use tauri::{AppHandle, State};

/// Returns the current application state to the frontend.
#[tauri::command]
pub fn get_app_state(state: State<'_, AppState>) -> CmdResult<AppViewState> {
    state.view_state().map_err(|e| e.to_string())
}

/// Persists frontend settings and returns the refreshed view state.
#[tauri::command]
pub fn save_settings(
    settings: SettingsInput,
    state: State<'_, AppState>,
) -> CmdResult<AppViewState> {
    state
        .save_frontend_settings(settings)
        .map_err(|e| e.to_string())
}

/// Starts the ChatGPT OAuth sign-in flow.
#[tauri::command]
pub fn start_chatgpt_login(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> CmdResult<AppViewState> {
    state
        .start_chatgpt_login(app_handle)
        .map_err(|e| e.to_string())
}

/// Clears stored ChatGPT authentication state.
#[tauri::command]
pub fn sign_out_chatgpt(state: State<'_, AppState>) -> CmdResult<AppViewState> {
    state.sign_out_chatgpt().map_err(|e| e.to_string())
}

/// Fetches the latest ChatGPT model catalog for the signed-in account.
#[tauri::command]
pub fn refresh_chatgpt_models(state: State<'_, AppState>) -> CmdResult<AppViewState> {
    state.refresh_chatgpt_models().map_err(|e| e.to_string())
}

/// Refreshes the displayed ChatGPT usage-limit label.
#[tauri::command]
pub fn refresh_chatgpt_limits(state: State<'_, AppState>) -> CmdResult<AppViewState> {
    state.refresh_chatgpt_limits().map_err(|e| e.to_string())
}
