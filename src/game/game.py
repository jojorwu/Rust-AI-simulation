import random
import copy
import math
from src.game.map import Map
from src.game.player import Player
from src.ai.agent import Agent
from src.utils.persistence import load_q_table, save_q_table
from src import config
from src import recipes

class Game:
    """
    The main controller for the game simulation.
    Manages the game state, the main loop, and all interactions.
    """
    def __init__(self):
        print("Simulation starting...")
        self.game_map = Map(width=config.WIDTH, height=config.HEIGHT)
        # Player is created at a temporary position. A valid one is set in run().
        self.player = Player(self.game_map, 0, 0)
        self.agent = self._initialize_agent()
        self.agent.q_table = load_q_table()

        # Performance tracking
        self.cycle_successes = 0
        self.last_cycle_performance = 0.0
        self.current_cycle_episodes = 0

    def _initialize_agent(self):
        """Initializes the AI agent."""
        actions = [
            'up', 'down', 'left', 'right', 'gather',
            'craft_stone_axe', 'craft_stone_pickaxe', 'craft_furnace', 'craft_metal_pickaxe',
            'equip_stone_axe', 'equip_stone_pickaxe', 'equip_metal_pickaxe',
            'place_furnace', 'smelt_iron'
        ]
        return Agent(actions,
                     learning_rate=config.LEARNING_RATE,
                     discount_factor=config.DISCOUNT_FACTOR,
                     epsilon=config.INITIAL_EPSILON)

    def _is_adjacent_to(self, tile_type):
        """Checks if the player is adjacent to a given tile type."""
        px, py = self.player.x, self.player.y
        for dx, dy in [(0, 1), (0, -1), (1, 0), (-1, 0)]:
            nx, ny = px + dx, py + dy
            if 0 <= nx < self.game_map.width and 0 <= ny < self.game_map.height:
                if self.game_map.grid[ny][nx] == tile_type:
                    return True
        return False

    def _find_and_set_valid_start_pos(self):
        """Finds a valid random starting position for the player and updates player's state."""
        start_x, start_y = 0, 0
        # Loop until a valid starting tile ('.') is found
        while True:
            start_x = random.randint(0, self.game_map.width - 1)
            start_y = random.randint(0, self.game_map.height - 1)
            if self.game_map.grid[start_y][start_x] == '.':
                break
        self.player.x = start_x
        self.player.y = start_y

    def setup_new_map(self):
        """Generates a new island map and populates it with biome-aware resources."""
        print("\n--- GENERATING NEW MAP ---")
        self.game_map.generate_island_map()

        # Get lists of valid tiles for each biome
        plains_tiles = []
        mountain_tiles = []
        for y in range(self.game_map.height):
            for x in range(self.game_map.width):
                if self.game_map.grid[y][x] == '.':
                    plains_tiles.append((x, y))
                elif self.game_map.grid[y][x] == 'M':
                    mountain_tiles.append((x, y))

        # Spawn resources in their respective biomes
        # Trees spawn in plains
        spawn_locations = random.sample(plains_tiles, min(len(plains_tiles), config.NUM_TREES))
        for x, y in spawn_locations:
            self.game_map.add_tree(x, y)

        # Rock, Sulfur, and Iron spawn in mountains and plains
        rock_spawn_candidates = plains_tiles + mountain_tiles
        spawn_locations = random.sample(rock_spawn_candidates, min(len(rock_spawn_candidates), config.NUM_STONE))
        for x, y in spawn_locations:
            self.game_map.add_rock(x, y)

        sulfur_spawn_candidates = plains_tiles + mountain_tiles
        spawn_locations = random.sample(sulfur_spawn_candidates, min(len(sulfur_spawn_candidates), config.NUM_SULFUR))
        for x, y in spawn_locations:
            self.game_map.add_sulfur(x, y)

        iron_spawn_candidates = mountain_tiles # Iron is only in mountains
        spawn_locations = random.sample(iron_spawn_candidates, min(len(iron_spawn_candidates), config.NUM_IRON_ORE))
        for x, y in spawn_locations:
            self.game_map.add_iron_ore_node(x, y)

    def run(self):
        """Runs the main training loop."""
        self.setup_new_map()
        original_map_grid = copy.deepcopy(self.game_map.grid)

        self._find_and_set_valid_start_pos()

        print("Initial Map:")
        self.game_map.display(self.player)
        print(f"Player starts at ({self.player.x}, {self.player.y})")

        total_ticks = 0
        time_of_day = 'DAY'
        day_cycle_length = config.DAY_LENGTH + config.NIGHT_LENGTH

        print(f"\n--- Starting Training (Goal: Craft a {config.CRAFTING_GOAL}) ---")
        for episode in range(config.EPISODES):
            if episode > 0 and episode % config.WIPE_CYCLE == 0:
                print(f"\n\n--- SERVER WIPE AT EPISODE {episode} ---\n")

                if self.current_cycle_episodes > 0:
                    success_rate = self.cycle_successes / self.current_cycle_episodes
                    print(f"--- Cycle Performance ---")
                    print(f"Episodes this cycle: {self.current_cycle_episodes}")
                    print(f"Successes this cycle: {self.cycle_successes}")
                    print(f"Success rate: {success_rate:.2%}")
                    print(f"Previous cycle's success rate: {self.last_cycle_performance:.2%}")

                    if math.isclose(success_rate, self.last_cycle_performance):
                        print("Performance stable. Learning rate remains unchanged.")
                    elif success_rate > self.last_cycle_performance:
                        self.agent.learning_rate = min(config.MAX_LEARNING_RATE, self.agent.learning_rate + config.LEARNING_RATE_ADJUSTMENT)
                        print(f"Performance improved. Increasing learning rate to {self.agent.learning_rate:.3f}")
                    else:
                        self.agent.learning_rate = max(config.MIN_LEARNING_RATE, self.agent.learning_rate - config.LEARNING_RATE_ADJUSTMENT)
                        print(f"Performance declined. Decreasing learning rate to {self.agent.learning_rate:.3f}")

                    self.last_cycle_performance = success_rate
                else:
                    print("No episodes in this cycle to evaluate.")

                self.cycle_successes = 0
                self.current_cycle_episodes = 0
                self.setup_new_map()
                original_map_grid = copy.deepcopy(self.game_map.grid)

            self.game_map.grid = copy.deepcopy(original_map_grid)
            self.player.reset()
            self._find_and_set_valid_start_pos()

            for step in range(config.MAX_STEPS_PER_EPISODE):
                total_ticks += 1
                state = self.player.get_state()
                action = self.agent.choose_action(state)
                reward = self._perform_action(action)
                next_state = self.player.get_state()
                self.agent.update_q_table(state, action, reward, next_state)

                if self.player.get_total_quantity(config.CRAFTING_GOAL) > 0:
                    break

            self.current_cycle_episodes += 1
            if self.player.get_total_quantity(config.CRAFTING_GOAL) > 0:
                self.cycle_successes += 1

            if self.agent.epsilon > config.MIN_EPSILON:
                self.agent.epsilon *= config.EPSILON_DECAY

            if (episode + 1) % 200 == 0:
                wood = self.player.get_total_quantity('wood')
                stone = self.player.get_total_quantity('stone')
                sulfur = self.player.get_total_quantity('sulfur')
                goal_item = self.player.get_total_quantity(config.CRAFTING_GOAL)
                print(f"E{episode+1}/{config.EPISODES} | Inv: {wood}W, {stone}R, {sulfur}U, {goal_item}A | Epsilon: {self.agent.epsilon:.3f}")

        print("--- Training Finished ---")
        print(f"\nFinal inventory from last episode: {self.player.inventory}")
        save_q_table(self.agent.q_table)

    def _perform_action(self, action):
        """Performs the given action and returns the reward."""
        reward = -0.1  # Time penalty

        # Movement
        if action in ['up', 'down', 'left', 'right']:
            if self.player.move(action):
                # Move was successful, check for special tile penalties/rewards
                current_tile = self.game_map.grid[self.player.y][self.player.x]
                if current_tile == 'M':
                    reward = -2.0  # Higher cost for moving in mountains
                elif current_tile in ['R', 'U', 'I', 'T']:
                    reward = 1.0 # Small reward for moving onto a resource
            else:
                # Move was unsuccessful (e.g., into water)
                reward = -5

        # Equip
        elif action.startswith('equip_'):
            item = action.split('_', 1)[1]
            if self.player.get_total_quantity(item) > 0:
                self.player.held_item = item
                reward = 2
            else:
                reward = -2

        # Craft
        elif action.startswith('craft_'):
            item = action.split('_', 1)[1]
            recipe = recipes.RECIPES.get(item)
            if recipe and self.player.has_resources(recipe):
                if self.player.remove_resources(recipe):
                    if self.player.add_item(item):
                        reward = 100 if item == config.CRAFTING_GOAL else 50
                    else:
                        reward = -15 # Inventory full
                else:
                    reward = -15 # Failed to remove resources, should be rare
            else:
                reward = -10 # Not enough resources

        # Place Furnace
        elif action == 'place_furnace':
            if self.player.get_total_quantity('furnace') > 0 and self.game_map.grid[self.player.y][self.player.x] == '.':
                self.player.remove_resources({'furnace': 1})
                self.game_map.grid[self.player.y][self.player.x] = 'F'
                reward = 40
            else:
                reward = -5

        # Smelt Iron
        elif action == 'smelt_iron':
            smelting_recipe = {'iron_ore': 1, 'wood': 1}
            if self._is_adjacent_to('F') and self.player.has_resources(smelting_recipe):
                if self.player.remove_resources(smelting_recipe):
                    self.player.add_item('iron_bars')
                    reward = 60
                else:
                    reward = -15 # Should not happen
            else:
                reward = -12

        # Gather
        elif action == 'gather':
            tile = self.game_map.grid[self.player.y][self.player.x]
            held_item = self.player.held_item

            tool_map = {
                'T': ('stone_axe', 'wood', 20),
                'R': ('stone_pickaxe', 'stone', 20),
                'U': ('stone_pickaxe', 'sulfur', 30),
                'I': ('metal_pickaxe', 'iron_ore', 40)
            }

            if tile in tool_map:
                required_tool, resource, reward_val = tool_map[tile]
                if held_item == required_tool:
                    if self.player.add_item(resource):
                        self.game_map.grid[self.player.y][self.player.x] = '.'
                        reward = reward_val
                    else:
                        reward = -15 # Inventory full
                else:
                    reward = -10 # Wrong tool
            else:
                reward = -2 # Gathering on empty tile

        return reward
