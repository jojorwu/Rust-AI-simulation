use std::any::{Any, TypeId};
use std::collections::HashMap;

pub type Entity = usize;

pub trait Component: 'static {}

pub trait ComponentVec {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn remove_component_for_entity(&mut self, entity: Entity);
}

impl<T: Component + Send> ComponentVec for Vec<Option<T>> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn remove_component_for_entity(&mut self, entity: Entity) {
        if entity < self.len() {
            self[entity] = None;
        }
    }
}

pub struct World {
    pub entities: Vec<Entity>,
    next_entity_id: usize,
    pub components: HashMap<TypeId, Box<dyn ComponentVec + Send>>,
}

impl World {
    pub fn new() -> Self {
        World {
            entities: Vec::new(),
            next_entity_id: 0,
            components: HashMap::new(),
        }
    }

    pub fn create_entity(&mut self) -> Entity {
        let entity_id = self.next_entity_id;
        self.entities.push(entity_id);
        self.next_entity_id += 1;
        entity_id
    }

    pub fn add_component<T: Component + Send>(
        &mut self,
        entity: Entity,
        component: T,
    ) -> Result<(), crate::errors::SimulationError> {
        let type_id = TypeId::of::<T>();
        let components = self
            .components
            .entry(type_id)
            .or_insert_with(|| Box::new(Vec::<Option<T>>::new()));

        let components = components
            .as_any_mut()
            .downcast_mut::<Vec<Option<T>>>()
            .ok_or_else(|| {
                crate::errors::SimulationError::UnwrapFailed(
                    "Failed to downcast component vector".to_string(),
                )
            })?;

        if entity >= components.len() {
            components.resize_with(entity + 1, || None);
        }
        components[entity] = Some(component);
        Ok(())
    }

    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        if let Some(components) = self.components.get(&type_id) {
            if let Some(components) = components.as_any().downcast_ref::<Vec<Option<T>>>() {
                if let Some(component) = components.get(entity) {
                    return component.as_ref();
                }
            }
        }
        None
    }

    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        if let Some(components) = self.components.get_mut(&type_id) {
            if let Some(components) = components.as_any_mut().downcast_mut::<Vec<Option<T>>>() {
                if let Some(component) = components.get_mut(entity) {
                    return component.as_mut();
                }
            }
        }
        None
    }

    pub fn remove_component<T: Component>(&mut self, entity: Entity) {
        let type_id = TypeId::of::<T>();
        if let Some(components) = self.components.get_mut(&type_id) {
            components.remove_component_for_entity(entity);
        }
    }

    pub fn remove_entity(&mut self, entity: Entity) {
        for (_type_id, components) in self.components.iter_mut() {
            components.remove_component_for_entity(entity);
        }
        self.entities.retain(|&e| e != entity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }
    impl Component for Position {}

    #[derive(Debug, PartialEq)]
    struct Velocity {
        dx: f32,
        dy: f32,
    }
    impl Component for Velocity {}

    #[test]
    fn test_remove_entity() -> Result<(), crate::errors::SimulationError> {
        let mut world = World::new();
        let entity1 = world.create_entity();
        world.add_component(entity1, Position { x: 1.0, y: 2.0 })?;
        world.add_component(entity1, Velocity { dx: 3.0, dy: 4.0 })?;

        let entity2 = world.create_entity();
        world.add_component(entity2, Position { x: 5.0, y: 6.0 })?;

        world.remove_entity(entity1);

        assert!(!world.entities.contains(&entity1));
        assert!(world.get_component::<Position>(entity1).is_none());
        assert!(world.get_component::<Velocity>(entity1).is_none());
        assert!(world.get_component::<Position>(entity2).is_some());
        Ok(())
    }
}
