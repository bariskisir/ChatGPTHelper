//! View models and data transfer types for the frontend.

use crate::domain::{
    AppSettings, AvailableModel, CatalogStorage, HistoryEntry, SelectionArea, ThinkingVariantOption,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppViewState {
    pub settings: AppSettings,
    pub status: String,
    pub chatgpt: ChatGptViewState,
    pub models: Vec<AvailableModel>,
    pub text_thinking_variants: Vec<ThinkingVariantOption>,
    pub image_thinking_variants: Vec<ThinkingVariantOption>,
    pub history: Vec<HistoryEntry>,
    pub history_index: usize,
    pub selected_history: Option<HistoryEntry>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatGptViewState {
    pub logged_in: bool,
    pub account_email: String,
    pub limit_label: String,
    pub error: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsInput {
    pub text_scan_model: String,
    pub image_scan_model: String,
    pub text_thinking_variant: String,
    pub image_thinking_variant: String,
    pub text_system_prompt_preset: String,
    pub image_system_prompt_preset: String,
    pub text_custom_system_prompt: String,
    pub image_custom_system_prompt: String,
    pub always_on_top: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualInput {
    pub text: String,
    #[serde(default)]
    pub image_data_url: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanInput {
    pub kind: crate::domain::ScanKind,
    pub text: String,
    #[serde(default)]
    pub image_data_url: String,
    pub area: SelectionArea,
}

/// Finds the reasoning-effort options for the selected model.
pub fn resolve_thinking_variants(
    model: &str,
    catalog: &CatalogStorage,
) -> Vec<ThinkingVariantOption> {
    catalog
        .available_models
        .iter()
        .find(|item| item.model == model)
        .or_else(|| catalog.available_models.iter().find(|item| item.is_default))
        .or_else(|| catalog.available_models.first())
        .map(|item| item.thinking_variants.clone())
        .filter(|items| !items.is_empty())
        .unwrap_or_else(crate::domain::fallback_thinking_variants)
}
