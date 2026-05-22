use chrono::Utc;
use rand::Rng;

use crate::models::{Dream, DreamType, Goal, Score};
use crate::trie::DynamicVectorTrie;

const MUTATION_RATE: f64 = 0.4;
const GRAFT_RATE: f64 = 0.25;
const PRUNE_RATE: f64 = 0.2;

#[derive(Debug, Clone)]
pub struct DreamEngine {
    pub trie: DynamicVectorTrie,
    pub dreams: Vec<Dream>,
    rng: rand::rngs::ThreadRng,
}

impl DreamEngine {
    pub fn new(trie: DynamicVectorTrie) -> Self {
        Self {
            trie,
            dreams: Vec::new(),
            rng: rand::thread_rng(),
        }
    }

    pub fn generate_dreams(&mut self, goals: &[Goal], batch_size: usize) -> Vec<Dream> {
        if goals.is_empty() {
            return vec![];
        }
        let mut dreams = Vec::new();
        let count = batch_size.min(goals.len());

        for _ in 0..count {
            let roll: f64 = self.rng.gen();
            let dream = if roll < MUTATION_RATE {
                self.mutate(goals)
            } else if roll < MUTATION_RATE + GRAFT_RATE {
                self.graft(goals)
            } else if roll < MUTATION_RATE + GRAFT_RATE + PRUNE_RATE {
                self.prune(goals)
            } else {
                self.cross_pollinate(goals)
            };
            dreams.push(dream);
        }

        self.dreams.extend(dreams.clone());
        dreams
    }

