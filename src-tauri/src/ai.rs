use crate::error::AppError;
use crate::store::{Storable, Store};
use futures_util::StreamExt;
use once_cell::sync::Lazy;
use rusqlite::{params, Result as RusqliteResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter, Manager, State};

const AI_KEYRING_SERVICE: &str = "dev.byteatatime.flare.ai";
const AI_KEYRING_USERNAME: &str = "openrouter_api_key";
const AI_USAGE_SCHEMA: &str = "CREATE TABLE IF NOT EXISTS ai_generations (
    id TEXT PRIMARY KEY,
    created INTEGER NOT NULL,
    model TEXT NOT NULL,
    tokens_prompt INTEGER NOT NULL,
    tokens_completion INTEGER NOT NULL,
    native_tokens_prompt INTEGER NOT NULL,
    native_tokens_completion INTEGER NOT NULL,
    total_cost REAL NOT NULL
)";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AskOptions {
    pub model: Option<String>,
    pub creativity: Option<String>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StreamChunk {
    request_id: String,
    text: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StreamEnd {
    request_id: String,
    full_text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GenerationData {
    pub id: String,
    pub created: i64,
    pub model: String,
    #[serde(default)]
    pub tokens_prompt: i64,
    #[serde(default)]
    pub tokens_completion: i64,
    #[serde(default)]
    pub native_tokens_prompt: i64,
    #[serde(default)]
    pub native_tokens_completion: i64,
    #[serde(default)]
    pub total_cost: f64,
}

impl Storable for GenerationData {
    fn from_row(row: &rusqlite::Row) -> RusqliteResult<Self> {
        Ok(GenerationData {
            id: row.get(0)?,
            created: row.get(1)?,
            model: row.get(2)?,
            tokens_prompt: row.get(3)?,
            tokens_completion: row.get(4)?,
            native_tokens_prompt: row.get(5)?,
            native_tokens_completion: row.get(6)?,
            total_cost: row.get(7)?,
        })
    }
}

static DEFAULT_AI_MODELS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    // OpenAI
    m.insert("OpenAI_GPT4.1", "openai/gpt-4.1");
    m.insert("OpenAI_GPT4.1-mini", "openai/gpt-4.1-mini");
    m.insert("OpenAI_GPT4.1-nano", "openai/gpt-4.1-nano");
    m.insert("OpenAI_GPT4", "openai/gpt-4");
    m.insert("OpenAI_GPT4-turbo", "openai/gpt-4-turbo");
    m.insert("OpenAI_GPT4o", "openai/gpt-4o");
    m.insert("OpenAI_GPT4o-mini", "openai/gpt-4o-mini");
    m.insert("OpenAI_o3", "openai/o3");
    m.insert("OpenAI_o4-mini", "openai/o4-mini");
    m.insert("OpenAI_o1", "openai/o1");
    m.insert("OpenAI_o3-mini", "openai/o3-mini");
    // Anthropic
    m.insert("Anthropic_Claude_Haiku", "anthropic/claude-3-haiku");
    m.insert("Anthropic_Claude_Sonnet", "anthropic/claude-3-sonnet");
    m.insert("Anthropic_Claude_Sonnet_3.7", "anthropic/claude-3.7-sonnet");
    m.insert("Anthropic_Claude_Opus", "anthropic/claude-3-opus");
    m.insert("Anthropic_Claude_4_Sonnet", "anthropic/claude-sonnet-4");
    m.insert("Anthropic_Claude_4_Opus", "anthropic/claude-opus-4");
    // Perplexity
    m.insert("Perplexity_Sonar", "perplexity/sonar");
    m.insert("Perplexity_Sonar_Pro", "perplexity/sonar-pro");
    m.insert("Perplexity_Sonar_Reasoning", "perplexity/sonar-reasoning");
    m.insert(
        "Perplexity_Sonar_Reasoning_Pro",
        "perplexity/sonar-reasoning-pro",
    );
    // Meta
    m.insert("Llama4_Scout", "meta-llama/llama-4-scout");
    m.insert("Llama3.3_70B", "meta-llama/llama-3.3-70b-instruct");
    m.insert("Llama3.1_8B", "meta-llama/llama-3.1-8b-instruct");
    m.insert("Llama3.1_405B", "meta-llama/llama-3.1-405b-instruct");
    // Mistral
    m.insert("Mistral_Nemo", "mistralai/mistral-nemo");
    m.insert("Mistral_Large", "mistralai/mistral-large");
    m.insert("Mistral_Medium", "mistralai/mistral-medium-3");
    m.insert("Mistral_Small", "mistralai/mistral-small");
    m.insert("Mistral_Codestral", "mistralai/codestral-2501");
    // DeepSeek
    m.insert(
        "DeepSeek_R1_Distill_Llama_3.3_70B",
        "deepseek/deepseek-r1-distill-llama-70b",
    );
    m.insert("DeepSeek_R1", "deepseek/deepseek-r1");
    m.insert("DeepSeek_V3", "deepseek/deepseek-chat");
    // Google
    m.insert("Google_Gemini_2.5_Pro", "google/gemini-2.5-pro");
    m.insert("Google_Gemini_2.5_Flash", "google/gemini-2.5-flash");
    m.insert("Google_Gemini_2.0_Flash", "google/gemini-2.0-flash-001");
    // xAI
    m.insert("xAI_Grok_3", "x-ai/grok-3");
    m.insert("xAI_Grok_3_Mini", "x-ai/grok-3-mini");
    m.insert("xAI_Grok_2", "x-ai/grok-2-1212");

    m
});

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AiSettings {
    enabled: bool,
    model_associations: HashMap<String, String>,
}

