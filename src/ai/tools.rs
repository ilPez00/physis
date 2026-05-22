use crate::ai::AiResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type ToolHandler = Box<dyn Fn(serde_json::Value) -> futures::future::BoxFuture<'static, AiResult<String>> + Send + Sync>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    pub type_: String,
    pub function: ToolFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

pub struct ToolRegistry {
    handlers: HashMap<String, ToolHandler>,
    definitions: Vec<ToolDefinition>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut reg = Self {
            handlers: HashMap::new(),
            definitions: Vec::new(),
        };
        reg.register_builtins();
        reg
    }

    pub fn register(
        &mut self,
        name: &str,
        description: &str,
        parameters: serde_json::Value,
        handler: ToolHandler,
    ) {
        self.definitions.push(ToolDefinition {
            type_: "function".into(),
            function: ToolFunction {
                name: name.to_string(),
                description: description.to_string(),
                parameters,
            },
        });
        self.handlers.insert(name.to_string(), handler);
    }

    pub fn definitions(&self) -> &[ToolDefinition] {
        &self.definitions
    }

    pub fn has(&self, name: &str) -> bool {
        self.handlers.contains_key(name)
    }

    pub fn names(&self) -> Vec<String> {
        self.handlers.keys().cloned().collect()
    }

    pub async fn call(&self, name: &str, args: serde_json::Value) -> AiResult<String> {
        match self.handlers.get(name) {
            Some(handler) => handler(args).await,
            None => Err(crate::ai::AiError::Tool(format!("unknown tool: {name}"))),
        }
    }

    fn register_builtins(&mut self) {
        self.register(
            "web_search",
            "Search the web for current information. Returns text results with URLs.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "Search query"}
                },
                "required": ["query"]
            }),
            Box::new(|args| {
                Box::pin(async move {
                    let query = args.get("query")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    tools_web_search(&query).await
                })
            }),
        );

        self.register(
            "read_url",
            "Fetch and read the content of a URL. Returns text/markdown content.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "url": {"type": "string", "description": "URL to read"}
                },
                "required": ["url"]
            }),
            Box::new(|args| {
                Box::pin(async move {
                    let url = args.get("url")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    tools_read_url(&url).await
                })
            }),
        );

        self.register(
            "read_file",
            "Read the contents of a file on the local filesystem.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Absolute path to file"}
                },
                "required": ["path"]
            }),
            Box::new(|args| {
                Box::pin(async move {
                    let path = args.get("path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    tools_read_file(&path)
                })
            }),
        );

        self.register(
            "write_file",
            "Write content to a file on the local filesystem. Creates directories if needed.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Absolute path to file"},
                    "content": {"type": "string", "description": "Content to write"}
                },
                "required": ["path", "content"]
            }),
            Box::new(|args| {
                Box::pin(async move {
                    let path = args.get("path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let content = args.get("content")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    tools_write_file(&path, &content)
                })
            }),
        );

        self.register(
            "list_dir",
            "List files and directories in a given path.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Directory path to list"}
                },
                "required": ["path"]
            }),
            Box::new(|args| {
                Box::pin(async move {
                    let path = args.get("path")
                        .and_then(|v| v.as_str())
                        .unwrap_or(".")
                        .to_string();
                    tools_list_dir(&path)
                })
            }),
        );

        self.register(
            "memory_save",
            "Save a piece of information to long-term memory for later recall.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "text": {"type": "string", "description": "Information to remember"},
                    "kind": {"type": "string", "description": "Category: observation|fact|preference", "default": "observation"}
                },
                "required": ["text"]
            }),
            Box::new(|args| {
                Box::pin(async move {
                    let text = args.get("text")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let kind = args.get("kind")
                        .and_then(|v| v.as_str())
                        .unwrap_or("observation")
                        .to_string();
                    Ok(format!("saved to memory [{kind}]: {text}"))
                })
            }),
        );

        self.register(
            "memory_search",
            "Search long-term memory for stored information by topic or keyword.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "Search query"}
                },
                "required": ["query"]
            }),
            Box::new(|args| {
                Box::pin(async move {
                    let query = args.get("query")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    Ok(format!("[memory search stub for: {query}]"))
                })
            }),
        );

        self.register(
            "get_timestamp",
            "Get the current date and time.",
            serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
            Box::new(|_args| {
                Box::pin(async move {
                    let now = chrono::Local::now();
                    Ok(now.format("%Y-%m-%d %H:%M:%S").to_string())
                })
            }),
        );
    }
}

async fn tools_web_search(query: &str) -> AiResult<String> {
    let client = reqwest::Client::new();
    let url = format!("https://api.duckduckgo.com/?q={q}&format=json&no_html=1", q = urlencoding(query));
    let resp = client.get(&url).send().await?;
    let text = resp.text().await?;
    let truncated: String = text.chars().take(3000).collect();
    Ok(truncated)
}

async fn tools_read_url(url: &str) -> AiResult<String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .unwrap_or_default();
    let resp = client.get(url)
        .header("User-Agent", "Mozilla/5.0 Aura/0.1")
        .send()
        .await?;
    let text = resp.text().await?;
    let truncated: String = text.chars().take(8000).collect();
    Ok(truncated)
}

fn tools_read_file(path: &str) -> AiResult<String> {
    use std::io::Read;
    let mut f = std::fs::File::open(path)?;
    let mut buf = String::new();
    f.read_to_string(&mut buf)?;
    let truncated: String = buf.chars().take(64000).collect();
    Ok(truncated)
}

fn tools_write_file(path: &str, content: &str) -> AiResult<String> {
    if let Some(parent) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content)?;
    Ok(format!("wrote {} bytes to {path}", content.len()))
}

fn tools_list_dir(path: &str) -> AiResult<String> {
    let entries = std::fs::read_dir(path)?;
    let mut lines = Vec::new();
    for entry in entries {
        let entry = entry?;
        let meta = entry.metadata()?;
        let kind = if meta.is_dir() { "d" } else { "-" };
        let size = meta.len();
        let name = entry.file_name().to_string_lossy().to_string();
        lines.push(format!("{kind} {size:>8} {name}"));
    }
    Ok(lines.join("\n"))
}

fn urlencoding(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "+".to_string(),
            other => format!("%{:02X}", other as u8),
        })
        .collect()
}

pub async fn call_tool(registry: &ToolRegistry, name: &str, args: serde_json::Value) -> String {
    match registry.call(name, args).await {
        Ok(result) => result,
        Err(e) => format!("error: {e}"),
    }
}
