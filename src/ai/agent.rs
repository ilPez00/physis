use crate::ai::provider::{Content, ContentPart, ImageUrl, Message};
use crate::ai::tools::{ToolRegistry, call_tool};
use crate::ai::{AiError, AiResult, provider::ProviderCascade};
use serde::{Deserialize, Serialize};

const MAX_TOOL_ROUNDS: u32 = 8;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub system_prompt: String,
    pub max_tokens: u32,
    pub detail: String,
}

#[derive(Debug, Clone)]
pub struct AgentOutput {
    pub text: String,
    pub provider: String,
    pub tool_rounds: u32,
    pub tokens_used: u32,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            system_prompt: String::new(),
            max_tokens: 1024,
            detail: "low".into(),
        }
    }
}

pub async fn run_agent(
    provider_cascade: &ProviderCascade,
    tools: &ToolRegistry,
    config: &AgentConfig,
    history: &[Message],
    query: &str,
    frame_b64: Option<&str>,
    mode: &str,
    model_override: Option<&str>,
) -> AiResult<AgentOutput> {
    let mut messages: Vec<Message> = Vec::new();

    messages.push(Message {
        role: "system".into(),
        content: Content::Text(config.system_prompt.clone()),
        tool_calls: None,
        tool_call_id: None,
        name: None,
    });

    let mode_prefix = match mode {
        "SCOUT" => "[SCOUT MODE - brief observation]\n",
        "ANALYZE" => "[ANALYZE MODE - detailed analysis]\n",
        "DATA" => "[DATA MODE - structured output]\n",
        _ => "",
    };

    messages.extend_from_slice(history);

    let user_content = if let Some(b64) = frame_b64 {
        Content::Parts(vec![
            ContentPart {
                type_: "text".into(),
                text: Some(format!("{mode_prefix}{query}")),
                image_url: None,
            },
            ContentPart {
                type_: "image_url".into(),
                text: None,
                image_url: Some(ImageUrl {
                    url: format!("data:image/jpeg;base64,{b64}"),
                    detail: Some(config.detail.clone()),
                }),
            },
        ])
    } else {
        Content::Text(format!("{mode_prefix}{query}"))
    };

    messages.push(Message {
        role: "user".into(),
        content: user_content,
        tool_calls: None,
        tool_call_id: None,
        name: None,
    });

    let route = provider_cascade.classify(query, frame_b64.is_some());
    let tool_defs = tools.definitions();
    let defs_for_call = if tool_defs.is_empty() {
        None
    } else {
        Some(tool_defs)
    };

    let mut tool_rounds = 0u32;
    let mut total_tokens = 0u32;

    loop {
        let (resp, provider_name) = if let Some(m) = model_override {
            // Find provider by model name
            let p = provider_cascade.providers_for_task(&route).into_iter()
                .find(|p| p.model() == m || p.name() == m)
                .ok_or_else(|| AiError::NoProvider)?;
            (p.complete(&messages, defs_for_call, Some(config.max_tokens)).await?, p.name().to_string())
        } else {
            provider_cascade
                .complete(&messages, defs_for_call, &route, Some(config.max_tokens))
                .await?
        };

        if let Some(u) = &resp.content {
            total_tokens += u.len() as u32;
        }

        messages.push(Message {
            role: "assistant".into(),
            content: Content::Text(resp.content.clone().unwrap_or_default()),
            tool_calls: resp.tool_calls.clone(),
            tool_call_id: None,
            name: None,
        });

        let calls = match resp.tool_calls {
            Some(c) if !c.is_empty() => c,
            _ => {
                let text = resp.content.unwrap_or_default();
                return Ok(AgentOutput {
                    text,
                    provider: provider_name,
                    tool_rounds,
                    tokens_used: total_tokens,
                });
            }
        };

        tool_rounds += 1;
        if tool_rounds > MAX_TOOL_ROUNDS {
            return Err(AiError::MaxToolRounds);
        }

        for call in &calls {
            let args: serde_json::Value =
                serde_json::from_str(&call.function.arguments).unwrap_or(serde_json::json!({}));
            let result = call_tool(tools, &call.function.name, args).await;

            messages.push(Message {
                role: "tool".into(),
                content: Content::Text(result),
                tool_calls: None,
                tool_call_id: Some(call.id.clone()),
                name: Some(call.function.name.clone()),
            });
        }
    }
}
