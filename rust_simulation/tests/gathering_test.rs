use bevy::prelude::*;
use rust_simulation::{
    components::{
        ai::KnownResources,
        intents::{IntendsToGather, IsGathering},
        Inventory, Position, Resource},
    events::Event,
    map::Map,
    systems::{
        find_resource::find_resource_system, gathering::gathering_system,
        resource_management::update_known_resources_system,
    },
};
use std::collections::{HashMap, HashSet};

#[test]
fn test_find_resource_system() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let map = Map::new(10, 10, "data/biomes.json", "data/resources.json")
        .expect("Failed to create map");
    let resource_pos = Position { x: 5, y: 5 };
    let resource_entity = app
        .world
        .spawn((
            Resource {
                name: "wood".to_string(),
                quantity: 10,
            },
            resource_pos,
        ))
        .id();
    map.add_entity_to_spatial_map(resource_entity, 5, 5);
    app.insert_resource(map);

    let mut known_resources = KnownResources(HashMap::new());
    let mut positions = HashSet::new();
    positions.insert(resource_pos);
    known_resources.0.insert("wood".to_string(), positions);
    let gatherer_entity = app
        .world
        .spawn((
            known_resources,
            Position { x: 0, y: 0 },
            IntendsToGather("wood".to_string(), 1),
        ))
        .id();

    app.add_systems(Update, find_resource_system);
    app.update();

    let gatherer = app.world.entity(gatherer_entity);
    let is_gathering = gatherer
        .get::<IsGathering>()
        .expect("Gatherer should have IsGathering component");
    assert_eq!(is_gathering.target, resource_entity);
    assert!(gatherer.get::<IntendsToGather>().is_none());
}

#[test]
fn test_gathering_depletes_resource_and_updates_known_resources() {
    // 1. Setup
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<Event>();
    app.add_systems(Update, (gathering_system, update_known_resources_system).chain());

    // Create a resource with 1 quantity
    let resource_pos = Position { x: 5, y: 5 };
    let resource_entity = app
        .world
        .spawn((
            Resource {
                name: "wood".to_string(),
                quantity: 1,
            },
            resource_pos,
        ))
        .id();

    // Create a function to spawn an agent with knowledge of the resource
    let mut create_agent = |pos: Position| {
        let mut known_resources = KnownResources(HashMap::new());
        let mut positions = HashSet::new();
        positions.insert(resource_pos);
        known_resources.0.insert("wood".to_string(), positions);
        app.world
            .spawn((known_resources, Inventory::new(), pos))
            .id()
    };

    // Create two agents
    let agent1_pos = Position { x: 5, y: 6 }; // Adjacent to resource
    let agent1 = create_agent(agent1_pos);
    let agent2 = create_agent(Position { x: 0, y: 0 });

    // Make agent1 gather the last resource
    app.world.entity_mut(agent1).insert(IsGathering {
        target: resource_entity,
        resource: "wood".to_string(),
        amount: 1,
    });

    // 2. Run the systems
    app.update();

    // 3. Verify
    // The resource entity should be despawned
    assert!(app.world.get_entity(resource_entity).is_none());

    // Agent1 should have 1 wood
    let agent1_inventory = app.world.get::<Inventory>(agent1).unwrap();
    assert_eq!(agent1_inventory.get_quantity("wood"), 1);

    // The depleted resource should be removed from Agent2's known resources
    let agent2_known = app.world.get::<KnownResources>(agent2).unwrap();
    let wood_positions = agent2_known.0.get("wood").unwrap();
    assert!(!wood_positions.contains(&resource_pos));
}
