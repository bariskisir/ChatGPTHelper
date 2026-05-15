//! Application settings and selection area models.

use super::{DEFAULT_MODEL, DEFAULT_THINKING_VARIANT, SystemPromptPreset};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectionArea {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
    pub saved_at: chrono::DateTime<chrono::Utc>,
}

impl SelectionArea {
    /// Clamps the selection area into valid screen-relative bounds.
    pub fn normalized(mut self) -> Self {
        self.left = clamp_ratio(self.left);
        self.top = clamp_ratio(self.top);
        self.width = clamp_ratio(self.width);
        self.height = clamp_ratio(self.height);
        if self.left + self.width > 1.0 {
            self.width = (1.0 - self.left).max(0.0);
        }
        if self.top + self.height > 1.0 {
            self.height = (1.0 - self.top).max(0.0);
        }
        self
    }

    /// Checks whether a selection is large enough to reuse or submit.
    pub fn usable(&self) -> bool {
        self.width >= 0.01 && self.height >= 0.01
    }
}

/// Constrains a ratio into the inclusive zero-to-one range.
fn clamp_ratio(value: f64) -> f64 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    #[serde(default = "default_model")]
    pub text_scan_model: String,
    #[serde(default = "default_model")]
    pub image_scan_model: String,
    #[serde(default = "default_thinking_variant")]
    pub text_thinking_variant: String,
    #[serde(default = "default_thinking_variant")]
    pub image_thinking_variant: String,
    #[serde(default)]
    pub text_system_prompt_preset: SystemPromptPreset,
    #[serde(default)]
    pub image_system_prompt_preset: SystemPromptPreset,
    #[serde(default)]
    pub text_custom_system_prompt: String,
    #[serde(default)]
    pub image_custom_system_prompt: String,
    #[serde(default)]
    pub last_text_area: Option<SelectionArea>,
    #[serde(default)]
    pub last_image_area: Option<SelectionArea>,
    #[serde(default)]
    pub always_on_top: bool,
}

impl Default for AppSettings {
    /// Builds the default value for this type.
    fn default() -> Self {
        Self {
            text_scan_model: DEFAULT_MODEL.to_owned(),
            image_scan_model: DEFAULT_MODEL.to_owned(),
            text_thinking_variant: DEFAULT_THINKING_VARIANT.to_owned(),
            image_thinking_variant: DEFAULT_THINKING_VARIANT.to_owned(),
            text_system_prompt_preset: SystemPromptPreset::Solver,
            image_system_prompt_preset: SystemPromptPreset::Solver,
            text_custom_system_prompt: String::new(),
            image_custom_system_prompt: String::new(),
            last_text_area: None,
            last_image_area: None,
            always_on_top: false,
        }
    }
}

/// Returns the fallback ChatGPT model identifier.
fn default_model() -> String {
    DEFAULT_MODEL.to_owned()
}
/// Returns the fallback reasoning effort value.
fn default_thinking_variant() -> String {
    DEFAULT_THINKING_VARIANT.to_owned()
}
