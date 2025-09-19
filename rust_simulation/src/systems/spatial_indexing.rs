use crate::{
    animals::pig::Pig,
    components::{Position, Resource as ResourceComponent},
    spatial::{SpatialIndex, SpatialPoint},
};
use bevy_ecs::prelude::*;
use rstar::RTree;

pub fn update_spatial_index_system(
    mut spatial_index: ResMut<SpatialIndex>,
    animal_query: Query<(Entity, &Position), With<Pig>>,
    resource_query: Query<(Entity, &Position), With<ResourceComponent>>,
) {
    spatial_index.animals = RTree::new();
    spatial_index.resources = RTree::new();

    for (entity, position) in animal_query.iter() {
        spatial_index.animals.insert(SpatialPoint {
            x: position.x as i32,
            y: position.y as i32,
            entity,
        });
    }

    for (entity, position) in resource_query.iter() {
        spatial_index.resources.insert(SpatialPoint {
            x: position.x as i32,
            y: position.y as i32,
            entity,
        });
    }
}
