use chrono::Utc;
use std::collections::HashMap;

use crate::config::OntologyLoader;
use crate::models::{Experience, Goal, Score};

#[derive(Debug, Clone)]
pub struct PDCAStats {
    pub total_actions: usize,
    pub total_goals: usize,
    pub avg_grade: f64,
    pub domain_grades: HashMap<String, Vec<Score>>,
    pub stagnant_goals: Vec<String>,
}

#[derive(Debug)]
pub struct PDCActor {
    pub experiences: Vec<Experience>,
    pub config_stagnant_threshold: f32,
    pub config_stagnant_window: usize,
}

impl PDCActor {
    pub fn new(stagnant_threshold: f32, stagnant_window: usize) -> Self {
        Self {
            experiences: Vec::new(),
            config_stagnant_threshold: stagnant_threshold,
            config_stagnant_window: stagnant_window,
        }
    }

    pub fn plan<'a>(&self, goals: &'a [Goal]) -> Vec<&'a Goal> {
        let mut sorted: Vec<&Goal> = goals.iter().collect();
        sorted.sort_by(|a, b| {
            b.priority
                .unwrap_or(0)
                .cmp(&a.priority.unwrap_or(0))
                .then_with(|| a.progress.partial_cmp(&b.progress).unwrap_or(std::cmp::Ordering::Equal))
        });
        sorted.truncate(3);
        sorted
    }

    pub fn do_action(&mut self, goal_id: &str, action: &str, rationale: &str) -> Experience {
        let exp = Experience::new(goal_id, action, 0.0, rationale);
        self.experiences.push(exp.clone());
        exp
    }

    pub fn check(&mut self, experience_id: &str, grade: Score) -> bool {
        if let Some(exp) = self.experiences.iter_mut().find(|e| e.id == experience_id) {
            exp.grade = grade;
            true
        } else {
            false
        }
    }

    pub fn act(&self, experiences: &[Experience], goals: &mut [Goal], ontology: &OntologyLoader) -> Vec<String> {
        let mut adjustments = Vec::new();

        for goal in goals.iter_mut() {
            let relevant: Vec<&Experience> = experiences
                .iter()
                .filter(|e| e.goal_id == goal.id)
                .collect();

            if relevant.is_empty() {
                continue;
            }

            let avg_grade: Score = relevant.iter().map(|e| e.grade).sum::<Score>() / relevant.len() as Score;
            goal.progress = (goal.progress + avg_grade * 0.1).clamp(0.0, 1.0);
            goal.updated_at = Utc::now();

            let enriched = ontology.enrich_goal(goal);
            adjustments.push(enriched);
        }

        adjustments
    }

    pub fn stats(&self, goals: &[Goal]) -> PDCAStats {
        let mut domain_grades: HashMap<String, Vec<Score>> = HashMap::new();

        for exp in &self.experiences {
            let domain = goals
                .iter()
                .find(|g| g.id == exp.goal_id)
                .map(|g| g.domain_name.clone())
                .unwrap_or_else(|| "unknown".to_string());

            domain_grades.entry(domain).or_default().push(exp.grade);
        }

        let grades: Vec<Score> = self.experiences.iter().map(|e| e.grade).collect();
        let avg_grade = if grades.is_empty() {
            0.0
        } else {
            grades.iter().sum::<Score>() / grades.len() as Score
        };

        let stagnant_goals = self.detect_stagnant(goals);

        PDCAStats {
            total_actions: self.experiences.len(),
            total_goals: goals.len(),
            avg_grade: avg_grade as f64,
            domain_grades,
            stagnant_goals,
        }
    }

    fn detect_stagnant(&self, goals: &[Goal]) -> Vec<String> {
        let mut stagnant = Vec::new();
        let window = self.config_stagnant_window;
        let threshold = self.config_stagnant_threshold;

        for goal in goals {
            let goal_exps: Vec<&Experience> = self.experiences
                .iter()
                .filter(|e| e.goal_id == goal.id)
                .collect();

            if goal_exps.len() < window {
                continue;
            }

            let recent: Vec<Score> = goal_exps
                .iter()
                .rev()
                .take(window)
                .map(|e| e.grade)
                .collect();

            if recent.iter().all(|g| *g < threshold) {
                stagnant.push(goal.name.clone());
            }
        }

        stagnant
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PhysisConfig;

    #[test]
    fn test_pdca_cycle() {
        let mut actor = PDCActor::new(0.2, 3);
        let config = PhysisConfig::default();
        let ontology = OntologyLoader::load_all(&config);

        let mut goals = vec![
            Goal::new("exercise", "Body & Fitness"),
            Goal::new("read book", "Intellectual"),
        ];

        let planned = actor.plan(&goals);
        assert_eq!(planned.len(), 2);

        let exp = actor.do_action(&goals[0].id, "ran 5k", "daily run");
        assert!(actor.check(&exp.id, 0.8));

        let adjustments = actor.act(&actor.experiences, &mut goals, &ontology);
        assert!(!adjustments.is_empty());
    }

    #[test]
    fn test_stagnant_detection() {
        let mut actor = PDCActor::new(0.3, 3);
        let goal = Goal::new("stagnant_task", "Work");

        for _ in 0..5 {
            let exp = actor.do_action(&goal.id, "tried", "no progress");
            actor.check(&exp.id, 0.1);
        }

        let goals = vec![goal];
        let stagnant = actor.detect_stagnant(&goals);
        assert_eq!(stagnant.len(), 1);
    }
}
