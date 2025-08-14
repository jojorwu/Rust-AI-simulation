class Player:
    """
    Represents the player in the simulation. This class holds the player's
    state within the game world, such as position and inventory.
    """
    def __init__(self, map_instance, start_x, start_y):
        self.map = map_instance
        self.initial_x = start_x
        self.initial_y = start_y
        self.x = start_x
        self.y = start_y
        self.inventory = {}

    def get_state(self):
        """
        Returns the current state of the player, including position and inventory.
        The state is defined as (x, y, wood_count, stone_count).
        """
        wood_count = self.inventory.get('wood', 0)
        stone_count = self.inventory.get('stone', 0)
        return (self.x, self.y, wood_count, stone_count)

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
        """Resets the player's position and inventory."""
        self.x = self.initial_x
        self.y = self.initial_y
        self.inventory = {}
