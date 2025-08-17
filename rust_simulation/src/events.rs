use crate::ecs::Entity;

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    EntityDied(Entity),
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
