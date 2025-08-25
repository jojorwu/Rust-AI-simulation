use crate::animals::wolf::{Wolf, WolfAI};
use crate::components::{Kills, Position};
use bevy::prelude::*;

const MAX_PACK_SIZE: usize = 3;
const LEADER_KILL_THRESHOLD: u32 = 2;
const PACK_JOIN_DISTANCE: f32 = 5.0;

#[derive(Component)]
pub struct Pack {
    pub members: Vec<Entity>,
    pub leader: Option<Entity>,
}

pub fn form_new_packs_system(
    mut commands: Commands,
    mut wolf_query: Query<(Entity, &mut WolfAI, &Position, &Kills), (With<Wolf>, Without<Pack>)>,
) {
    let mut lone_wolves: Vec<(Entity, Position, Kills)> = wolf_query
        .iter_mut()
        .map(|(entity, _, pos, kills)| (entity, *pos, *kills))
        .collect();

    let mut new_packs: Vec<(Entity, Vec<Entity>)> = Vec::new();

    let mut i = 0;
    while i < lone_wolves.len() {
        let mut j = i + 1;
        while j < lone_wolves.len() {
            let (wolf_a_entity, wolf_a_pos, wolf_a_kills) = lone_wolves[i];
            let (wolf_b_entity, wolf_b_pos, wolf_b_kills) = lone_wolves[j];

            if wolf_a_pos.distance(&wolf_b_pos) < PACK_JOIN_DISTANCE {
                let leader = if wolf_a_kills.0 >= LEADER_KILL_THRESHOLD {
                    wolf_a_entity
                } else if wolf_b_kills.0 >= LEADER_KILL_THRESHOLD {
                    wolf_b_entity
                } else {
                    wolf_a_entity // Default to the first wolf
                };

                let pack_entity = commands
                    .spawn(Pack {
                        members: vec![wolf_a_entity, wolf_b_entity],
                        leader: Some(leader),
                    })
                    .id();

                new_packs.push((pack_entity, vec![wolf_a_entity, wolf_b_entity]));

                lone_wolves.remove(j);
                lone_wolves.remove(i);
                i = 0;
                break;
            }
            j += 1;
        }
        i += 1;
    }

    for (pack_entity, members) in new_packs {
        for member_entity in members {
            if let Ok((_, mut wolf_ai, _, _)) = wolf_query.get_mut(member_entity) {
                wolf_ai.pack = Some(pack_entity);
            }
        }
    }
}

pub fn join_existing_packs_system(
    mut lone_wolf_query: Query<(Entity, &mut WolfAI, &Position), (With<Wolf>, Without<Pack>)>,
    mut pack_query: Query<(Entity, &mut Pack)>,
    wolf_position_query: Query<&Position, With<Wolf>>,
) {
    for (lone_wolf_entity, mut lone_wolf_ai, lone_wolf_pos) in lone_wolf_query.iter_mut() {
        for (pack_entity, mut pack) in pack_query.iter_mut() {
            if pack.members.len() < MAX_PACK_SIZE {
                if let Some(leader_entity) = pack.leader {
                    if let Ok(leader_pos) = wolf_position_query.get(leader_entity) {
                        if lone_wolf_pos.distance(leader_pos) < PACK_JOIN_DISTANCE {
                            pack.members.push(lone_wolf_entity);
                            lone_wolf_ai.pack = Some(pack_entity);
                            break;
                        }
                    }
                }
            }
        }
    }
}
