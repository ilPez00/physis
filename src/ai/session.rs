use crate::ai::agent::{run_agent, AgentConfig};
use crate::ai::provider::{Content, ContentPart, ImageUrl, Message, ProviderCascade};
use crate::ai::tools::ToolRegistry;
use crate::ai::AiResult;

const MAX_HISTORY: usize = 30;

pub struct Session {
    provider_cascade: ProviderCascade,
    tools: ToolRegistry,
    config: AgentConfig,
    history: Vec<Message>,
}

impl Session {
    pub fn new(system_prompt: &str) -> Self {
        let cascade = ProviderCascade::from_env();
        let providers = cascade.provider_names();
        log::info!("session created with providers: {}", providers.join(", "));

        Self {
            provider_cascade: cascade,
            tools: ToolRegistry::new(),
            config: AgentConfig {
                system_prompt: system_prompt.to_string(),
                max_tokens: 4096,
                detail: "low".into(),
            },
            history: Vec::new(),
        }
    }

    pub fn provider_cascade(&self) -> &ProviderCascade {
        &self.provider_cascade
    }

    pub fn tools(&self) -> &ToolRegistry {
        &self.tools
    }

    pub fn config(&self) -> &AgentConfig {
        &self.config
    }

    pub fn history(&self) -> &[Message] {
        &self.history
    }

    pub fn push_context(&mut self, text: &str, frame_b64: Option<&str>, mode: &str) {
        let mut parts = vec![ContentPart {
            type_: "text".into(),
            text: Some(format!("[{mode} context] {text}")),
            image_url: None,
        }];
        if let Some(b64) = frame_b64 {
            parts.push(ContentPart {
                type_: "image_url".into(),
                text: None,
                image_url: Some(ImageUrl {
                    url: format!("data:image/jpeg;base64,{b64}"),
                    detail: Some("low".into()),
                }),
            });
        }
        self.history.push(Message {
            role: "user".into(),
            content: Content::Parts(parts),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        });
        self.trim_history();
    }

    pub async fn query(&mut self, question: &str, mode: &str) -> AiResult<String> {
        let output = run_agent(
            &self.provider_cascade,
            &self.tools,
            &self.config,
            &self.history,
            question,
            None,
            mode,
            None,
        )
        .await?;

        self.history.push(Message {
            role: "user".into(),
            content: Content::Text(question.to_string()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        });
        self.history.push(Message {
            role: "assistant".into(),
            content: Content::Text(output.text.clone()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        });
        self.trim_history();

        Ok(output.text)
    }

    pub async fn query_with_frame(
        &mut self,
        question: &str,
        frame_b64: &str,
        mode: &str,
    ) -> AiResult<String> {
        let output = run_agent(
            &self.provider_cascade,
            &self.tools,
            &self.config,
            &self.history,
            question,
            Some(frame_b64),
            mode,
            None,
        )
        .await?;

        self.history.push(Message {
            role: "user".into(),
            content: Content::Parts(vec![
                ContentPart {
                    type_: "text".into(),
                    text: Some(question.to_string()),
                    image_url: None,
                },
                ContentPart {
                    type_: "image_url".into(),
                    text: None,
                    image_url: Some(ImageUrl {
                        url: format!("data:image/jpeg;base64,{frame_b64}"),
                        detail: Some("low".into()),
                    }),
                },
            ]),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        });
        self.history.push(Message {
            role: "assistant".into(),
            content: Content::Text(output.text.clone()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        });
        self.trim_history();

        Ok(output.text)
    }

    pub async fn auto_observe(&mut self, mode: &str) -> AiResult<Option<String>> {
        if self.history.is_empty() {
            return Ok(None);
        }

        let observe_config = AgentConfig {
            system_prompt: self.config.system_prompt.clone(),
            max_tokens: 200,
            detail: "low".into(),
        };

        let output = run_agent(
            &self.provider_cascade,
            &self.tools,
            &observe_config,
            &[],
            "Brief observation of current context. 1-2 sentences.",
            None,
            mode,
            None,
        )
        .await?;

        Ok(Some(output.text))
    }

    pub fn provider_names(&self) -> Vec<String> {
        self.provider_cascade.provider_names()
    }

    pub fn tool_names(&self) -> Vec<String> {
        self.tools.names()
    }

    fn trim_history(&mut self) {
        let sys_count = self
            .history
            .iter()
            .filter(|m| m.role == "system")
            .count();
        if self.history.len() > MAX_HISTORY + sys_count {
            let all: Vec<Message> = self.history.drain(..).collect();
            let systems: Vec<Message> = all.iter().filter(|m| m.role == "system").cloned().collect();
            let non_systems: Vec<Message> = all.into_iter().filter(|m| m.role != "system").collect();
            let skip = non_systems.len().saturating_sub(MAX_HISTORY);
            let keep: Vec<Message> = non_systems.into_iter().skip(skip).collect();
            let mut combined = systems;
            combined.extend(keep);
            self.history = combined;
        }
    }
}