    fn mutate(&mut self, goals: &[Goal]) -> Dream {
        let goal = &goals[self.rng.gen_range(0..goals.len())];
        let path_tokens = self.trie.tokenize_mut(&goal.name);
        let path_strs: Vec<String> = path_tokens
            .iter()
            .map(|t| self.trie.token_str(*t).to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let variation = if path_strs.is_empty() {
            vec!["mutated_idea".to_string()]
        } else {
            let mut v = path_strs.clone();
            if !v.is_empty() {
                let idx = self.rng.gen_range(0..v.len());
                let suffixes = ["v2", "alt", "variant", "experimental", "next"];
                v[idx] = format!("{}_{}", v[idx], suffixes[self.rng.gen_range(0..suffixes.len())]);
            }
            v
        };

        Dream {
            id: uuid::Uuid::new_v4().to_string(),
            dream_type: DreamType::Mutation,
            source_goal_id: goal.id.clone(),
            source_path: path_strs,
            variation,
            description: "Stochastic mutation of existing goal path".to_string(),
            created_at: Utc::now(),
            grade: None,
        }
    }

    fn graft(&mut self, goals: &[Goal]) -> Dream {
        let goal = &goals[self.rng.gen_range(0..goals.len())];
        let path_tokens = self.trie.tokenize_mut(&goal.name);
        let path_strs: Vec<String> = path_tokens
            .iter()
            .map(|t| self.trie.token_str(*t).to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let grafts = [
            "analysis", "review", "optimization", "integration",
            "monitoring", "automation", "scaling", "testing",
        ];
        let graft_word = grafts[self.rng.gen_range(0..grafts.len())];

        let mut variation = path_strs.clone();
        variation.push(graft_word.to_string());

        Dream {
            id: uuid::Uuid::new_v4().to_string(),
            dream_type: DreamType::Graft,
            source_goal_id: goal.id.clone(),
            source_path: path_strs,
            variation,
            description: format!("Grafted domain with new branch: {graft_word}"),
            created_at: Utc::now(),
            grade: None,
        }
    }

    fn prune(&mut self, goals: &[Goal]) -> Dream {
        let goal = &goals[self.rng.gen_range(0..goals.len())];
        let path_tokens = self.trie.tokenize_mut(&goal.name);
        let path_strs: Vec<String> = path_tokens
            .iter()
            .map(|t| self.trie.token_str(*t).to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let variation = if path_strs.len() <= 1 {
            vec!["simplified".to_string()]
        } else {
            let keep = self.rng.gen_range(1..=path_strs.len().saturating_sub(1));
            path_strs[..keep].to_vec()
        };

        Dream {
            id: uuid::Uuid::new_v4().to_string(),
            dream_type: DreamType::Prune,
            source_goal_id: goal.id.clone(),
            source_path: path_strs,
            variation,
            description: "Pruned to simpler form".to_string(),
            created_at: Utc::now(),
            grade: None,
        }
    }

    fn cross_pollinate(&mut self, goals: &[Goal]) -> Dream {
        let g1 = &goals[self.rng.gen_range(0..goals.len())];
        let g2 = &goals[self.rng.gen_range(0..goals.len())];

        let t1: Vec<String> = self.trie
            .tokenize_mut(&g1.name)
            .iter()
            .map(|t| self.trie.token_str(*t).to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let t2: Vec<String> = self.trie
            .tokenize_mut(&g2.name)
            .iter()
            .map(|t| self.trie.token_str(*t).to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let split1 = if t1.len() > 1 {
            self.rng.gen_range(1..t1.len())
        } else {
            0
        };
        let split2 = if t2.len() > 1 {
            self.rng.gen_range(1..t2.len())
        } else {
            0
        };

        let mut variation: Vec<String> = t1[..split1].to_vec();
        variation.extend_from_slice(&t2[split2..]);

        if variation.is_empty() {
            variation = vec!["cross_idea".to_string()];
        }

        let mut source_path = t1.clone();
        source_path.extend(t2.clone());

        Dream {
            id: uuid::Uuid::new_v4().to_string(),
            dream_type: DreamType::CrossPollination,
            source_goal_id: g1.id.clone(),
            source_path,
            variation,
            description: format!("Cross-pollinated '{}' and '{}'", g1.name, g2.name),
            created_at: Utc::now(),
            grade: None,
        }
    }

    pub fn evaluate_dream(&mut self, dream_id: &str, grade: Score) -> bool {
        if let Some(dream) = self.dreams.iter_mut().find(|d| d.id == dream_id) {
            dream.grade = Some(grade);
            let variation_tids: Vec<u32> = dream
                .variation
                .iter()
                .flat_map(|s| self.trie.tokenize_mut(s))
                .collect();

            if grade >= 0.6 {
                self.trie.insert(&variation_tids);
                true
            } else {
                self.trie.delete(&variation_tids);
                false
            }
        } else {
            false
        }
    }

    pub fn ungraded_dreams(&self) -> Vec<&Dream> {
        self.dreams.iter().filter(|d| d.grade.is_none()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Goal;

    #[test]
    fn test_generate_dreams() {
        let trie = DynamicVectorTrie::new();
        let mut engine = DreamEngine::new(trie);

        let goals = vec![
            Goal::new("project/philosophy/identity", "philosophy"),
            Goal::new("project/ai/memory-compression", "ai"),
            Goal::new("project/ai/vector-trie", "ai"),
        ];

        let dreams = engine.generate_dreams(&goals, 10);
        assert!(!dreams.is_empty());
    }

    #[test]
    fn test_evaluate_dream_accept() {
        let mut trie = DynamicVectorTrie::new();
        trie.insert_path("test/goal");
        let mut engine = DreamEngine::new(trie);

        let goals = vec![Goal::new("test/goal", "test")];
        let dreams = engine.generate_dreams(&goals, 1);
        let dream_id = dreams[0].id.clone();

        assert!(engine.evaluate_dream(&dream_id, 0.8));
    }

    #[test]
    fn test_evaluate_dream_reject() {
        let mut trie = DynamicVectorTrie::new();
        trie.insert_path("test/goal");
        let mut engine = DreamEngine::new(trie);

        let goals = vec![Goal::new("test/goal", "test")];
        let dreams = engine.generate_dreams(&goals, 1);
        let dream_id = dreams[0].id.clone();

        assert!(!engine.evaluate_dream(&dream_id, 0.3));
    }

    #[test]
    fn test_dream_nonexistent_id() {
        let trie = DynamicVectorTrie::new();
        let mut engine = DreamEngine::new(trie);
        assert!(!engine.evaluate_dream("nonexistent-id", 0.5));
    }

    #[test]
    fn test_generate_no_goals() {
        let trie = DynamicVectorTrie::new();
        let mut engine = DreamEngine::new(trie);
        let dreams = engine.generate_dreams(&[], 5);
        assert!(dreams.is_empty());
    }

    #[test]
    fn test_dream_all_types() {
        let mut trie = DynamicVectorTrie::new();
        trie.insert_path("project/feature/alpha");
        let mut engine = DreamEngine::new(trie);

        let goals = vec![
            Goal::new("project/feature/alpha", "code"),
            Goal::new("project/feature/beta", "code"),
            Goal::new("docs/guide", "docs"),
            Goal::new("test/suite", "testing"),
            Goal::new("build/config", "config"),
        ];

        let dreams = engine.generate_dreams(&goals, 20);
        let types: std::collections::HashSet<String> = dreams
            .iter()
            .map(|d| d.dream_type.as_str().to_string())
            .collect();
        assert!(types.len() >= 3, "expected at least 3 different dream types, got {:?}", types);
    }

    #[test]
    fn test_ungraded_dreams() {
        let trie = DynamicVectorTrie::new();
        let mut engine = DreamEngine::new(trie);

        let goals = vec![Goal::new("test", "test")];
        engine.generate_dreams(&goals, 5);
        let ungraded = engine.ungraded_dreams();
        assert_eq!(ungraded.len(), 5);
    }

    #[test]
    fn test_graded_dreams_removed_from_ungraded() {
        let mut trie = DynamicVectorTrie::new();
        trie.insert_path("test/goal");
        let mut engine = DreamEngine::new(trie);

        let goals = vec![Goal::new("test/goal", "test")];
        let dreams = engine.generate_dreams(&goals, 3);
        engine.evaluate_dream(&dreams[0].id, 0.8);

        let ungraded = engine.ungraded_dreams();
        assert_eq!(ungraded.len(), 2);
    }
}
