//! AI diagram generator (OpenAI-compatible chat API; local Ollama or cloud).

use crate::app::OpenPlant;
use crate::plantuml::parse_plantuml;
use crate::services::settings::Settings;
use crate::workbench::ToastKind;
use gpui_kit::component::input::{InputState, TextareaState};
use gpui_kit::*;
use serde_json::{Value, json};

pub const SYSTEM: &str = "You are a PlantUML generator. Reply with ONLY a PlantUML class diagram, no explanations, no markdown fences. Start with @startuml, end with @enduml. Use \"class Name\" lines and relations like \"A --> B : label\", \"A --|> B\", \"A o-- B\".";

pub const EXAMPLES: &[&str] = &[
    "Online shop with customers, orders and products",
    "Library with books, members and loans",
    "Blog platform with posts, authors and comments",
];

pub struct AiState {
    pub local: bool,
    pub prompt: Entity<TextareaState>,
    pub endpoint: Entity<InputState>,
    pub local_endpoint: Entity<InputState>,
    pub key: Entity<InputState>,
    pub model: Entity<InputState>,
    pub local_model: String,
    pub local_models: Vec<String>,
    pub models_error: String,
    pub busy: bool,
    pub error: String,
    pub status: String,
    pub settings_open: bool,
}

impl AiState {
    pub fn new(s: &Settings, window: &mut Window, cx: &mut App) -> Self {
        let field = |v: &str, ph: &str, window: &mut Window, cx: &mut App| {
            let (v, ph) = (v.to_string(), ph.to_string());
            cx.new(|cx| InputState::new(window, cx).default_value(v).placeholder(ph))
        };
        Self {
            local: s.ai_provider != "cloud",
            prompt: cx.new(|cx| {
                TextareaState::new(window, cx)
                    .default_value("An online shop with customers, orders and products")
                    .placeholder("e.g. An online shop with customers, orders and products")
            }),
            endpoint: field(&s.ai_endpoint, "https://api.openai.com/v1", window, cx),
            local_endpoint: field(
                &s.ai_local_endpoint,
                "http://localhost:11434/v1",
                window,
                cx,
            ),
            key: {
                let v = s.ai_key.clone();
                cx.new(|cx| {
                    InputState::new(window, cx)
                        .masked(true)
                        .default_value(v)
                        .placeholder("sk-... (stored only on this machine)")
                })
            },
            model: field(&s.ai_model, "gpt-4o-mini", window, cx),
            local_model: s.ai_local_model.clone(),
            local_models: vec![],
            models_error: String::new(),
            busy: false,
            error: String::new(),
            status: String::new(),
            settings_open: false,
        }
    }
}

pub(crate) fn api_root(base: &str) -> String {
    base.trim_end_matches('/')
        .trim_end_matches("/v1")
        .to_string()
}

impl OpenPlant {
    pub fn ai_refresh_models(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let root = api_root(&self.ai.local_endpoint.read(cx).value());
        self.ai.models_error.clear();
        let task = cx.background_executor().spawn(async move {
            let mut res = ureq::get(&format!("{root}/api/tags"))
                .call()
                .map_err(|e| e.to_string())?;
            let v: Value = res.body_mut().read_json().map_err(|e| e.to_string())?;
            Ok::<Vec<String>, String>(
                v["models"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|m| m["name"].as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default(),
            )
        });
        cx.spawn_in(window, async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                match result {
                    Ok(names) => {
                        if !names.is_empty() && !names.contains(&this.ai.local_model) {
                            this.ai.local_model = names[0].clone();
                        }
                        this.ai.local_models = names;
                    }
                    Err(_) => {
                        this.ai.local_models.clear();
                        this.ai.models_error =
                            "Ollama tidak terjangkau. Jalankan `ollama serve` dulu.".into();
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn ai_generate(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let local = self.ai.local;
        let base = if local {
            self.ai.local_endpoint.read(cx).value()
        } else {
            self.ai.endpoint.read(cx).value()
        };
        let base = base.trim_end_matches('/').to_string();
        let model = if local {
            self.ai.local_model.trim().to_string()
        } else {
            self.ai.model.read(cx).value().trim().to_string()
        };
        let key = self.ai.key.read(cx).value().trim().to_string();
        let prompt = self.ai.prompt.read(cx).value().to_string();
        // Remember provider settings for the next session.
        self.settings.ai_provider = if local {
            "local".into()
        } else {
            "cloud".into()
        };
        self.settings.ai_endpoint = self.ai.endpoint.read(cx).value().to_string();
        self.settings.ai_local_endpoint = self.ai.local_endpoint.read(cx).value().to_string();
        self.settings.ai_key = key.clone();
        self.settings.ai_model = self.ai.model.read(cx).value().to_string();
        self.settings.ai_local_model = self.ai.local_model.clone();
        self.settings.save();
        if model.is_empty() {
            self.ai.error = if local {
                "Pilih dulu model Ollama lokal.".into()
            } else {
                "Isi nama model.".into()
            };
            cx.notify();
            return;
        }
        self.ai.busy = true;
        self.ai.error.clear();
        self.ai.status.clear();
        cx.notify();
        let task = cx.background_executor().spawn(async move {
            let body = json!({
                "model": model,
                "temperature": 0.2,
                "messages": [{"role": "system", "content": SYSTEM}, {"role": "user", "content": prompt}],
            });
            let mut req = ureq::post(&format!("{base}/chat/completions")).header("Content-Type", "application/json");
            if !local && !key.is_empty() {
                req = req.header("Authorization", &format!("Bearer {key}"));
            }
            let mut res = req.send_json(&body).map_err(|e| e.to_string())?;
            let v: Value = res.body_mut().read_json().map_err(|e| e.to_string())?;
            Ok::<String, String>(v["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string())
        });
        cx.spawn_in(window, async move |this, cx| {
            let result = task.await;
            let _ = this.update_in(cx, |this, window, cx| {
                this.ai.busy = false;
                match result {
                    Ok(content) => {
                        let text = match (content.find("@startuml"), content.rfind("@enduml")) {
                            (Some(a), Some(b)) if b > a => content[a..b + 7].to_string(),
                            _ => content,
                        };
                        let parsed = parse_plantuml(&text);
                        if parsed.shapes.is_empty() {
                            this.ai.error = "AI returned no parseable classes.".into();
                        } else {
                            let (n, r) = (parsed.shapes.len(), parsed.relations.len());
                            this.wb.replace_from_plantuml(parsed);
                            this.pending_fit = true;
                            this.ai.status = format!("Generated {n} classes, {r} relations.");
                            this.toast(
                                ToastKind::Success,
                                format!("AI generated {n} classes."),
                                window,
                                cx,
                            );
                        }
                    }
                    Err(e) => {
                        this.ai.error = e.clone();
                        this.toast(
                            ToastKind::Error,
                            format!("AI generate failed: {e}"),
                            window,
                            cx,
                        );
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }
}
