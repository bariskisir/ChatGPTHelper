//! Shared application state and business logic.

use super::events::UiEvent;
use super::view::{
    AppViewState, ChatGptViewState, ManualInput, ScanInput, SettingsInput,
    resolve_thinking_variants,
};
use crate::domain::{
    AppSettings, AuthStorage, CatalogStorage, HISTORY_LIMIT, HistoryEntry, HistoryEntryType,
    ScanKind, SelectionArea, SystemPromptPreset,
};
use crate::infra::{chatgpt, paths::AppPaths, shell, storage::Storage};
use anyhow::{Result, anyhow};
use chrono::Utc;
use std::sync::{Arc, Mutex, MutexGuard};
use tauri::{AppHandle, Emitter};
use tokio::runtime::Runtime;

const ASK_RESPONSE_STYLE: &str = "low";
const SCAN_RESPONSE_STYLE: &str = "medium";

#[derive(Clone)]
pub struct AppState {
    inner: Arc<Mutex<StateInner>>,
    runtime: Arc<Runtime>,
}

struct StateInner {
    storage: Storage,
    settings: AppSettings,
    auth: AuthStorage,
    catalog: CatalogStorage,
    status: String,
    history: Vec<HistoryEntry>,
    history_index: usize,
}

#[derive(Clone)]
struct PendingResponse {
    prompt: String,
    history_input: String,
    history_image_data_url: String,
    entry_type: HistoryEntryType,
    model: String,
    thinking_variant: String,
    instructions: String,
    response_style: String,
}

impl AppState {
    /// Initializes app state from persisted storage.
    pub fn new(paths: AppPaths) -> Result<Self> {
        let storage = Storage::new(&paths)?;
        let settings = storage.load_settings()?;
        let auth = storage.load_auth()?;
        let catalog = storage.load_catalog()?;
        let history = storage.load_history()?;
        let history_index = history.len().saturating_sub(1);
        Ok(Self {
            inner: Arc::new(Mutex::new(StateInner {
                storage,
                settings,
                auth,
                catalog,
                status: "Ready.".to_owned(),
                history,
                history_index,
            })),
            runtime: Arc::new(Runtime::new()?),
        })
    }

    /// Returns the current frontend view model.
    pub fn view_state(&self) -> Result<AppViewState> {
        let inner = self.lock()?;
        Ok(inner.build_view())
    }

    /// Normalizes and persists settings received from the frontend.
    pub fn save_frontend_settings(&self, input: SettingsInput) -> Result<AppViewState> {
        let mut inner = self.lock()?;
        inner.settings.text_scan_model =
            normalize_model_choice(&input.text_scan_model, &inner.catalog);
        inner.settings.image_scan_model =
            normalize_model_choice(&input.image_scan_model, &inner.catalog);
        inner.settings.text_thinking_variant = normalize_thinking_choice(
            &input.text_thinking_variant,
            &inner.settings.text_scan_model,
            &inner.catalog,
        );
        inner.settings.image_thinking_variant = normalize_thinking_choice(
            &input.image_thinking_variant,
            &inner.settings.image_scan_model,
            &inner.catalog,
        );
        inner.settings.text_system_prompt_preset =
            normalize_prompt_preset(&input.text_system_prompt_preset);
        inner.settings.image_system_prompt_preset =
            normalize_prompt_preset(&input.image_system_prompt_preset);
        inner.settings.text_custom_system_prompt = input
            .text_custom_system_prompt
            .trim()
            .chars()
            .take(4000)
            .collect();
        inner.settings.image_custom_system_prompt = input
            .image_custom_system_prompt
            .trim()
            .chars()
            .take(4000)
            .collect();
        inner.settings.always_on_top = input.always_on_top;
        inner.storage.save_settings(&inner.settings)?;
        Ok(inner.build_view())
    }

