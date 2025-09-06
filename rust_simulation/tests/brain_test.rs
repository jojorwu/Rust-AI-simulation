use rust_simulation::{
    brain::InventorySummary,
    components::Inventory,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[test]
fn test_inventory_summary_from_inventory() {
    let mut inventory = Inventory::new();
    inventory.add_item("wood", 10);
    inventory.add_item("berries", 5);
    inventory.add_item("stone_axe", 1);

    let summary = InventorySummary::from(&inventory);

    assert_eq!(summary.items.len(), 3);
    assert_eq!(summary.items.get("wood"), Some(&10));
    assert_eq!(summary.items.get("berries"), Some(&5));
    assert_eq!(summary.items.get("stone_axe"), Some(&1));
    assert_eq!(summary.items.get("iron_ore"), None);
}

#[test]
fn test_inventory_summary_hashing() {
    let mut inventory1 = Inventory::new();
    inventory1.add_item("wood", 10);
    inventory1.add_item("stone", 5);

    let mut inventory2 = Inventory::new();
    inventory2.add_item("stone", 5);
    inventory2.add_item("wood", 10);

    let summary1 = InventorySummary::from(&inventory1);
    let summary2 = InventorySummary::from(&inventory2);

    // The summaries should be equal, even though the insertion order was different.
    assert_eq!(summary1, summary2);

    // The hashes should also be equal.
    let mut hasher1 = DefaultHasher::new();
    summary1.hash(&mut hasher1);
    let hash1 = hasher1.finish();

    let mut hasher2 = DefaultHasher::new();
    summary2.hash(&mut hasher2);
    let hash2 = hasher2.finish();

    assert_eq!(hash1, hash2);
}
