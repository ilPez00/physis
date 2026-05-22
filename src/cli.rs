use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::actor::PDCActor;
use crate::config::{OntologyLoader, PhysisConfig};
use crate::dream::DreamEngine;
use crate::mapper::OntologyMapper;
use crate::models::{Goal, Score};
use crate::network::NetworkScanner;
use crate::output;
use crate::trie::DynamicVectorTrie;

#[derive(Parser)]
#[command(name = "physis", version, about = "Ontological mapper and PDCA dream engine")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Scan a directory and build ontological map
    Scan {
        /// Directory to scan
        dir: PathBuf,
        /// Output format (wiki, json, mermaid)
        #[arg(short, long, default_value = "wiki")]
        format: String,
    },
    /// Query the ontology trie
    Query {
        /// Search query
        query: String,
        #[arg(short, long, default_value = "10")]
        max_results: usize,
    },
    /// Generate stochastic dreams
    Dream {
        /// Number of dreams to generate
        #[arg(short, long, default_value = "5")]
        count: usize,
    },
    /// Evaluate a dream
    Evaluate {
        /// Dream ID
        id: String,
        /// Grade (0.0 - 1.0)
        grade: Score,
    },
    /// Start MCP server
    #[cfg(feature = "mcp")]
    Serve,
    /// Watch directories for changes
    Watch {
        /// Directories to watch
        dirs: Vec<PathBuf>,
    },
    /// Show stats
    Stats,
    /// Print config
    Config,
}

pub struct PhysisApp {
    pub config: PhysisConfig,
    pub ontology: OntologyLoader,
    pub mapper: OntologyMapper,
    pub actor: PDCActor,
    pub dreams: DreamEngine,
    pub goals: Vec<Goal>,
}

impl PhysisApp {
    pub fn new(config: PhysisConfig) -> Self {
        let ontology = OntologyLoader::load_all(&config);
        let trie = DynamicVectorTrie::new();
        let mapper = OntologyMapper::new(ontology.clone());
        let actor = PDCActor::new(config.pdca_stagnant_threshold, config.pdca_stagnant_window);
        let dreams = DreamEngine::new(trie);

        Self {
            config,
            ontology,
            mapper,
            actor,
            dreams,
            goals: Vec::new(),
        }
    }

    pub fn run_scan(&mut self, dir: &std::path::Path, format: &str) -> String {
        let goals = self.mapper.map_filesystem(dir, None);
        self.goals = goals;
        self.dreams.trie = self.mapper.trie.clone();

        match format {
            "json" => output::format_json_graph(&self.mapper.trie),
            "mermaid" => output::format_mermaid_mindmap(&self.mapper.trie, "Physis Scan"),
            _ => output::format_wiki(&self.mapper.trie, &self.goals, "Physis Scan"),
        }
    }

    pub fn run_query(&self, query: &str, max_results: usize) -> Vec<Vec<String>> {
        let words: Vec<&str> = query.split_whitespace().collect();
        let tids: Vec<u32> = words
            .iter()
            .filter_map(|w| self.mapper.trie.token_id(w))
            .collect();
        if tids.is_empty() {
            return vec![];
        }
        self.mapper.trie.prefix_search(&tids, 2, max_results)
    }

    pub fn run_dream(&mut self, count: usize) -> Vec<crate::models::Dream> {
        self.dreams.generate_dreams(&self.goals, count)
    }

    pub fn run_evaluate(&mut self, id: &str, grade: Score) -> bool {
        self.dreams.evaluate_dream(id, grade)
    }

    pub fn run_watch(&mut self, dirs: Vec<PathBuf>) -> String {
        let cache_path = self.config.data_dir.join("hash_cache.json");
        let scanner = NetworkScanner::new(dirs.clone(), cache_path, self.config.network_scan_interval_secs);
        let diffs = scanner.scan_all();
        let mut changes = Vec::new();
        for diff in &diffs {
            changes.push(format!("{} new, {} changed, {} deleted",
                diff.summary.new, diff.summary.changed, diff.summary.deleted));
        }
        changes.join("\n")
    }

    pub fn run_stats(&self) -> String {
        let stats = self.mapper.stats();
        let mut out = String::new();
        out.push_str("=== Physis Stats ===\n");
        let mut pairs: Vec<_> = stats.into_iter().collect();
        pairs.sort_by(|a, b| a.0.cmp(&b.0));
        for (k, v) in pairs {
            out.push_str(&format!("  {k}: {v}\n"));
        }
        let actor_stats = self.actor.stats(&self.goals);
        out.push_str(&format!("  total_actions: {}\n", actor_stats.total_actions));
        out.push_str(&format!("  avg_grade: {:.3}\n", actor_stats.avg_grade));
        if !actor_stats.stagnant_goals.is_empty() {
            out.push_str("  stagnant_goals:\n");
            for g in &actor_stats.stagnant_goals {
                out.push_str(&format!("    - {g}\n"));
            }
        }
        out
    }
}
