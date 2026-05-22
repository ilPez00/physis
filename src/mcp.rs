#[cfg(feature = "mcp")]
mod mcp_impl {
    use std::sync::Arc;
    use tokio::sync::Mutex;

    use crate::actor::PDCActor;
    use crate::config::{OntologyLoader, PhysisConfig};
    use crate::dream::DreamEngine;
    use crate::mapper::OntologyMapper;
    use crate::models::Goal;

    pub struct PhysisContext {
        pub mapper: OntologyMapper,
        pub actor: PDCActor,
        pub dreams: DreamEngine,
        pub goals: Vec<Goal>,
    }

    pub type SharedContext = Arc<Mutex<PhysisContext>>;

    pub async fn start_mcp_server(config: PhysisConfig) -> anyhow::Result<()> {
        let ontology = OntologyLoader::load_all(&config);
        let mapper = OntologyMapper::new(ontology);
        let actor = PDCActor::new(config.pdca_stagnant_threshold, config.pdca_stagnant_window);
        let dreams = DreamEngine::new(mapper.trie.clone());

        let ctx = Arc::new(Mutex::new(PhysisContext {
            mapper,
            actor,
            dreams,
            goals: Vec::new(),
        }));

        let server = rmcp::Service::new(ctx);

        println!("Physis MCP server starting...");
        server.serve("127.0.0.1:9876").await?;

        Ok(())
    }
}

#[cfg(feature = "mcp")]
pub use mcp_impl::*;

pub struct McpClient {
    #[allow(dead_code)]
    base_url: String,
}

impl McpClient {
    pub fn new(url: &str) -> Self {
        Self {
            base_url: url.to_string(),
        }
    }

    pub async fn send_command(&self, _command: &str, _args: &[(&str, &str)]) -> anyhow::Result<String> {
        anyhow::bail!("MCP client not yet implemented; use the CLI instead")
    }
}
