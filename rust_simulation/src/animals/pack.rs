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

pub fn pack_system(
    mut commands: Commands,
    mut wolf_query: Query<(Entity, &mut WolfAI, &Position, &Kills), With<Wolf>>,
    mut pack_query: Query<(Entity, &mut Pack)>,
) {
    let mut wolves: Vec<_> = wolf_query.iter_mut().map(|(e, ai, p, k)| (e, ai.pack, *p, k.0)).collect();

    for i in 0..wolves.len() {
        for j in (i + 1)..wolves.len() {
            let (wolf_a_entity, wolf_a_pack, wolf_a_pos, wolf_a_kills) = wolves[i];
            let (wolf_b_entity, wolf_b_pack, wolf_b_pos, wolf_b_kills) = wolves[j];

            if wolf_a_pack.is_none() && wolf_b_pack.is_none() {
                // Two lone wolves, form a new pack
                if wolf_a_pos.distance(&wolf_b_pos) < PACK_JOIN_DISTANCE {
                    let leader = if wolf_a_kills >= LEADER_KILL_THRESHOLD {
                        wolf_a_entity
                    } else if wolf_b_kills >= LEADER_KILL_THRESHOLD {
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

                    if let Ok((_, mut wolf_a_ai, _, _)) = wolf_query.get_mut(wolf_a_entity) {
                        wolf_a_ai.pack = Some(pack_entity);
                    }
                    if let Ok((_, mut wolf_b_ai, _, _)) = wolf_query.get_mut(wolf_b_entity) {
                        wolf_b_ai.pack = Some(pack_entity);
                    }
                }
            } else if let Some(pack_entity) = wolf_a_pack {
                // Wolf A is in a pack, check if Wolf B can join
                if wolf_b_pack.is_none() {
                    if let Ok((_, mut pack)) = pack_query.get_mut(pack_entity) {
                        if pack.members.len() < MAX_PACK_SIZE {
                            if wolf_a_pos.distance(&wolf_b_pos) < PACK_JOIN_DISTANCE {
                                pack.members.push(wolf_b_entity);
                                if let Ok((_, mut wolf_b_ai, _, _)) = wolf_query.get_mut(wolf_b_entity) {
                                    wolf_b_ai.pack = Some(pack_entity);
                                }
                            }
                        }
                    }
                }
            } else if let Some(pack_entity) = wolf_b_pack {
                // Wolf B is in a pack, check if Wolf A can join
                if wolf_a_pack.is_none() {
                    if let Ok((_, mut pack)) = pack_query.get_mut(pack_entity) {
                        if pack.members.len() < MAX_PACK_SIZE {
                            if wolf_a_pos.distance(&wolf_a_pos) < PACK_JOIN_DISTANCE {
                                pack.members.push(wolf_a_entity);
                                if let Ok((_, mut wolf_a_ai, _, _)) = wolf_query.get_mut(wolf_a_entity) {
                                    wolf_a_ai.pack = Some(pack_entity);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
