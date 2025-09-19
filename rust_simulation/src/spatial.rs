use bevy_ecs::prelude::*;
use rstar::{Point, RTree};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpatialPoint {
    pub x: i32,
    pub y: i32,
    pub entity: Entity,
}

impl Point for SpatialPoint {
    type Scalar = i32;
    const DIMENSIONS: usize = 2;

    fn generate(mut generator: impl FnMut(usize) -> Self::Scalar) -> Self {
        let mut array = [0; 2];
        for (i, value) in array.iter_mut().enumerate() {
            *value = generator(i);
        }
        SpatialPoint {
            x: array[0],
            y: array[1],
            entity: Entity::from_raw(0), // Dummy entity
        }
    }

    fn nth(&self, index: usize) -> Self::Scalar {
        match index {
            0 => self.x,
            1 => self.y,
            _ => unreachable!(),
        }
    }

    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => unreachable!(),
        }
    }
}

#[derive(Resource, Default)]
pub struct SpatialIndex {
    pub animals: RTree<SpatialPoint>,
    pub resources: RTree<SpatialPoint>,
}

impl SpatialIndex {
    pub fn new() -> Self {
        Self::default()
    }
}
