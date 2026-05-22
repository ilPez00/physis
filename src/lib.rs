pub mod linguistic;

pub mod ai;
pub mod actor;
pub mod config;
pub mod dream;
pub mod mapper;
pub mod models;
pub mod ontology;
pub mod ontology_nonhuman;
pub mod output;
pub mod scanner;
pub mod trie;

pub mod network;
pub mod mcp;

#[cfg(feature = "cli")]
pub mod cli;

pub use actor::PDCActor;
pub use config::{OntologyLoader, PhysisConfig};
pub use dream::DreamEngine;
pub use mapper::OntologyMapper;
pub use models::*;
pub use output::*;
pub use scanner::*;
pub use trie::DynamicVectorTrie;

#[cfg(feature = "cli")]
pub use cli::PhysisApp;

pub use linguistic::*;
