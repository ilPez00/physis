use std::collections::HashMap;

use crate::models::{DomainDef, Goal};
use crate::trie::DynamicVectorTrie;

pub fn format_wiki(trie: &DynamicVectorTrie, goals: &[Goal], title: &str) -> String {
    let mut out = String::new();
    out.push_str(&format!("# {}\n\n", title));

    out.push_str("## Goals\n\n");
    for goal in goals {
        out.push_str(&format!("- [[{}]] — progress: {:.0}%\n", goal.name, goal.progress * 100.0));
    }

    out.push_str("\n## Ontology Tree\n\n");
    out.push_str("```mermaid\nmindmap\n  root((Physis))\n");
    for i in 1..trie.pool.len().min(4) {
        let token = trie.token_str(trie.pool[i].token_id);
        if !token.is_empty() {
            out.push_str(&format!("    {}\n", token));
        }
    }
    out.push_str("```\n\n");

    let s = trie.stats();
    out.push_str("## Stats\n\n");
    out.push_str(&format!("- Nodes: {}\n", s.get("nodes").unwrap_or(&0)));
    out.push_str(&format!("- Tokens: {}\n", s.get("tokens").unwrap_or(&0)));
    out.push_str(&format!("- Terminal nodes: {}\n", s.get("terminal_nodes").unwrap_or(&0)));
    out.push_str(&format!("- Max depth: {}\n", s.get("max_depth").unwrap_or(&0)));

    out
}

pub fn format_json_graph(trie: &DynamicVectorTrie) -> String {
    let json = trie.export_json();
    serde_json::to_string_pretty(&json).unwrap_or_default()
}

pub fn format_mermaid_mindmap(trie: &DynamicVectorTrie, title: &str) -> String {
    let mut out = String::new();
    out.push_str("mindmap\n");
    out.push_str(&format!("  root(({}))\n", title));

    fn add_children(
        pool: &[crate::trie::Node],
        dict: &[String],
        idx: usize,
        depth: usize,
        out: &mut String,
    ) {
        let node = &pool[idx];
        if depth > 0 && !dict[node.token_id as usize].is_empty() {
            let indent = "  ".repeat(depth + 1);
            out.push_str(&format!("{}{}\n", indent, dict[node.token_id as usize]));
        }
        let cs = node.child_start;
        let cc = node.child_count as usize;
        if cs >= 0 {
            for i in 0..cc {
                add_children(pool, dict, cs as usize + i, depth + 1, out);
            }
        }
    }

    add_children(&trie.pool, &trie.dictionary, 0, 0, &mut out);
    out
}

pub fn format_mermaid_flowchart(trie: &DynamicVectorTrie) -> String {
    let mut out = String::new();
    out.push_str("flowchart TD\n");

    for (i, node) in trie.pool.iter().enumerate() {
        let label = trie.token_str(node.token_id);
        if label.is_empty() && i == 0 {
            continue;
        }
        let escaped = label.replace('\"', "'");
        let display = if label.is_empty() { "root" } else { &escaped };

        let cs = node.child_start as usize;
        let cc = node.child_count as usize;
        for j in 0..cc {
            let child_idx = cs + j;
            let child_label = trie.token_str(trie.pool[child_idx].token_id);
            let child_escaped = child_label.replace('\"', "'");
            out.push_str(&format!("    n{}[{}] --> n{}[{}]\n", i, display, child_idx, child_escaped));
        }
    }

    out
}

pub fn format_domain_report(
    goals: &[Goal],
    domains: &HashMap<String, DomainDef>,
    domain_grades: &HashMap<String, Vec<f32>>,
) -> String {
    let mut out = String::new();
    out.push_str("# Domain Report\n\n");

    for (name, def) in domains {
        out.push_str(&format!("## {}\n\n", def.name));
        out.push_str(&format!("- Category: {}\n", def.category.as_deref().unwrap_or("none")));
        out.push_str(&format!("- Domain: {}\n", def.domain.as_str()));
        out.push_str(&format!("- Mode: {}\n", def.mode.as_str()));
        out.push_str(&format!("- Unit: {}\n", def.unit));

        if let Some(grades) = domain_grades.get(name) {
            let avg: f32 = grades.iter().sum::<f32>() / grades.len() as f32;
            out.push_str(&format!("- Avg grade: {:.2}\n", avg));
        }

        let domain_goals: Vec<&Goal> = goals.iter().filter(|g| g.domain_name == *name).collect();
        if !domain_goals.is_empty() {
            out.push_str("\nGoals:\n");
            for g in &domain_goals {
                out.push_str(&format!("  - {} ({:.0}%)\n", g.name, g.progress * 100.0));
            }
        }
        out.push('\n');
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PhysisConfig;
    use crate::config::OntologyLoader;

    #[test]
    fn test_format_wiki() {
        let trie = DynamicVectorTrie::new();
        let goals = vec![Goal::new("test/goal", "test")];
        let wiki = format_wiki(&trie, &goals, "Test Wiki");
        assert!(wiki.contains("Test Wiki"));
        assert!(wiki.contains("test/goal"));
    }

    #[test]
    fn test_format_mermaid() {
        let mut trie = DynamicVectorTrie::new();
        trie.insert_path("a/b/c");
        let mm = format_mermaid_mindmap(&trie, "Test");
        assert!(mm.contains("mindmap"));
    }

    #[test]
    fn test_domain_report() {
        let config = PhysisConfig::default();
        let loader = OntologyLoader::load_all(&config);
        let goals = vec![];
        let grades = HashMap::new();
        let report = format_domain_report(&goals, &loader.human_domains, &grades);
        assert!(report.contains("Domain Report"));
    }

    #[test]
    fn test_format_json_graph() {
        let mut trie = DynamicVectorTrie::new();
        trie.insert_path("a/b/c");
        let json_str = format_json_graph(&trie);
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        let entities = parsed["entities"].as_array().unwrap();
        let _relations = parsed["relations"].as_array().unwrap();
        assert!(!entities.is_empty(), "should have at least one entity");
        assert!(entities.len() >= 2, "should have at least 2 entities (b/c and c)");
    }

    #[test]
    fn test_format_json_graph_empty() {
        let trie = DynamicVectorTrie::new();
        let json_str = format_json_graph(&trie);
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert!(parsed["entities"].as_array().unwrap().is_empty());
        assert!(parsed["relations"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_format_mermaid_flowchart() {
        let mut trie = DynamicVectorTrie::new();
        trie.insert_path("a/b/c");
        let fc = format_mermaid_flowchart(&trie);
        assert!(fc.contains("flowchart"));
        assert!(fc.contains("-->"));
    }
}
