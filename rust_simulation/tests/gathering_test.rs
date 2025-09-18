use bevy::prelude::*;
use rust_simulation::{
    components::{
        ai::KnownResources,
        intents::IsGathering,
        path::PathRequest,
        Inventory, Position, Resource as ResourceComponent,
    },
    map::Map,
    systems::gathering::gathering_system,
};
use std::collections::HashMap;

fn setup_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(
        Map::new(10, 10, "data/biomes.json", "data/resources.json")
            .expect("Failed to create map"),
    );
    app.add_systems(Update, gathering_system);
    app
}

#[test]
fn test_gathering_system_path_request() {
    let mut app = setup_test_app();

    let resource_pos = Position { x: 5, y: 5 };
    let resource_entity = app
        .world
        .spawn((
            ResourceComponent {
                name: "wood".to_string(),
                quantity: 10,
            },
            resource_pos,
        ))
        .id();
    app.world
        .resource_mut::<Map>()
        .add_entity_to_spatial_map(resource_entity, 5, 5);

    let gatherer_pos = Position { x: 0, y: 0 };
    let gatherer_entity = app
        .world
        .spawn((
            KnownResources(HashMap::new()),
            gatherer_pos,
            Inventory::new(),
            IsGathering {
                target: resource_entity,
                resource: "wood".to_string(),
                amount: 1,
            },
        ))
        .id();

    app.update();

    let gatherer = app.world.entity(gatherer_entity);
    assert!(gatherer.get::<PathRequest>().is_some());
    assert!(gatherer.get::<IsGathering>().is_none());
}
