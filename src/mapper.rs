use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::config::OntologyLoader;
use crate::models::Goal;
use crate::scanner::{self, FileInfo};
use crate::trie::DynamicVectorTrie;

#[derive(Debug, Clone)]
pub struct OntologyMapper {
    pub ontology: OntologyLoader,
    pub trie: DynamicVectorTrie,
}

impl OntologyMapper {
    pub fn new(ontology: OntologyLoader) -> Self {
        Self {
            ontology,
            trie: DynamicVectorTrie::new(),
        }
    }

    pub fn map_filesystem(
        &mut self,
        root_dir: &Path,
        extra_exclude: Option<&HashSet<String>>,
    ) -> Vec<Goal> {
        let files = scanner::scan_project(root_dir, extra_exclude);
        let mut goals = Vec::new();

        for file in &files {
            let path_str = file.structural_path();
            let tids = self.trie.tokenize_path_mut(&file.path);
            self.trie.insert(&tids);

            let domain_name = self.resolve_domain_for_file(file);
            let goal = Goal::new(&path_str, &domain_name);
            goals.push(goal);

            for line in &file.structural_lines {
                let line_path = format!("{} → {}", file.structural_path(), line);
                let line_tids = self.trie.tokenize_mut(&line_path);
                self.trie.insert(&line_tids);
            }
        }

        goals
    }

    pub fn map_goals_to_trie(&mut self, goals: &[Goal]) {
        for goal in goals {
            let tokens = self.trie.tokenize_mut(&goal.name);
            self.trie.insert(&tokens);
        }
    }

    fn resolve_domain_for_file(&self, file: &FileInfo) -> String {
        let ext_no_dot = file.ext.trim_start_matches('.');
        if ext_no_dot.is_empty() {
            return "file".to_string();
        }

        let lang_domain = match ext_no_dot {
            "rs" | "py" | "kt" | "java" | "ts" | "js" | "go" | "rb" | "swift" | "c" | "cpp" | "h" | "hpp" => "code",
            "md" | "txt" | "rst" | "adoc" => "documentation",
            "json" | "yaml" | "yml" | "toml" | "xml" => "config",
            "html" | "css" | "scss" | "less" => "web",
            "sh" | "bash" | "zsh" => "script",
            "sql" | "graphql" | "prisma" => "data",
            _ => "file",
        };

        let lower_path = file.path.to_lowercase();
        for (_name, def) in &self.ontology.machine_domains {
            if def.hints.iter().any(|h| lower_path.contains(h)) {
                return def.name.clone();
            }
        }

        lang_domain.to_string()
    }

    pub fn query(&self, query: &str) -> Vec<Vec<String>> {
        let words: Vec<&str> = query.split_whitespace().collect();
        let tids: Vec<u32> = words
            .iter()
            .filter_map(|w| self.trie.token_id(w))
            .collect();
        if tids.is_empty() {
            return vec![];
        }
        self.trie.prefix_search(&tids, 2, 10)
    }

    pub fn stats(&self) -> HashMap<String, usize> {
        let mut s = self.trie.stats();
        s.insert("human_domains".to_string(), self.ontology.human_domains.len());
        s.insert("machine_domains".to_string(), self.ontology.machine_domains.len());
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PhysisConfig;

    #[test]
    fn test_mapper_filesystem() {
        let config = PhysisConfig::default();
        let ontology = OntologyLoader::load_all(&config);
        let mut mapper = OntologyMapper::new(ontology);

        let dir = std::env::temp_dir().join("physis_mapper_test");
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(dir.join("test.rs"), "fn main() {}").unwrap();
        std::fs::write(dir.join("note.md"), "# Hello").unwrap();

        let goals = mapper.map_filesystem(&dir, None);
        assert!(!goals.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_query() {
        let config = PhysisConfig::default();
        let ontology = OntologyLoader::load_all(&config);
        let mut mapper = OntologyMapper::new(ontology);

        mapper.trie.insert_path("project/philosophy/identity");
        mapper.trie.insert_path("project/ai/memory");

        let results = mapper.query("project");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_map_empty_directory() {
        let config = PhysisConfig::default();
        let ontology = OntologyLoader::load_all(&config);
        let mut mapper = OntologyMapper::new(ontology);

        let dir = std::env::temp_dir().join("physis_mapper_empty_test");
        let _ = std::fs::create_dir_all(&dir);

        let goals = mapper.map_filesystem(&dir, None);
        assert!(goals.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_query_no_match() {
        let config = PhysisConfig::default();
        let ontology = OntologyLoader::load_all(&config);
        let mapper = OntologyMapper::new(ontology);

        let results = mapper.query("nonexistent_query");
        assert!(results.is_empty());
    }

    #[test]
    fn test_structural_lines_in_trie() {
        let config = PhysisConfig::default();
        let ontology = OntologyLoader::load_all(&config);
        let mut mapper = OntologyMapper::new(ontology);

        let dir = std::env::temp_dir().join("physis_mapper_struct_test");
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(dir.join("code.rs"), "fn hello() {}\nstruct Foo {}").unwrap();

        mapper.map_filesystem(&dir, None);
        let results = mapper.query("hello");
        assert!(!results.is_empty(), "structural line 'hello' should be in trie");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_map_goals_to_trie() {
        let config = PhysisConfig::default();
        let ontology = OntologyLoader::load_all(&config);
        let mut mapper = OntologyMapper::new(ontology);

        let goals = vec![
            Goal::new("project/feature", "code"),
            Goal::new("docs/readme", "documentation"),
        ];
        mapper.map_goals_to_trie(&goals);

        let results = mapper.query("project");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_stats_after_mapping() {
        let config = PhysisConfig::default();
        let ontology = OntologyLoader::load_all(&config);
        let mut mapper = OntologyMapper::new(ontology);

        mapper.trie.insert_path("test/stats/path");
        let s = mapper.stats();
        assert_eq!(*s.get("human_domains").unwrap(), 14);
        assert_eq!(*s.get("machine_domains").unwrap(), 53);
        assert!(*s.get("nodes").unwrap() > 1);
    }
}
