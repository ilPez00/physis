use crate::ai::{AiError, AiResult, tools::ToolDefinition};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub base_url: String,
    pub model: String,
    pub api_key: String,
    pub supports_vision: bool,
    pub supports_tools: bool,
    pub max_tokens: u32,
    pub timeout_secs: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: Content,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallMsg>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Content {
    Text(String),
    Parts(Vec<ContentPart>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentPart {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<ImageUrl>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrl {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallMsg {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<Choice>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub message: ResponseMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMessage {
    pub role: String,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub tool_calls: Option<Vec<ToolCallMsg>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

pub struct Provider {
    config: ProviderConfig,
    key_idx: AtomicI64,
    available: AtomicBool,
    cooldown_until: AtomicI64,
    client: reqwest::Client,
}

impl Provider {
    pub fn new(config: ProviderConfig) -> Self {
        let timeout = std::time::Duration::from_secs_f64(config.timeout_secs.max(5.0));
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .unwrap_or_default();
        Self {
            config,
            key_idx: AtomicI64::new(0),
            available: AtomicBool::new(true),
            cooldown_until: AtomicI64::new(0),
            client,
        }
    }

    pub fn name(&self) -> &str {
        &self.config.name
    }

    pub fn model(&self) -> &str {
        &self.config.model
    }

    pub fn supports_tools(&self) -> bool {
        self.config.supports_tools
    }

    pub fn is_available(&self) -> bool {
        self.available.load(Ordering::Relaxed)
            && std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .ok()
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0)
                >= self.cooldown_until.load(Ordering::Relaxed)
    }

    pub fn cooldown(&self, seconds: i64) {
        let until = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .map(|d| d.as_secs() as i64 + seconds)
            .unwrap_or(seconds);
        self.cooldown_until.store(until, Ordering::Relaxed);
    }

    pub fn mark_unavailable(&self) {
        self.available.store(false, Ordering::Relaxed);
    }

    pub async fn complete(
        &self,
        messages: &[Message],
        tools: Option<&[ToolDefinition]>,
        max_tokens: Option<u32>,
    ) -> AiResult<ResponseMessage> {
        let mut req = ChatRequest {
            model: self.config.model.clone(),
            messages: messages.to_vec(),
            tools: tools.map(|t| t.to_vec()),
            tool_choice: None,
            max_tokens: max_tokens.or(Some(self.config.max_tokens)),
            temperature: Some(0.7),
        };

        if req.tools.is_some() {
            req.tool_choice = Some(serde_json::json!("auto"));
        }

        let body = serde_json::to_string(&req)?;

        let url = format!(
            "{}/chat/completions",
            self.config.base_url.trim_end_matches('/')
        );

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;

        let status = resp.status();
        if status == 429 {
            self.cooldown(30);
            return Err(AiError::Provider(format!("rate limited on {}", self.config.name)));
        }
        if status == 401 || status == 403 {
            self.mark_unavailable();
            return Err(AiError::Provider(format!("auth error on {}", self.config.name)));
        }
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(AiError::Provider(format!(
                "{} returned {}: {}",
                self.config.name,
                status,
                text.chars().take(200).collect::<String>()
            )));
        }

        let cr: ChatResponse = resp.json().await?;
        cr.choices
            .into_iter()
            .next()
            .map(|c| c.message)
            .ok_or_else(|| AiError::Provider("empty response".into()))
    }

    pub async fn transcribe(&self, audio_path: &str, model: Option<&str>) -> AiResult<String> {
        let url = format!(
            "{}/audio/transcriptions",
            self.config.base_url.trim_end_matches('/')
        );

        let file_content = std::fs::read(audio_path)?;
        let part = reqwest::multipart::Part::bytes(file_content)
            .file_name("audio.wav")
            .mime_str("audio/wav")
            .unwrap();

        let model_name = model.unwrap_or("whisper-large-v3");
        let form = reqwest::multipart::Form::new()
            .part("file", part)
            .text("model", model_name.to_string());

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .multipart(form)
            .send()
            .await?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(AiError::Provider(format!("transcription failed: {}", text)));
        }

        #[derive(Deserialize)]
        struct TransResponse {
            text: String,
        }

        let tr: TransResponse = resp.json().await?;
        Ok(tr.text)
    }

    pub async fn translate(&self, text: &str, target_lang: &str) -> AiResult<String> {
        let messages = vec![
            Message {
                role: "system".into(),
                content: Content::Text(format!("Translate the following text to {}. Output ONLY the translated text.", target_lang)),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            },
            Message {
                role: "user".into(),
                content: Content::Text(text.to_string()),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            }
        ];
        
        let resp = self.complete(&messages, None, None).await?;
        Ok(resp.content.unwrap_or_default())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteHint {
    pub task_type: String,
    pub has_image: bool,
}

pub struct ProviderCascade {
    providers: Vec<Provider>,
}

impl ProviderCascade {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    pub fn add(&mut self, config: ProviderConfig) {
        self.providers.push(Provider::new(config));
    }

    pub fn from_env() -> Self {
        let mut cascade = Self::new();

        let openrouter_key = std::env::var("AI_OPENROUTER_KEY").ok().filter(|k| !k.is_empty());
        if let Some(key) = openrouter_key {
            cascade.add(ProviderConfig {
                name: "openrouter".into(),
                base_url: "https://openrouter.ai/api/v1".into(),
                model: "deepseek/deepseek-chat".into(),
                api_key: key,
                supports_vision: true,
                supports_tools: true,
                max_tokens: 4096,
                timeout_secs: 30.0,
            });
        }

        let groq_key = std::env::var("AI_GROQ_KEY").ok().filter(|k| !k.is_empty());
        if let Some(key) = groq_key {
            cascade.add(ProviderConfig {
                name: "groq".into(),
                base_url: "https://api.groq.com/openai/v1".into(),
                model: "llama-3.3-70b-versatile".into(),
                api_key: key,
                supports_vision: false,
                supports_tools: true,
                max_tokens: 8192,
                timeout_secs: 15.0,
            });
        }

        let openai_key = std::env::var("AURA_OPENAI_KEY")
            .or_else(|_| std::env::var("OPENAI_API_KEY"))
            .ok()
            .filter(|k| !k.is_empty());
        if let Some(key) = openai_key {
            let url = std::env::var("AURA_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".into());
            let model = std::env::var("AURA_MODEL").unwrap_or_else(|_| "gpt-4o-mini".into());
            cascade.add(ProviderConfig {
                name: "openai".into(),
                base_url: url,
                model,
                api_key: key,
                supports_vision: true,
                supports_tools: true,
                max_tokens: 4096,
                timeout_secs: 30.0,
            });
        }

        let deepseek_key = std::env::var("AI_DEEPSEEK_KEY").ok().filter(|k| !k.is_empty());
        if let Some(key) = deepseek_key {
            cascade.add(ProviderConfig {
                name: "deepseek".into(),
                base_url: "https://api.deepseek.com".into(),
                model: "deepseek-chat".into(),
                api_key: key,
                supports_vision: false,
                supports_tools: true,
                max_tokens: 8192,
                timeout_secs: 30.0,
            });
        }

        if let Some(keys) = std::env::var("AI_GEMINI_KEY").ok().filter(|k| !k.is_empty()) {
            let first = keys.split(',').next().unwrap_or("").to_string();
            cascade.add(ProviderConfig {
                name: "gemini".into(),
                base_url: "https://generativelanguage.googleapis.com/v1beta/openai/".into(),
                model: "gemini-2.0-flash".into(),
                api_key: first,
                supports_vision: true,
                supports_tools: true,
                max_tokens: 8192,
                timeout_secs: 30.0,
            });
        }

        // Local Ollama
        cascade.add(ProviderConfig {
            name: "ollama".into(),
            base_url: "http://localhost:11434/v1".into(),
            model: "llama3.2".into(),
            api_key: "ollama".into(),
            supports_vision: false,
            supports_tools: false,
            max_tokens: 4096,
            timeout_secs: 60.0,
        });

        cascade
    }

    pub fn classify(&self, query: &str, has_image: bool) -> RouteHint {
        if has_image {
            return RouteHint {
                task_type: "vision".into(),
                has_image: true,
            };
        }
        let q = query.to_lowercase();
        if q.len() < 80 {
            RouteHint { task_type: "fast".into(), has_image: false }
        } else {
            RouteHint { task_type: "smart".into(), has_image: false }
        }
    }

    pub fn providers_for_task(&self, route: &RouteHint) -> Vec<&Provider> {
        if route.has_image {
            self.providers
                .iter()
                .filter(|p| p.is_available() && p.config.supports_vision)
                .collect()
        } else if route.task_type == "fast" {
            self.providers
                .iter()
                .filter(|p| p.is_available() && !p.config.model.contains("gpt-4"))
                .collect()
        } else {
            self.providers
                .iter()
                .filter(|p| p.is_available())
                .collect()
        }
    }

    pub async fn complete(
        &self,
        messages: &[Message],
        tools: Option<&[ToolDefinition]>,
        route: &RouteHint,
        max_tokens: Option<u32>,
    ) -> AiResult<(ResponseMessage, String)> {
        let candidates = self.providers_for_task(route);

        let needs_tools = tools.map(|t| !t.is_empty()).unwrap_or(false);
        let provider_pool: Vec<&Provider> = if needs_tools {
            candidates
                .into_iter()
                .filter(|p| p.supports_tools())
                .collect()
        } else {
            candidates
        };

        if provider_pool.is_empty() {
            return Err(AiError::NoProvider);
        }

        for provider in &provider_pool {
            match provider.complete(messages, tools, max_tokens).await {
                Ok(resp) => return Ok((resp, provider.name().to_string())),
                Err(e) => {
                    log::warn!("provider {} failed: {:?}", provider.name(), e);
                    continue;
                }
            }
        }

        Err(AiError::NoProvider)
    }

    pub async fn translate(&self, text: &str, target_lang: &str) -> AiResult<String> {
        let best = self.providers.iter().find(|p| p.is_available());
        if let Some(p) = best {
            return p.translate(text, target_lang).await;
        }
        Err(AiError::NoProvider)
    }

    pub fn provider_names(&self) -> Vec<String> {
        self.providers.iter().map(|p| p.name().to_string()).collect()
    }

    pub async fn transcribe(&self, audio_path: &str, model: Option<&str>) -> AiResult<String> {
        let groq = self.providers.iter().find(|p| p.name() == "groq");
        if let Some(p) = groq {
            return p.transcribe(audio_path, model).await;
        }
        let openai = self.providers.iter().find(|p| p.name() == "openai");
        if let Some(p) = openai {
            return p.transcribe(audio_path, model).await;
        }
        Err(AiError::NoProvider)
    }
}

impl Default for ProviderCascade {
    fn default() -> Self {
        Self::new()
    }
}
