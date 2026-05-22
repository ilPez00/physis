use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub type Score = f32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum AxisKind {
    Human,
    Machine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HumanScoreAxis {
    Physical,
    Economic,
    Intellectual,
    Psychological,
}

impl HumanScoreAxis {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Physical => "physical",
            Self::Economic => "economic",
            Self::Intellectual => "intellectual",
            Self::Psychological => "psychological",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MachineScoreAxis {
    Operational,
    Structural,
    Informational,
    Energetic,
}

impl MachineScoreAxis {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Operational => "operational",
            Self::Structural => "structural",
            Self::Informational => "informational",
            Self::Energetic => "energetic",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionDomain {
    Fabricate,
    Study,
    Construct,
    Bond,
    Heal,
}

impl ActionDomain {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Fabricate => "FABRICATE",
            Self::Study => "STUDY",
            Self::Construct => "CONSTRUCT",
            Self::Bond => "BOND",
            Self::Heal => "HEAL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionMode {
    Lift,
    Rest,
    Create,
    Walk,
    Work,
    Learn,
}

impl ActionMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Lift => "LIFT",
            Self::Rest => "REST",
            Self::Create => "CREATE",
            Self::Walk => "WALK",
            Self::Work => "WORK",
            Self::Learn => "LEARN",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DomainDef {
    pub name: String,
    pub category: Option<String>,
    pub domain: ActionDomain,
    pub mode: ActionMode,
    pub axis_kind: AxisKind,
    pub axis_human: Option<HumanScoreAxis>,
    pub axis_machine: Option<MachineScoreAxis>,
    pub unit: String,
    pub hints: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub name: String,
    pub domain_name: String,
    pub progress: Score,
    pub priority: Option<u32>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Goal {
    pub fn new(name: &str, domain_name: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            domain_name: domain_name.to_string(),
            progress: 0.0,
            priority: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersonalEntity {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub associations: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

impl PersonalEntity {
    pub fn new(name: &str, category: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            category: category.to_string(),
            description: String::new(),
            associations: vec![],
            metadata: HashMap::new(),
            first_seen: now,
            last_seen: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experience {
    pub id: String,
    pub goal_id: String,
    pub action: String,
    pub rationale: String,
    pub grade: Score,
    pub scores: HashMap<String, Score>,
    pub timestamp: DateTime<Utc>,
}

impl Experience {
    pub fn new(goal_id: &str, action: &str, grade: Score, rationale: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            goal_id: goal_id.to_string(),
            action: action.to_string(),
            rationale: rationale.to_string(),
            grade,
            scores: HashMap::new(),
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DreamType {
    Mutation,
    Graft,
    Prune,
    CrossPollination,
}

impl DreamType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Mutation => "MUTATION",
            Self::Graft => "GRAFT",
            Self::Prune => "PRUNE",
            Self::CrossPollination => "CROSS_POLLINATION",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dream {
    pub id: String,
    pub dream_type: DreamType,
    pub source_goal_id: String,
    pub source_path: Vec<String>,
    pub variation: Vec<String>,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub grade: Option<Score>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OntologyEntry {
    pub name: String,
    pub category: Option<String>,
    pub domain: String,
    pub mode: String,
    pub axis_kind: String,
    pub axis_name: String,
    pub unit: String,
    pub hints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OntologyConfig {
    pub kind: String,
    pub domains: Vec<OntologyEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub description: Option<String>,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub source: String,
    pub target: String,
    pub predicate: String,
    pub weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OntologicalMap {
    pub entities: HashMap<String, Entity>,
    pub relationships: Vec<Relationship>,
}

impl OntologicalMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.insert(entity.id.clone(), entity);
    }

    pub fn add_relationship(&mut self, rel: Relationship) {
        self.relationships.push(rel);
    }

    pub fn merge(&mut self, other: OntologicalMap) {
        for (id, entity) in other.entities {
            if let Some(existing) = self.entities.get_mut(&id) {
                for (k, v) in entity.attributes {
                    existing.attributes.insert(k, v);
                }
                if entity.description.is_some() {
                    existing.description = entity.description;
                }
            } else {
                self.entities.insert(id, entity);
            }
        }
        self.relationships.extend(other.relationships);
    }
}