    /// Starts the ChatGPT OAuth sign-in flow.
    pub fn start_chatgpt_login(&self, app_handle: AppHandle) -> Result<AppViewState> {
        let (pending, authorization_url) = chatgpt::create_login_request()?;
        {
            let mut inner = self.lock()?;
            inner.auth.pending_oauth = Some(pending.clone());
            inner.auth.error.clear();
            inner.status = "Opening ChatGPT sign-in...".to_owned();
            inner.storage.save_auth(&inner.auth)?;
        }
        let state = self.clone();
        let handle = app_handle.clone();
        self.runtime.spawn(async move {
            let result = async {
                let code = chatgpt::wait_for_oauth_callback(pending.state.clone()).await?;
                let auth = chatgpt::exchange_authorization_code(&code, &pending.verifier).await?;
                state.complete_login(auth).await
            }
            .await;
            match result {
                Ok(view) => {
                    let _ = handle.emit(
                        "app-event",
                        UiEvent::State {
                            state: Box::new(view),
                        },
                    );
                }
                Err(error) => {
                    state.set_auth_error(&error.to_string());
                    let _ = handle.emit(
                        "app-event",
                        UiEvent::Error {
                            message: error.to_string(),
                        },
                    );
                }
            }
        });
        shell::open_url(&authorization_url)?;
        self.view_state()
    }

    /// Clears stored ChatGPT authentication state.
    pub fn sign_out_chatgpt(&self) -> Result<AppViewState> {
        let mut inner = self.lock()?;
        inner.auth = AuthStorage::default();
        inner.storage.save_auth(&inner.auth)?;
        inner.status = "Signed out of ChatGPT.".to_owned();
        Ok(inner.build_view())
    }

    /// Fetches the latest ChatGPT model catalog for the signed-in account.
    pub fn refresh_chatgpt_models(&self) -> Result<AppViewState> {
        let state = self.clone();
        self.runtime.block_on(async move {
            let access = state.access_context().await?;
            let mut catalog = chatgpt::fetch_model_catalog(&access).await?;
            catalog.chatgpt_limit_label = chatgpt::fetch_usage_limit_label(&access)
                .await
                .unwrap_or_default();
            let mut inner = state.lock()?;
            inner.catalog = catalog;
            inner.settings.text_scan_model =
                normalize_model_choice(&inner.settings.text_scan_model, &inner.catalog);
            inner.settings.image_scan_model =
                normalize_model_choice(&inner.settings.image_scan_model, &inner.catalog);
            inner.settings.text_thinking_variant = normalize_thinking_choice(
                &inner.settings.text_thinking_variant,
                &inner.settings.text_scan_model,
                &inner.catalog,
            );
            inner.settings.image_thinking_variant = normalize_thinking_choice(
                &inner.settings.image_thinking_variant,
                &inner.settings.image_scan_model,
                &inner.catalog,
            );
            inner.storage.save_catalog(&inner.catalog)?;
            inner.storage.save_settings(&inner.settings)?;
            inner.status = "ChatGPT models refreshed.".to_owned();
            Ok(inner.build_view())
        })
    }

    /// Refreshes the displayed ChatGPT usage-limit label.
    pub fn refresh_chatgpt_limits(&self) -> Result<AppViewState> {
        let state = self.clone();
        self.runtime.block_on(async move {
            let access = state.access_context().await?;
            let limit = chatgpt::fetch_usage_limit_label(&access).await?;
            let mut inner = state.lock()?;
            inner.catalog.chatgpt_limit_label = limit;
            inner.storage.save_catalog(&inner.catalog)?;
            inner.status = "ChatGPT limits refreshed.".to_owned();
            Ok(inner.build_view())
        })
    }

    /// Submits manual text or pasted image input for a ChatGPT response.
    pub fn submit_manual_input(
        &self,
        input: ManualInput,
        app_handle: AppHandle,
    ) -> Result<AppViewState> {
        let text = input.text.trim().to_owned();
        let image_data_url = input.image_data_url.trim().to_owned();
        if text.is_empty() && image_data_url.is_empty() {
            return Err(anyhow!("Enter text or add an image first."));
        }
        let entry_type = if image_data_url.is_empty() {
            HistoryEntryType::Ask
        } else {
            HistoryEntryType::Image
        };
        let work = {
            let mut inner = self.lock()?;
            let scan_kind = if image_data_url.is_empty() {
                ScanKind::Text
            } else {
                ScanKind::Image
            };
            let response_style = if matches!(entry_type, HistoryEntryType::Ask) {
                ASK_RESPONSE_STYLE
            } else {
                SCAN_RESPONSE_STYLE
            };
            let prompt = if image_data_url.is_empty() {
                text.clone()
            } else if text.trim().is_empty() {
                ".".to_owned()
            } else {
                text.clone()
            };
            let work = inner.build_pending_response(
                prompt,
                text,
                image_data_url,
                entry_type,
                scan_kind,
                response_style,
            );
            inner.status = "Generating answer...".to_owned();
            (inner.build_view(), work)
        };
        self.spawn_chat_response(work.1, app_handle);
        Ok(work.0)
    }

