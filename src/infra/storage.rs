//! JSON file persistence for settings, auth, model catalog, and history.

use crate::domain::{AppSettings, AuthStorage, CatalogStorage, HISTORY_LIMIT, HistoryEntry};
use crate::infra::paths::AppPaths;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct Storage {
    settings: PathBuf,
    auth: PathBuf,
    catalog: PathBuf,
    history: PathBuf,
}

impl Storage {
    /// Initializes app state from persisted storage.
    pub fn new(paths: &AppPaths) -> Result<Self> {
        fs::create_dir_all(&paths.data_dir).context("Could not create app data directory")?;
        Ok(Self {
            settings: paths.settings.clone(),
            auth: paths.auth.clone(),
            catalog: paths.catalog.clone(),
            history: paths.history.clone(),
        })
    }

    /// Documents the l oa d s et ti ng s function.
    pub fn load_settings(&self) -> Result<AppSettings> {
        read_pretty_or_default(&self.settings, "settings")
    }

    /// Persists frontend settings and returns the refreshed view state.
    pub fn save_settings(&self, settings: &AppSettings) -> Result<()> {
        write_pretty(&self.settings, settings, "settings")
    }

    /// Documents the l oa d a ut h function.
    pub fn load_auth(&self) -> Result<AuthStorage> {
        read_pretty_or_default(&self.auth, "auth")
    }

    /// Documents the s av e a ut h function.
    pub fn save_auth(&self, auth: &AuthStorage) -> Result<()> {
        write_pretty(&self.auth, auth, "auth")
    }

    /// Documents the l oa d c at al og function.
    pub fn load_catalog(&self) -> Result<CatalogStorage> {
        read_pretty_or_default(&self.catalog, "catalog")
    }

    /// Documents the s av e c at al og function.
    pub fn save_catalog(&self, catalog: &CatalogStorage) -> Result<()> {
        write_pretty(&self.catalog, catalog, "catalog")
    }

    /// Documents the l oa d h is to ry function.
    pub fn load_history(&self) -> Result<Vec<HistoryEntry>> {
        read_pretty_or_default(&self.history, "history")
    }

    /// Documents the s av e h is to ry function.
    pub fn save_history(&self, history: &[HistoryEntry]) -> Result<()> {
        let start = history.len().saturating_sub(HISTORY_LIMIT);
        write_pretty(&self.history, &history[start..], "history")
    }

    /// Documents the c le ar h is to ry function.
    pub fn clear_history(&self) -> Result<()> {
        if self.history.exists() {
            fs::remove_file(&self.history).context("Could not remove history.json")?;
        }
        Ok(())
    }
}

/// Loads pretty-printed JSON or returns a default value when missing.
fn read_pretty_or_default<T>(path: &Path, label: &str) -> Result<T>
where
    T: serde::de::DeserializeOwned + Default,
{
    if !path.exists() {
        return Ok(T::default());
    }
    let text = fs::read_to_string(path).with_context(|| format!("Could not read {label}.json"))?;
    serde_json::from_str(&text).with_context(|| format!("Could not parse {label}.json"))
}

/// Writes a serializable value as pretty-printed JSON.
fn write_pretty<T>(path: &Path, value: &T, label: &str) -> Result<()>
where
    T: serde::Serialize + ?Sized,
{
    let text = serde_json::to_string_pretty(value)
        .with_context(|| format!("Could not serialize {label}"))?;
    fs::write(path, text).with_context(|| format!("Could not write {}", path.display()))
}
