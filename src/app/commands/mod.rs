//! Tauri command handlers.

mod account;
mod history;
mod screen;
mod window;

pub use account::{
    get_app_state, refresh_chatgpt_limits, refresh_chatgpt_models, save_settings, sign_out_chatgpt,
    start_chatgpt_login,
};
pub use history::{delete_history, select_history_by_offset};
pub use screen::{repeat_scan, select_screen_area, submit_manual_input, submit_scan};
pub use window::{
    copy_text_to_clipboard, open_developer_site, open_source_site, set_always_on_top,
};

type CmdResult<T> = std::result::Result<T, String>;