    /// Submits OCR text or a cropped image scan for a ChatGPT response.
    pub fn submit_scan(&self, input: ScanInput, app_handle: AppHandle) -> Result<AppViewState> {
        let area = input.area.normalized();
        if !area.usable() {
            return Err(anyhow!("Selected area is too small."));
        }
        let text = input.text.trim().to_owned();
        let image_data_url = input.image_data_url.trim().to_owned();
        if input.kind == ScanKind::Text && text.is_empty() {
            return Err(anyhow!(
                "OCR did not find readable text in the selected area."
            ));
        }
        if input.kind == ScanKind::Image && image_data_url.is_empty() {
            return Err(anyhow!("Could not crop the selected image area."));
        }
        let prompt = match input.kind {
            ScanKind::Text => format!(
                "Answer the text extracted from the selected area. For math, solve it. Keep the answer concise.\n\n{text}"
            ),
            ScanKind::Image => ".".to_owned(),
        };
        let history_input = if input.kind == ScanKind::Text {
            text
        } else {
            String::new()
        };
        let history_image = if input.kind == ScanKind::Image {
            image_data_url.clone()
        } else {
            String::new()
        };
        let work = {
            let mut inner = self.lock()?;
            match input.kind {
                ScanKind::Text => inner.settings.last_text_area = Some(area),
                ScanKind::Image => inner.settings.last_image_area = Some(area),
            }
            inner.storage.save_settings(&inner.settings)?;
            let work = inner.build_pending_response(
                prompt,
                history_input,
                history_image,
                HistoryEntryType::from(input.kind),
                input.kind,
                SCAN_RESPONSE_STYLE,
            );
            inner.status = "Generating answer...".to_owned();
            (inner.build_view(), work)
        };
        self.spawn_chat_response(work.1, app_handle);
        Ok(work.0)
    }

    /// Returns or starts the most recently saved scan area for the requested kind.
    pub fn repeat_scan(&self, kind: ScanKind) -> Result<Option<SelectionArea>> {
        let inner = self.lock()?;
        let area = match kind {
            ScanKind::Text => inner.settings.last_text_area.clone(),
            ScanKind::Image => inner.settings.last_image_area.clone(),
        };
        Ok(area.filter(SelectionArea::usable))
    }

    /// Moves the selected history entry by the supplied offset.
    pub fn select_history_by_offset(&self, offset: isize) -> Result<AppViewState> {
        let mut inner = self.lock()?;
        if inner.history.is_empty() {
            inner.history_index = 0;
            return Ok(inner.build_view());
        }
        let next = (inner.history_index as isize + offset)
            .max(0)
            .min(inner.history.len() as isize - 1) as usize;
        inner.history_index = next;
        Ok(inner.build_view())
    }

    /// Clears all stored answer history.
    pub fn delete_history(&self) -> Result<AppViewState> {
        let mut inner = self.lock()?;
        inner.history.clear();
        inner.history_index = 0;
        inner.storage.clear_history()?;
        inner.status = "History deleted.".to_owned();
        Ok(inner.build_view())
    }

    /// Persists the always-on-top window setting.
    pub fn set_always_on_top(&self, enabled: bool) -> Result<AppViewState> {
        let mut inner = self.lock()?;
        inner.settings.always_on_top = enabled;
        inner.storage.save_settings(&inner.settings)?;
        Ok(inner.build_view())
    }

    /// Opens the developer website in the default browser.
    pub fn open_developer_site(&self) -> Result<()> {
        shell::open_url("https://www.bariskisir.com")
    }
    /// Opens the source repository in the default browser.
    pub fn open_source_site(&self) -> Result<()> {
        shell::open_url("https://github.com/bariskisir/ChatGPTHelper")
    }

    /// Runs ChatGPT response generation on the background runtime.
    fn spawn_chat_response(&self, work: PendingResponse, app_handle: AppHandle) {
        let state = self.clone();
        self.runtime.spawn(async move {
            let result = state
                .execute_chat_response(work.clone(), app_handle.clone())
                .await;
            match result {
                Ok(view) => {
                    let _ = app_handle.emit(
                        "app-event",
                        UiEvent::State {
                            state: Box::new(view),
                        },
                    );
                }
                Err(error) => {
                    state.set_status(&format!("Could not generate answer: {error}"));
                    let _ = app_handle.emit(
                        "app-event",
                        UiEvent::Error {
                            message: error.to_string(),
                        },
                    );
                }
            }
        });
    }