fn get_settings_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|_| "Failed to get app local data dir".to_string())?;

    if !data_dir.exists() {
        fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
    }
    Ok(data_dir.join("ai_settings.json"))
}

fn read_settings(path: &Path) -> Result<AiSettings, String> {
    if !path.exists() {
        return Ok(AiSettings::default());
    }
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    if content.trim().is_empty() {
        return Ok(AiSettings::default());
    }
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

fn write_settings(path: &Path, settings: &AiSettings) -> Result<(), String> {
    let content = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_ai_settings(app: tauri::AppHandle) -> Result<AiSettings, String> {
    let path = get_settings_path(&app)?;
    let mut user_settings = read_settings(&path)?;

    for (key, &default_value) in DEFAULT_AI_MODELS.iter() {
        let entry = user_settings
            .model_associations
            .entry(key.to_string())
            .or_insert_with(|| default_value.to_string());

        if entry.is_empty() {
            *entry = default_value.to_string();
        }
    }

    Ok(user_settings)
}

#[tauri::command]
pub fn set_ai_settings(app: tauri::AppHandle, settings: AiSettings) -> Result<(), String> {
    let path = get_settings_path(&app)?;

    let mut settings_to_save = AiSettings {
        enabled: settings.enabled,
        model_associations: HashMap::new(),
    };

    for (key, value) in settings.model_associations {
        let is_different_from_default = DEFAULT_AI_MODELS
            .get(key.as_str())
            .map_or(true, |&default_val| default_val != value);

        if is_different_from_default {
            settings_to_save.model_associations.insert(key, value);
        }
    }

    write_settings(&path, &settings_to_save)
}

fn get_keyring_entry() -> Result<keyring::Entry, AppError> {
    keyring::Entry::new(AI_KEYRING_SERVICE, AI_KEYRING_USERNAME).map_err(AppError::from)
}

#[tauri::command]
pub fn set_ai_api_key(key: String) -> Result<(), String> {
    get_keyring_entry()
        .and_then(|entry| entry.set_password(&key).map_err(AppError::from))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn is_ai_api_key_set() -> Result<bool, String> {
    match get_keyring_entry().and_then(|entry| entry.get_password().map_err(AppError::from)) {
        Ok(_) => Ok(true),
        Err(AppError::Keyring(keyring::Error::NoEntry)) => Ok(false),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn clear_ai_api_key() -> Result<(), String> {
    get_keyring_entry()
        .and_then(|entry| entry.delete_credential().map_err(AppError::from))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn ai_can_access(app: tauri::AppHandle) -> Result<bool, String> {
    let settings = get_ai_settings(app)?;
    if !settings.enabled {
        return Ok(false);
    }
    is_ai_api_key_set()
}

pub struct AiUsageManager {
    store: Store,
}

impl AiUsageManager {
    pub fn new(app_handle: &AppHandle) -> Result<Self, AppError> {
        let store = Store::new(app_handle, "ai_usage.sqlite")?;
        store.init_table(AI_USAGE_SCHEMA)?;
        Ok(Self { store })
    }

    pub fn log_generation(&self, data: &GenerationData) -> Result<(), AppError> {
        self.store.execute(
            "INSERT OR REPLACE INTO ai_generations (id, created, model, tokens_prompt, tokens_completion, native_tokens_prompt, native_tokens_completion, total_cost)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                data.id,
                data.created,
                data.model,
                data.tokens_prompt,
                data.tokens_completion,
                data.native_tokens_prompt,
                data.native_tokens_completion,
                data.total_cost
            ],
        )?;
        Ok(())
    }

    pub fn get_history(&self, limit: u32, offset: u32) -> Result<Vec<GenerationData>, AppError> {
        self.store.query(
            "SELECT id, created, model, tokens_prompt, tokens_completion, native_tokens_prompt, native_tokens_completion, total_cost FROM ai_generations ORDER BY created DESC LIMIT ?1 OFFSET ?2",
            params![limit, offset],
        )
    }
}

#[tauri::command]
pub fn get_ai_usage_history(
    manager: State<AiUsageManager>,
    limit: u32,
    offset: u32,
) -> Result<Vec<GenerationData>, String> {
    manager
        .get_history(limit, offset)
        .map_err(|e| e.to_string())
}

async fn fetch_and_log_usage(
    open_router_request_id: String,
    api_key: String,
    app_handle: AppHandle,
) -> Result<(), AppError> {
    let manager = app_handle.state::<AiUsageManager>();
    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "https://openrouter.ai/api/v1/generation?id={}",
            open_router_request_id
        ))
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .map_err(|e| AppError::Ai(e.to_string()))?;

    if response.status().is_success() {
        let generation_response: Value = response
            .json()
            .await
            .map_err(|e| AppError::Ai(e.to_string()))?;
        let generation_data: GenerationData =
            serde_json::from_value(generation_response["data"].clone())
                .map_err(|e| AppError::Ai(format!("Failed to parse generation data: {}", e)))?;
        manager.log_generation(&generation_data)?;
    } else {
        let error_text = response.text().await.unwrap_or_default();
        return Err(AppError::Ai(format!(
            "Failed to fetch usage data: {}",
            error_text
        )));
    }
    Ok(())
}

#[tauri::command]
pub async fn ai_ask_stream(
    app_handle: AppHandle,
    request_id: String,
    prompt: String,
    options: AskOptions,
) -> Result<(), String> {
    let settings = get_ai_settings(app_handle.clone())?;
    if !settings.enabled {
        return Err("AI features are not enabled.".to_string());
    }

    let api_key =
        match get_keyring_entry().and_then(|entry| entry.get_password().map_err(AppError::from)) {
            Ok(key) => key,
            Err(e) => return Err(e.to_string()),
        };

    let model_key = options.model.unwrap_or_else(|| "default".to_string());

    let model_id = settings
        .model_associations
        .get(&model_key)
        .cloned()
        .unwrap_or_else(|| "mistralai/mistral-7b-instruct:free".to_string());

    let temperature = match options.creativity.as_deref() {
        Some("none") => 0.0,
        Some("low") => 0.4,
        Some("medium") => 0.7,
        Some("high") => 1.0,
        _ => 0.7,
    };

    let body = serde_json::json!({
        "model": model_id,
        "messages": [{"role": "user", "content": prompt}],
        "stream": true,
        "temperature": temperature,
    });

    let client = reqwest::Client::new();
    let res = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("HTTP-Referer", "http://localhost")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let open_router_request_id = res
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    if !res.status().is_success() {
        let error_body = res.text().await.unwrap_or_else(|_| "Unknown error".into());
        return Err(format!("API Error: {}", error_body));
    }

    let mut stream = res.bytes_stream();
    let mut full_text = String::new();

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| e.to_string())?;
        let lines = String::from_utf8_lossy(&chunk);

        for line in lines.split("\n\n").filter(|s| !s.is_empty()) {
            if line.starts_with("data: ") {
                let json_str = &line[6..];
                if json_str.trim() == "[DONE]" {
                    break;
                }
                if let Ok(json) = serde_json::from_str::<Value>(json_str) {
                    if let Some(delta) = json
                        .get("choices")
                        .and_then(|c| c.get(0))
                        .and_then(|c0| c0.get("delta"))
                    {
                        if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                            full_text.push_str(content);
                            app_handle
                                .emit(
                                    "ai-stream-chunk",
                                    StreamChunk {
                                        request_id: request_id.clone(),
                                        text: content.to_string(),
                                    },
                                )
                                .map_err(|e| e.to_string())?;
                        }
                    }
                }
            }
        }
    }

    app_handle
        .emit(
            "ai-stream-end",
            StreamEnd {
                request_id: request_id.clone(),
                full_text: full_text.clone(),
            },
        )
        .map_err(|e| e.to_string())?;

    if let Some(or_req_id) = open_router_request_id {
        let handle_clone = app_handle.clone();
        tokio::spawn(async move {
            if let Err(e) = fetch_and_log_usage(or_req_id, api_key, handle_clone).await {
                eprintln!("[AI Usage Tracking] Error: {}", e);
            }
        });
    }

    Ok(())
}
