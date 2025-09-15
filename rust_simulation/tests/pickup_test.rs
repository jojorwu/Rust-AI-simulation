use bevy::prelude::*;
use rust_simulation::{
    components::{DroppedItem, IsPickingUp, PickupClaimed, Position, WantsToPickup},
    map::Map,
    systems::pickup::claim_item_system,
};

#[test]
fn test_item_claiming_handles_race_condition() {
    // 1. Setup
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Create a map and add the claim system
    let map = Map::new(10, 10, "data/biomes.json", "data/resources.json").unwrap();
    app.insert_resource(map);
    app.add_systems(Update, claim_item_system);

    // Create one item at a specific position
    let item_pos = Position { x: 5, y: 5 };
    let item_entity = app
        .world
        .spawn((
            item_pos,
            DroppedItem {
                item_name: "wood".to_string(),
                quantity: 1,
            },
        ))
        .id();

    // Add the item to the map's spatial index
    let mut map = app.world.resource_mut::<Map>();
    map.add_entity_to_spatial_map(item_entity, item_pos.x, item_pos.y);

    // Create two agents at the same position, both wanting to pick up an item
    let agent1 = app.world.spawn((item_pos, WantsToPickup {})).id();
    let agent2 = app.world.spawn((item_pos, WantsToPickup {})).id();

    // 2. Run the system
    app.update();

    // 3. Verify
    // Check which agent successfully claimed the item
    let agent1_claimed = app.world.entity(agent1).get::<IsPickingUp>().is_some();
    let agent2_claimed = app.world.entity(agent2).get::<IsPickingUp>().is_some();

    // Assert that exactly one of them claimed the item
    assert!(
        agent1_claimed ^ agent2_claimed,
        "Exactly one agent should have claimed the item"
    );

    // Assert that the item itself is marked as claimed
    assert!(
        app.world.entity(item_entity).get::<PickupClaimed>().is_some(),
        "Item should be marked as claimed"
    );

    // Verify the IsPickingUp component points to the correct item
    if agent1_claimed {
        assert_eq!(app.world.get::<IsPickingUp>(agent1).unwrap().item, item_entity);
    } else {
        assert_eq!(app.world.get::<IsPickingUp>(agent2).unwrap().item, item_entity);
    }
}
