//! Frontend events emitted by the Rust backend.

use super::view::AppViewState;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum UiEvent {
    State { state: Box<AppViewState> },
    Answer { answer: String, streaming: bool },
    Error { message: String },
    Shortcut { action: String },
}
