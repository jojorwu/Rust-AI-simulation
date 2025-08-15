import random

class Map:
    """
    Represents the simulation map.
    """
    def __init__(self, width, height):
        self.width = width
        self.height = height
        self.grid = []

    def generate_random_map(self, obstacle_density=0.2):
        """
        Generates a random map with obstacles.
        """
        for y in range(self.height):
            row = []
            for x in range(self.width):
                if random.random() < obstacle_density:
                    row.append('#')  # Obstacle
                else:
                    row.append('.')  # Empty space
            self.grid.append(row)

    def display(self, player=None):
        """
        Displays the map in the console, with the player's position.
        """
        for y, row in enumerate(self.grid):
            display_row = []
            for x, tile in enumerate(row):
                if player and player.x == x and player.y == y:
                    display_row.append('P')  # Player symbol
                else:
                    display_row.append(tile)
            print(' '.join(display_row))

    def add_tree(self, x, y):
        """
        Adds a tree at the given coordinates if the tile is empty.
        """
        if 0 <= x < self.width and 0 <= y < self.height:
            if self.grid[y][x] == '.':
                self.grid[y][x] = 'T'
                return True
        return False

    def add_stone(self, x, y):
        """
        Adds a stone node at the given coordinates if the tile is empty.
        """
        if 0 <= x < self.width and 0 <= y < self.height:
            if self.grid[y][x] == '.':
                self.grid[y][x] = 'S'
                return True
        return False

    def add_sulfur(self, x, y):
        """
        Adds a sulfur node at the given coordinates if the tile is empty.
        """
        if 0 <= x < self.width and 0 <= y < self.height:
            if self.grid[y][x] == '.':
                self.grid[y][x] = 'U'
                return True
        return False
