use crate::ecs::Entity;
use crate::components::Position;

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    EntityDied(Entity),
    FoundationBuilt { builder: Entity, position: Position },
}

#[derive(Default)]
pub struct EventBus {
    events: Vec<Event>,
}

impl EventBus {
    pub fn new() -> Self {
        EventBus { events: Vec::new() }
    }

    pub fn publish(&mut self, event: Event) {
        self.events.push(event);
    }

    pub fn take_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }
}
