use crate::ecs::World;
use crate::components::{Position, Velocity};

pub fn movement_system(world: &mut World) {
    for entity in 0..world.entities.len() {
        let (dx, dy) = if let Some(vel) = world.get_component::<Velocity>(entity) {
            (vel.dx, vel.dy)
        } else {
            (0, 0)
        };

        if dx != 0 || dy != 0 {
            if let Some(pos) = world.get_component_mut::<Position>(entity) {
                pos.x = (pos.x as i32 + dx) as u32;
                pos.y = (pos.y as i32 + dy) as u32;
            }
        }
    }

    // Reset velocities
    for entity in 0..world.entities.len() {
        world.remove_component::<Velocity>(entity);
    }
}
