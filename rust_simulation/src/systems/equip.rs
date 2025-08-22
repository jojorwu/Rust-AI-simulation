use crate::components::{intents::WantsToEquip, Equipped, Inventory};
use crate::events::Event;
use bevy_ecs::prelude::*;

pub fn equip_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Equipped, &mut Inventory, &WantsToEquip)>,
    mut event_writer: EventWriter<Event>,
) {
    for (entity, mut equipped, mut inventory, wants_to_equip) in query.iter_mut() {
        let item_name = &wants_to_equip.0;

        // Check if the agent has the item.
        if inventory.has_item(item_name, 1) {
            // If something is already equipped, unequip it first by putting it back in inventory.
            if let Some(old_tool) = equipped.tool.take() {
                inventory.add_item(&old_tool, 1);
            }

            // Remove the new item from inventory and equip it.
            // This should not fail if has_item is correct.
            if inventory.remove_item(item_name, 1) {
                equipped.tool = Some(item_name.clone());
                event_writer.send(Event::ToolEquipped {
                    entity,
                    tool_name: item_name.clone(),
                });
            }
        }

        // The intent is always removed, whether successful or not.
        commands.entity(entity).remove::<WantsToEquip>();
    }
}
