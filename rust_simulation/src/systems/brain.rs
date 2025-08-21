use crate::brain::{Brain, BrainAction};
use crate::components::{
    BrainComponent, Health, Inventory, Player, Position, Velocity, WantsToBuild, WantsToCraft,
    WantsToGather, WantsToStoreItem,
};
use crate::fov;
use crate::lib::{BrainResource, IsDay, Map};
use bevy_ecs::prelude::*;

/// A helper struct that provides spatial query functionality to the brain logic.
/// This acts as a bridge between the procedural brain code and the ECS map resource.
struct WorldView<'a> {
    map: &'a Map,
}

impl<'a> crate::brain::EntityFinder for WorldView<'a> {
    fn get_entities_at(&self, pos: &Position) -> Option<Vec<Entity>> {
        self.map.get_entities_at(pos.x, pos.y)
    }
}

/// The main system that drives the AI for each entity with a BrainComponent.
pub fn brain_system(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut BrainComponent,
        &Position,
        &Health,
        &Inventory,
        &Player,
    )>,
    brain_res: Res<BrainResource>,
    map: Res<Map>,
    is_day: Res<IsDay>,
    world: &mut World,
) {
    let brain = &brain_res.0;
    let world_view = WorldView { map: &map };

    for (entity, mut brain_comp, pos, health, inventory, player) in query.iter_mut() {
        // 1. Get visible tiles for the brain's decision making.
        let fov_radius = if is_day.0 { 8 } else { 4 };
        let visible_tiles = fov::field_of_view(pos, fov_radius, &map);

        // 2. Construct the high-level state for the Q-learning model.
        let high_level_state = brain.get_high_level_state(health, inventory, &brain_comp, is_day.0);

        // 3. Tick the brain to get a state update and a potential action.
        let result = brain.tick(
            &brain_comp,
            &world_view,
            world,
            entity,
            &high_level_state,
            &visible_tiles,
        );

        match result {
            Ok(Some((update, action))) => {
                // 4. Apply the state update from the brain to the component.
                brain_comp.current_goal = update.current_goal;
                brain_comp.goal_stack = update.goal_stack;
                brain_comp.current_path = update.current_path;
                brain_comp.goal_commitment_ticks = update.goal_commitment_ticks;
                brain_comp.prev_state = update.prev_state;
                brain_comp.prev_goal = update.prev_goal;

                // 5. Apply the chosen action by adding a component to the entity.
                apply_brain_action(&mut commands, entity, action);
            }
            Ok(None) => {
                // The brain decided to do nothing this tick.
            }
            Err(e) => {
                // Log the error. In a real game, you might want more robust handling.
                log::error!("Brain tick error for entity {:?}: {}", entity, e);
            }
        }
    }
}

/// Applies the action chosen by the brain to the entity using Commands.
fn apply_brain_action(commands: &mut Commands, entity: Entity, action: BrainAction) {
    match action {
        BrainAction::Move(vel) => {
            commands.entity(entity).insert(vel);
        }
        BrainAction::Gather(wants) => {
            commands.entity(entity).insert(wants);
        }
        BrainAction::Craft(wants) => {
            commands.entity(entity).insert(wants);
        }
        BrainAction::Build(wants) => {
            commands.entity(entity).insert(wants);
        }
        BrainAction::Attack(wants) => {
            commands.entity(entity).insert(wants);
        }
        BrainAction::Store(wants) => {
            commands.entity(entity).insert(wants);
        }
    }
}