    /// Streams a ChatGPT response and stores the completed history entry.
    async fn execute_chat_response(
        &self,
        work: PendingResponse,
        app_handle: AppHandle,
    ) -> Result<AppViewState> {
        let access = self.access_context().await?;
        let image = if work.history_image_data_url.is_empty() {
            None
        } else {
            Some(work.history_image_data_url.clone())
        };
        let request = chatgpt::HelperRequest {
            prompt: work.prompt.clone(),
            image_data_url: image,
            model: work.model.clone(),
            thinking_variant: work.thinking_variant.clone(),
            instructions: work.instructions.clone(),
            response_style: work.response_style.clone(),
        };
        let final_answer = chatgpt::stream_helper_response(&access, request, move |partial| {
            let _ = app_handle.emit(
                "app-event",
                UiEvent::Answer {
                    answer: partial,
                    streaming: true,
                },
            );
        })
        .await?;
        let mut inner = self.lock()?;
        inner.history.push(HistoryEntry {
            input: work.history_input,
            input_image_data_url: work.history_image_data_url,
            output: final_answer,
            entry_type: work.entry_type,
            created_at: Utc::now(),
        });
        if inner.history.len() > HISTORY_LIMIT {
            let drop_count = inner.history.len() - HISTORY_LIMIT;
            inner.history.drain(0..drop_count);
        }
        inner.history_index = inner.history.len().saturating_sub(1);
        inner.storage.save_history(&inner.history)?;
        inner.status = "Answer ready.".to_owned();
        Ok(inner.build_view())
    }

    /// Stores successful ChatGPT authentication and refreshes account data.
    async fn complete_login(&self, auth: AuthStorage) -> Result<AppViewState> {
        let access = chatgpt::AccessContext::from_auth(&auth);
        let mut catalog = chatgpt::fetch_model_catalog(&access)
            .await
            .unwrap_or_else(|_| CatalogStorage::default());
        catalog.chatgpt_limit_label = chatgpt::fetch_usage_limit_label(&access)
            .await
            .unwrap_or_default();
        let mut inner = self.lock()?;
        inner.auth = auth;
        inner.catalog = catalog;
        inner.settings.text_scan_model =
            normalize_model_choice(&inner.settings.text_scan_model, &inner.catalog);
        inner.settings.image_scan_model =
            normalize_model_choice(&inner.settings.image_scan_model, &inner.catalog);
        inner.status = "Signed in with ChatGPT.".to_owned();
        inner.storage.save_auth(&inner.auth)?;
        inner.storage.save_catalog(&inner.catalog)?;
        inner.storage.save_settings(&inner.settings)?;
        Ok(inner.build_view())
    }

    /// Returns a valid ChatGPT access context, refreshing tokens when needed.
    async fn access_context(&self) -> Result<chatgpt::AccessContext> {
        let auth = {
            let inner = self.lock()?;
            inner.auth.clone()
        };
        if auth.access_token.is_empty() && auth.refresh_token.is_empty() {
            return Err(anyhow!("Please sign in with ChatGPT first."));
        }
        if !auth.access_token.is_empty()
            && auth.expires_at > Utc::now().timestamp_millis() + 5 * 60 * 1000
        {
            return Ok(chatgpt::AccessContext::from_auth(&auth));
        }
        let refreshed = chatgpt::refresh_access_token(&auth).await?;
        let access = chatgpt::AccessContext::from_auth(&refreshed);
        let mut inner = self.lock()?;
        inner.auth = refreshed;
        inner.storage.save_auth(&inner.auth)?;
        Ok(access)
    }

    /// Updates the shared status message when state locking succeeds.
    fn set_status(&self, message: &str) {
        if let Ok(mut inner) = self.lock() {
            inner.status = message.to_owned();
        }
    }

    /// Records an authentication error in state and persisted auth storage.
    fn set_auth_error(&self, message: &str) {
        if let Ok(mut inner) = self.lock() {
            inner.auth.error = message.to_owned();
            inner.status = format!("ChatGPT sign-in failed: {message}");
            let _ = inner.storage.save_auth(&inner.auth);
        }
    }

