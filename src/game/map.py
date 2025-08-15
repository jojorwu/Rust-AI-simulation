import random
import noise

class Map:
    """
    Represents the simulation map.
    """
    def __init__(self, width, height):
        self.width = width
        self.height = height
        self.grid = []

    def generate_island_map(self, scale=50.0, octaves=6, persistence=0.5, lacunarity=2.0, seed=None):
        """
        Generates a random island map using Perlin noise.
        """
        if seed is None:
            seed = random.randint(0, 100)

        self.grid = [['' for _ in range(self.width)] for _ in range(self.height)]

        for y in range(self.height):
            for x in range(self.width):
                # Calculate distance from center for radial gradient
                nx = 2 * x / self.width - 1
                ny = 2 * y / self.height - 1
                # Using a squared distance for a more circular island
                dist = 1 - (1 - nx**2) * (1 - ny**2)

                # Generate Perlin noise value
                noise_val = noise.pnoise2(x / scale,
                                          y / scale,
                                          octaves=octaves,
                                          persistence=persistence,
                                          lacunarity=lacunarity,
                                          repeatx=self.width,
                                          repeaty=self.height,
                                          base=seed)

                # Combine noise with radial gradient to form an island
                island_val = (noise_val + 1) / 2 # Normalize noise to 0-1 range
                height = island_val * (1 - dist)

                # Assign tile based on height
                if height < 0.1:
                    self.grid[y][x] = 'W'  # Water
                elif height < 0.15:
                    self.grid[y][x] = 'S'  # Sand
                elif height < 0.5:
                    self.grid[y][x] = '.'  # Plains
                else:
                    self.grid[y][x] = 'M'  # Mountain

    def display(self, player=None):
        """
        Displays the map in the console, with the player's position.
        """
        for y, row in enumerate(self.grid):
            display_row = []
            for x, tile in enumerate(row):
                if player and player.x == x and player.y == y:
                    display_row.append('P')
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

    def add_rock(self, x, y):
        """
        Adds a rock (stone) node at the given coordinates.
        """
        if 0 <= x < self.width and 0 <= y < self.height:
            if self.grid[y][x] in ['.', 'M']:
                self.grid[y][x] = 'R'
                return True
        return False

    def add_sulfur(self, x, y):
        """
        Adds a sulfur node at the given coordinates.
        """
        if 0 <= x < self.width and 0 <= y < self.height:
            if self.grid[y][x] in ['.', 'M']:
                self.grid[y][x] = 'U'
                return True
        return False

    def add_iron_ore_node(self, x, y):
        """
        Adds an iron ore node at the given coordinates.
        """
        if 0 <= x < self.width and 0 <= y < self.height:
            if self.grid[y][x] in ['.', 'M']:
                self.grid[y][x] = 'I'
                return True
        return False
