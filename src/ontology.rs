use crate::models::*;

pub const HUMAN_ONTOLOGY_NAME: &str = "praxis_ontology";
pub const HUMAN_TYPE: &str = "human";

pub fn parse_ontology_entry(entry: &OntologyEntry) -> DomainDef {
    let domain = match entry.domain.to_uppercase().as_str() {
        "FABRICATE" => ActionDomain::Fabricate,
        "STUDY" => ActionDomain::Study,
        "CONSTRUCT" => ActionDomain::Construct,
        "BOND" => ActionDomain::Bond,
        "HEAL" => ActionDomain::Heal,
        _ => ActionDomain::Fabricate,
    };
    let mode = match entry.mode.to_uppercase().as_str() {
        "LIFT" => ActionMode::Lift,
        "REST" => ActionMode::Rest,
        "CREATE" => ActionMode::Create,
        "WALK" => ActionMode::Walk,
        "WORK" => ActionMode::Work,
        "LEARN" => ActionMode::Learn,
        _ => ActionMode::Work,
    };
    let axis_kind = if entry.axis_kind.eq_ignore_ascii_case("machine") {
        AxisKind::Machine
    } else {
        AxisKind::Human
    };
    let axis_human = if axis_kind == AxisKind::Human {
        match entry.axis_name.to_lowercase().as_str() {
            "physical" => Some(HumanScoreAxis::Physical),
            "economic" => Some(HumanScoreAxis::Economic),
            "intellectual" => Some(HumanScoreAxis::Intellectual),
            "psychological" => Some(HumanScoreAxis::Psychological),
            _ => None,
        }
    } else {
        None
    };
    let axis_machine = if axis_kind == AxisKind::Machine {
        match entry.axis_name.to_lowercase().as_str() {
            "operational" => Some(MachineScoreAxis::Operational),
            "structural" => Some(MachineScoreAxis::Structural),
            "informational" => Some(MachineScoreAxis::Informational),
            "energetic" => Some(MachineScoreAxis::Energetic),
            _ => None,
        }
    } else {
        None
    };
    DomainDef {
        name: entry.name.clone(),
        category: entry.category.clone(),
        domain,
        mode,
        axis_kind,
        axis_human,
        axis_machine,
        unit: entry.unit.clone(),
        hints: entry.hints.clone(),
    }
}
