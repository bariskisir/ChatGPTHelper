//! History navigation and clearing command handlers.

use super::CmdResult;
use crate::app::state::AppState;
use crate::app::view::AppViewState;
use tauri::State;

/// Moves the selected history entry by the supplied offset.
#[tauri::command]
pub fn select_history_by_offset(
    offset: isize,
    state: State<'_, AppState>,
) -> CmdResult<AppViewState> {
    state
        .select_history_by_offset(offset)
        .map_err(|e| e.to_string())
}

/// Clears all stored answer history.
#[tauri::command]
pub fn delete_history(state: State<'_, AppState>) -> CmdResult<AppViewState> {
    state.delete_history().map_err(|e| e.to_string())
}
