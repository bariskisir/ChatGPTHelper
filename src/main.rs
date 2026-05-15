//! Starts the Tauri desktop ChatGPT Helper application.

#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod domain;
mod infra;

use anyhow::Result;
use app::{
    AppState, UiEvent, copy_text_to_clipboard, delete_history, get_app_state, open_developer_site,
    open_source_site, refresh_chatgpt_limits, refresh_chatgpt_models, repeat_scan, save_settings,
    select_history_by_offset, select_screen_area, set_always_on_top, sign_out_chatgpt,
    start_chatgpt_login, submit_manual_input, submit_scan,
};
use infra::paths::app_paths;
use tauri::{Emitter, Manager, PhysicalPosition, Position};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};

/// Starts the build or application entry point.
fn main() -> Result<()> {
    let paths = app_paths()?;
    infra::logging::install_logger(paths.log_file.clone())?;
    log::info!(
        "ChatGPT Helper Tauri application starting; data_dir={}",
        paths.data_dir.display()
    );
    let state = AppState::new(paths)?;
    let managed_state = state.clone();

    tauri::Builder::default()
        .plugin(global_shortcut_plugin()?)
        .manage(managed_state)
        .setup(move |app| {
            let app_version = app.package_info().version.to_string();
            if let Some(window) = app.get_webview_window("main") {
                window.set_title(&format!("ChatGPT Helper - v{app_version}"))?;
                window.set_position(Position::Physical(PhysicalPosition::new(0, 0)))?;
                if let Ok(view) = state.view_state()
                    && let Err(error) = window.set_always_on_top(view.settings.always_on_top)
                {
                    log::warn!("Could not apply always-on-top setting: {error}");
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_app_state,
            save_settings,
            start_chatgpt_login,
            sign_out_chatgpt,
            refresh_chatgpt_models,
            refresh_chatgpt_limits,
            submit_manual_input,
            select_screen_area,
            submit_scan,
            repeat_scan,
            select_history_by_offset,
            delete_history,
            copy_text_to_clipboard,
            set_always_on_top,
            open_developer_site,
            open_source_site
        ])
        .run(tauri::generate_context!())
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;

    log::info!("ChatGPT Helper Tauri application stopped");
    Ok(())
}

/// Registers global keyboard shortcuts for scan and repeat actions.
fn global_shortcut_plugin() -> Result<tauri::plugin::TauriPlugin<tauri::Wry>> {
    Ok(tauri_plugin_global_shortcut::Builder::new()
        .with_shortcuts([
            "Ctrl+Shift+T",
            "Ctrl+Shift+I",
            "Ctrl+Shift+1",
            "Ctrl+Shift+2",
        ])?
        .with_handler(|app, shortcut, event| {
            if event.state() != ShortcutState::Pressed {
                return;
            }
            let action = shortcut_action(shortcut);
            if action.is_empty() {
                return;
            }
            let _ = app.emit(
                "app-event",
                UiEvent::Shortcut {
                    action: action.to_owned(),
                },
            );
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        })
        .build())
}

/// Maps a pressed shortcut to the frontend action name.
fn shortcut_action(shortcut: &Shortcut) -> &'static str {
    let required_mods = Modifiers::CONTROL | Modifiers::SHIFT;
    if !shortcut.matches(required_mods, shortcut.key) {
        return "";
    }
    match shortcut.key {
        Code::KeyT => "scan-text",
        Code::KeyI => "scan-image",
        Code::Digit1 => "repeat-text",
        Code::Digit2 => "repeat-image",
        _ => "",
    }
}
