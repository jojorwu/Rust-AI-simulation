from src import config

# Define stackable items. Tools are not stackable.
STACKABLE_ITEMS = ['wood', 'stone', 'sulfur', 'iron_ore', 'iron_bars']

class Player:
    """
    Represents the player in the simulation. This class holds the player's
    state within the game world, such as position and inventory.
    """
    def __init__(self, map_instance, start_x, start_y):
        self.map = map_instance
        self.x = start_x
        self.y = start_y
        # Inventory is now a list of slots, where each slot is a dict or None
        self.inventory = [None] * config.INVENTORY_SLOTS
        self.held_item = None

    def reset(self):
        """Resets the player's inventory and held item."""
        self.inventory = [None] * config.INVENTORY_SLOTS
        self.held_item = None

    # --- Inventory Helper Methods ---

    def find_item_slot(self, item_name):
        """Finds the first slot containing the given item."""
        for i, slot in enumerate(self.inventory):
            if slot and slot['item'] == item_name:
                return i
        return -1

    def find_empty_slot(self):
        """Finds the first empty slot."""
        for i, slot in enumerate(self.inventory):
            if slot is None:
                return i
        return -1

    def get_total_quantity(self, item_name):
        """Gets the total quantity of an item across all stacks in the inventory."""
        total = 0
        for slot in self.inventory:
            if slot and slot['item'] == item_name:
                total += slot['quantity']
        return total

    def has_resources(self, recipe):
        """Checks if the player has enough resources to craft an item."""
        for resource, required_amount in recipe.items():
            if self.get_total_quantity(resource) < required_amount:
                return False
        return True

    def add_item(self, item_name, quantity=1):
        """Adds an item to the inventory. Handles stacking."""
        # For stackable items, first try to add to an existing stack
        if item_name in STACKABLE_ITEMS:
            for slot in self.inventory:
                if slot and slot['item'] == item_name:
                    slot['quantity'] += quantity
                    return True

        # If not stackable or no existing stack was found, find an empty slot
        empty_slot_index = self.find_empty_slot()
        if empty_slot_index != -1:
            self.inventory[empty_slot_index] = {'item': item_name, 'quantity': quantity}
            return True

        # Inventory is full
        return False

    def remove_resources(self, recipe):
        """A more robust way to remove resources for crafting."""
        if not self.has_resources(recipe):
            return False

        for resource, amount_to_remove in recipe.items():
            removed_amount = 0
            while removed_amount < amount_to_remove:
                amount_needed = amount_to_remove - removed_amount

                # Find a stack of the resource to remove from
                # To be simple, we just find the first one. A better implementation
                # could choose the smallest stack, etc.
                slot_to_remove_from = None
                slot_index = -1
                for i, slot in enumerate(self.inventory):
                    if slot and slot['item'] == resource:
                        slot_to_remove_from = slot
                        slot_index = i
                        break

                if slot_to_remove_from is None:
                     # Should not happen if has_resources() check passed
                    return False

                removable_amount = min(amount_needed, slot_to_remove_from['quantity'])

                slot_to_remove_from['quantity'] -= removable_amount
                removed_amount += removable_amount

                if slot_to_remove_from['quantity'] == 0:
                    self.inventory[slot_index] = None
        return True


    # --- State and Action Methods ---

    def get_state(self, view_radius=1):
        """
        Returns the current state of the player, including a local view and inventory.
        The state is a tuple containing a flattened view of the surrounding tiles,
        the inventory slots, and the currently held item.
        """
        # Get local view
        local_view = []
        for dy in range(-view_radius, view_radius + 1):
            for dx in range(-view_radius, view_radius + 1):
                nx, ny = self.x + dx, self.y + dy
                if 0 <= nx < self.map.width and 0 <= ny < self.map.height:
                    local_view.append(self.map.grid[ny][nx])
                else:
                    local_view.append('X') # 'X' for out of bounds

        # Get inventory state
        inventory_state = []
        for slot in self.inventory:
            if slot:
                # To keep the state space manageable, we might need to normalize quantities later
                inventory_state.extend([slot['item'], slot['quantity']])
            else:
                inventory_state.extend([None, 0]) # Represent empty slot

        # The final state is a combination of the view, inventory, and held item
        return tuple(local_view) + tuple(inventory_state) + (self.held_item,)

    def move(self, action):
        """Moves the player based on the chosen action."""
        dx, dy = 0, 0
        if action == 'up': dy = -1
        elif action == 'down': dy = 1
        elif action == 'left': dx = -1
        elif action == 'right': dx = 1

        new_x = self.x + dx
        new_y = self.y + dy

        if 0 <= new_x < self.map.width and 0 <= new_y < self.map.height and self.map.grid[new_y][new_x] != 'W':
            self.x = new_x
            self.y = new_y
            return True
        return False
