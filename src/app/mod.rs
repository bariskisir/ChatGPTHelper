//! Application layer: state management, Tauri commands, and UI events.

mod commands;
mod events;
pub mod state;
mod view;

pub use commands::{
    copy_text_to_clipboard, delete_history, get_app_state, open_developer_site, open_source_site,
    refresh_chatgpt_limits, refresh_chatgpt_models, repeat_scan, save_settings,
    select_history_by_offset, select_screen_area, set_always_on_top, sign_out_chatgpt,
    start_chatgpt_login, submit_manual_input, submit_scan,
};
pub use events::UiEvent;
pub use state::AppState;
