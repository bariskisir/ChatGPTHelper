//! Domain models and constants for ChatGPT Helper.

mod catalog;
mod settings;

pub use catalog::*;
pub use settings::*;

pub const DEFAULT_MODEL: &str = "gpt-5.4-mini";
pub const DEFAULT_THINKING_VARIANT: &str = "medium";
pub const DEFAULT_CODEX_CLIENT_VERSION: &str = "0.128.0";
pub const HISTORY_LIMIT: usize = 50;

pub const TEXT_SOLVER_PROMPT: &str = "You are a careful problem solver. Read the selected content, solve accurately, and give the final answer clearly.";
pub const IMAGE_SOLVER_PROMPT: &str = "You are a careful image problem solver. Analyze the selected image area, solve math accurately, interpret charts, diagrams, UI, or other image content when present, and give the key answer concisely and clearly.";

#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ScanKind {
    Text,
    Image,
}

impl ScanKind {
    /// Returns the default solver prompt for this scan kind.
    pub fn solver_prompt(self) -> &'static str {
        match self {
            Self::Text => TEXT_SOLVER_PROMPT,
            Self::Image => IMAGE_SOLVER_PROMPT,
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HistoryEntryType {
    Ask,
    Text,
    Image,
}

impl From<ScanKind> for HistoryEntryType {
    /// Converts a scan kind into the matching history entry type.
    fn from(value: ScanKind) -> Self {
        match value {
            ScanKind::Text => Self::Text,
            ScanKind::Image => Self::Image,
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SystemPromptPreset {
    Solver,
    None,
    Other,
}

impl Default for SystemPromptPreset {
    /// Builds the default value for this type.
    fn default() -> Self {
        Self::Solver
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub input: String,
    #[serde(default)]
    pub input_image_data_url: String,
    pub output: String,
    pub entry_type: HistoryEntryType,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
