//! Persistent preferences, stored at `$XDG_CONFIG_HOME/openplant/settings.json`.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub sidebar_w: f32,
    pub navigator_w: f32,
    pub ai_provider: String,
    pub ai_endpoint: String,
    pub ai_local_endpoint: String,
    pub ai_key: String,
    pub ai_model: String,
    pub ai_local_model: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            sidebar_w: 264.0,
            navigator_w: 300.0,
            ai_provider: "local".into(),
            ai_endpoint: "https://api.openai.com/v1".into(),
            ai_local_endpoint: "http://localhost:11434/v1".into(),
            ai_key: String::new(),
            ai_model: "gpt-4o-mini".into(),
            ai_local_model: String::new(),
        }
    }
}

fn path() -> Option<PathBuf> {
    // Tests never read or write the user's real preferences.
    if cfg!(test) {
        return None;
    }
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    Some(base.join("openplant").join("settings.json"))
}

impl Settings {
    pub fn load() -> Self {
        path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let Some(p) = path() else { return };
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(s) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(p, s);
        }
    }
}
