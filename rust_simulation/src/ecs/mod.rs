use std::any::{Any, TypeId};
use std::collections::HashMap;

pub type Entity = usize;

pub trait Component: 'static {}

pub struct World {
    pub entities: Vec<Entity>,
    next_entity_id: usize,
    components: HashMap<TypeId, Box<dyn Any + Send>>,
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

    pub fn add_component<T: Component + Send>(&mut self, entity: Entity, component: T) {
        let type_id = TypeId::of::<T>();
        let components = self.components
            .entry(type_id)
            .or_insert_with(|| Box::new(Vec::<Option<T>>::new()))
            .downcast_mut::<Vec<Option<T>>>()
            .unwrap();

        if entity >= components.len() {
            components.resize_with(entity + 1, || None);
        }
        components[entity] = Some(component);
    }

    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        if let Some(components) = self.components.get(&type_id) {
            if let Some(components) = components.downcast_ref::<Vec<Option<T>>>() {
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
            if let Some(components) = components.downcast_mut::<Vec<Option<T>>>() {
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
            if let Some(components) = components.downcast_mut::<Vec<Option<T>>>() {
                if entity < components.len() {
                    components[entity] = None;
                }
            }
        }
    }
}
