class Player:
    """
    Represents the player in the simulation. This class holds the player's
    state within the game world, such as position and inventory.
    """
    def __init__(self, map_instance, start_x, start_y):
        self.map = map_instance
        self.x = start_x
        self.y = start_y
        self.inventory = {}

    def get_state(self, view_radius=1):
        """
        Returns the current state of the player, including a local view and inventory.
        The state is a tuple containing a flattened view of the surrounding tiles
        and the player's inventory counts.
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
        wood_count = self.inventory.get('wood', 0)
        stone_count = self.inventory.get('stone', 0)

        # The final state is a combination of the view and inventory
        return tuple(local_view) + (wood_count, stone_count)

    def move(self, action):
        """Moves the player based on the chosen action."""
        dx, dy = 0, 0
        if action == 'up': dy = -1
        elif action == 'down': dy = 1
        elif action == 'left': dx = -1
        elif action == 'right': dx = 1

        new_x = self.x + dx
        new_y = self.y + dy

        if 0 <= new_x < self.map.width and 0 <= new_y < self.map.height and self.map.grid[new_y][new_x] != '#':
            self.x = new_x
            self.y = new_y
            return True
        return False

    def reset(self):
        """Resets the player's inventory."""
        self.inventory = {}