    /// Locks the shared application state with a user-facing error on failure.
    fn lock(&self) -> Result<MutexGuard<'_, StateInner>> {
        self.inner
            .lock()
            .map_err(|_| anyhow!("App state lock failed"))
    }
}

impl StateInner {
    /// Builds the serializable state consumed by the frontend.
    fn build_view(&self) -> AppViewState {
        let history_index = if self.history.is_empty() {
            0
        } else {
            self.history_index.min(self.history.len() - 1)
        };
        AppViewState {
            settings: self.settings.clone(),
            status: self.status.clone(),
            chatgpt: ChatGptViewState {
                logged_in: !self.auth.access_token.is_empty()
                    || !self.auth.refresh_token.is_empty(),
                account_email: self.auth.account_email.clone(),
                limit_label: self.catalog.chatgpt_limit_label.clone(),
                error: self.auth.error.clone(),
            },
            models: self.catalog.available_models.clone(),
            text_thinking_variants: resolve_thinking_variants(
                &self.settings.text_scan_model,
                &self.catalog,
            ),
            image_thinking_variants: resolve_thinking_variants(
                &self.settings.image_scan_model,
                &self.catalog,
            ),
            history: self.history.clone(),
            history_index,
            selected_history: self.history.get(history_index).cloned(),
        }
    }

    /// Creates the ChatGPT work item from prompt, history, and settings.
    fn build_pending_response(
        &self,
        prompt: String,
        history_input: String,
        history_image_data_url: String,
        entry_type: HistoryEntryType,
        scan_kind: ScanKind,
        response_style: &str,
    ) -> PendingResponse {
        let (model, thinking_variant) = match scan_kind {
            ScanKind::Text => (
                self.settings.text_scan_model.clone(),
                self.settings.text_thinking_variant.clone(),
            ),
            ScanKind::Image => (
                self.settings.image_scan_model.clone(),
                self.settings.image_thinking_variant.clone(),
            ),
        };
        PendingResponse {
            prompt,
            history_input,
            history_image_data_url,
            entry_type,
            model,
            thinking_variant,
            instructions: resolve_system_prompt(&self.settings, scan_kind),
            response_style: response_style.to_owned(),
        }
    }
}

/// Converts a frontend prompt preset string into the domain enum.
fn normalize_prompt_preset(value: &str) -> SystemPromptPreset {
    match value {
        "none" => SystemPromptPreset::None,
        "other" => SystemPromptPreset::Other,
        _ => SystemPromptPreset::Solver,
    }
}

/// Keeps a model selection valid against the current catalog.
fn normalize_model_choice(value: &str, catalog: &CatalogStorage) -> String {
    if catalog
        .available_models
        .iter()
        .any(|item| item.model == value)
    {
        value.to_owned()
    } else {
        catalog
            .available_models
            .iter()
            .find(|item| item.is_default)
            .or_else(|| catalog.available_models.first())
            .map(|item| item.model.clone())
            .unwrap_or_else(|| crate::domain::DEFAULT_MODEL.to_owned())
    }
}

/// Keeps a reasoning-effort selection valid for the chosen model.
fn normalize_thinking_choice(value: &str, model: &str, catalog: &CatalogStorage) -> String {
    let variants = resolve_thinking_variants(model, catalog);
    if variants.iter().any(|item| item.value == value) {
        value.to_owned()
    } else {
        catalog
            .available_models
            .iter()
            .find(|item| item.model == model)
            .map(|item| item.default_thinking_variant.clone())
            .filter(|item| !item.is_empty())
            .unwrap_or_else(|| crate::domain::DEFAULT_THINKING_VARIANT.to_owned())
    }
}

/// Resolves the final system prompt for a scan kind and settings.
fn resolve_system_prompt(settings: &AppSettings, kind: ScanKind) -> String {
    let (preset, custom) = match kind {
        ScanKind::Text => (
            &settings.text_system_prompt_preset,
            &settings.text_custom_system_prompt,
        ),
        ScanKind::Image => (
            &settings.image_system_prompt_preset,
            &settings.image_custom_system_prompt,
        ),
    };
    match preset {
        SystemPromptPreset::None => String::new(),
        SystemPromptPreset::Other => {
            if custom.trim().is_empty() {
                kind.solver_prompt().to_owned()
            } else {
                custom.trim().to_owned()
            }
        }
        SystemPromptPreset::Solver => kind.solver_prompt().to_owned(),
    }
}
