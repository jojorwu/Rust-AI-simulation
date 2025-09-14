use crate::{
    brain::Goal,
    components::ai::KnownResources,
    map::Map,
};

/// Checks if a goal is currently valid based on the agent's knowledge and the map state.
pub fn is_goal_valid(goal: &Goal, known_resources: &KnownResources, map: &Map) -> bool {
    match goal {
        Goal::GatherResource(resource_name, _amount) => {
            // A resource is valid to gather if it's a huntable animal (which can be anywhere)
            // or if the agent knows the location of a static resource node.
            if let Some(resource_def) = map.resources.iter().find(|r| r.name == *resource_name) {
                if resource_def.huntable {
                    return true;
                }
            }
            known_resources
                .0
                .get(resource_name)
                .is_some_and(|s| !s.is_empty())
        }
        // For now, all other goals are considered valid if they are in the agent's goal list.
        // More complex validation could be added here (e.g., check if a build site is valid).
        _ => true,
    }
}
