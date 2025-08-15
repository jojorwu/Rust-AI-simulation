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
            'craft_stone_axe', 'craft_stone_pickaxe',
            'equip_stone_axe', 'equip_stone_pickaxe'
        ]
        return Agent(actions,
                     learning_rate=config.LEARNING_RATE,
                     discount_factor=config.DISCOUNT_FACTOR,
                     epsilon=config.INITIAL_EPSILON)

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
        """Generates a new random map and populates it with resources."""
        print("\n--- GENERATING NEW MAP ---")
        self.game_map.generate_random_map(config.OBSTACLE_DENSITY)
        for _ in range(config.NUM_TREES):
            tx, ty = random.randint(0, self.game_map.width - 1), random.randint(0, self.game_map.height - 1)
            self.game_map.add_tree(tx, ty)
        for _ in range(config.NUM_STONE):
            sx, sy = random.randint(0, self.game_map.width - 1), random.randint(0, self.game_map.height - 1)
            self.game_map.add_stone(sx, sy)
        for _ in range(config.NUM_SULFUR):
            ux, uy = random.randint(0, self.game_map.width - 1), random.randint(0, self.game_map.height - 1)
            self.game_map.add_sulfur(ux, uy)

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

                # System improvement phase
                if self.current_cycle_episodes > 0:
                    success_rate = self.cycle_successes / self.current_cycle_episodes
                    print(f"--- Cycle Performance ---")
                    print(f"Episodes this cycle: {self.current_cycle_episodes}")
                    print(f"Successes this cycle: {self.cycle_successes}")
                    print(f"Success rate: {success_rate:.2%}")
                    print(f"Previous cycle's success rate: {self.last_cycle_performance:.2%}")

                    # Adjust learning rate based on performance
                    if math.isclose(success_rate, self.last_cycle_performance):
                        print("Performance stable. Learning rate remains unchanged.")
                    elif success_rate > self.last_cycle_performance:
                        self.agent.learning_rate = min(config.MAX_LEARNING_RATE, self.agent.learning_rate + config.LEARNING_RATE_ADJUSTMENT)
                        print(f"Performance improved. Increasing learning rate to {self.agent.learning_rate:.3f}")
                    else: # success_rate < self.last_cycle_performance
                        self.agent.learning_rate = max(config.MIN_LEARNING_RATE, self.agent.learning_rate - config.LEARNING_RATE_ADJUSTMENT)
                        print(f"Performance declined. Decreasing learning rate to {self.agent.learning_rate:.3f}")

                    self.last_cycle_performance = success_rate
                else:
                    print("No episodes in this cycle to evaluate.")

                # Reset counters for the new cycle
                self.cycle_successes = 0
                self.current_cycle_episodes = 0

                self.setup_new_map()
                original_map_grid = copy.deepcopy(self.game_map.grid)

            # Reset episode state
            self.game_map.grid = copy.deepcopy(original_map_grid)
            self.player.reset() # Resets inventory
            self._find_and_set_valid_start_pos() # Find a new valid start pos for the episode

            for step in range(config.MAX_STEPS_PER_EPISODE):
                total_ticks += 1
                current_cycle_tick = total_ticks % day_cycle_length
                new_time_of_day = 'DAY' if current_cycle_tick < config.DAY_LENGTH else 'NIGHT'
                if new_time_of_day != time_of_day:
                    time_of_day = new_time_of_day

                state = self.player.get_state()
                action = self.agent.choose_action(state)
                reward = self._perform_action(action)
                next_state = self.player.get_state()
                self.agent.update_q_table(state, action, reward, next_state)

                if self.player.inventory.get(config.CRAFTING_GOAL, 0) > 0:
                    break

            self.current_cycle_episodes += 1
            if self.player.inventory.get(config.CRAFTING_GOAL, 0) > 0:
                self.cycle_successes += 1

            if self.agent.epsilon > config.MIN_EPSILON:
                self.agent.epsilon *= config.EPSILON_DECAY

            if (episode + 1) % 200 == 0:
                wood = self.player.inventory.get('wood', 0)
                stone = self.player.inventory.get('stone', 0)
                sulfur = self.player.inventory.get('sulfur', 0)
                goal_item = self.player.inventory.get(config.CRAFTING_GOAL, 0)
                print(f"E{episode+1}/{config.EPISODES} | Inv: {wood}W, {stone}S, {sulfur}U, {goal_item}A | Epsilon: {self.agent.epsilon:.3f}")

        print("--- Training Finished ---")
        print(f"\nFinal inventory from last episode: {self.player.inventory}")
        print("Final Q-table size:", len(self.agent.q_table))
        save_q_table(self.agent.q_table)

    def _perform_action(self, action):
        """Performs the given action and returns the reward."""
        reward = -0.1  # Base cost of living/time penalty

        # Movement actions
        if action in ['up', 'down', 'left', 'right']:
            moved = self.player.move(action)
            if not moved:
                reward = -5  # Penalty for bumping into walls
            elif self.game_map.grid[self.player.y][self.player.x] in ['T', 'S', 'U']:
                reward = 1  # Small reward for moving to a resource tile

        # Equip actions
        elif action.startswith('equip_'):
            item_to_equip = action.split('_', 1)[1]
            if self.player.inventory.get(item_to_equip, 0) > 0:
                self.player.held_item = item_to_equip
                # print(f"DEBUG: Player equipped {item_to_equip}")
                reward = 2  # Reward for equipping a tool
            else:
                # print(f"DEBUG: Player failed to equip {item_to_equip} (not in inventory)")
                reward = -2  # Penalty for equipping an item they don't have

        # Craft actions
        elif action.startswith('craft_'):
            item_to_craft = action.split('_', 1)[1]
            recipe = recipes.RECIPES.get(item_to_craft)
            if recipe:
                has_resources = all(self.player.inventory.get(res, 0) >= amount for res, amount in recipe.items())
                if has_resources:
                    for res, amount in recipe.items():
                        self.player.inventory[res] -= amount
                    self.player.inventory[item_to_craft] = self.player.inventory.get(item_to_craft, 0) + 1
                    # Big reward for crafting the main goal item, smaller for others
                    reward = 100 if item_to_craft == config.CRAFTING_GOAL else 50
                else:
                    reward = -10  # Penalty for trying to craft without resources
            else:
                reward = -1 # Should not happen if actions are defined correctly

        # Gather action
        elif action == 'gather':
            tile = self.game_map.grid[self.player.y][self.player.x]
            held_item = self.player.held_item
            # print(f"DEBUG: Player holding '{held_item}' attempts to gather from tile '{tile}'")

            if tile == 'T':  # Tree
                if held_item == 'stone_axe':
                    reward = 20
                    self.player.inventory['wood'] = self.player.inventory.get('wood', 0) + 1
                    self.game_map.grid[self.player.y][self.player.x] = '.'
                else:
                    # print(f"DEBUG: Gather failed - wrong tool.")
                    reward = -10  # Big penalty for wrong tool
            elif tile == 'S':  # Stone
                if held_item == 'stone_pickaxe':
                    reward = 20
                    self.player.inventory['stone'] = self.player.inventory.get('stone', 0) + 1
                    self.game_map.grid[self.player.y][self.player.x] = '.'
                else:
                    # print(f"DEBUG: Gather failed - wrong tool.")
                    reward = -10 # Big penalty for wrong tool
            elif tile == 'U': # Sulfur
                if held_item == 'stone_pickaxe':
                    reward = 30 # Sulfur is more valuable
                    self.player.inventory['sulfur'] = self.player.inventory.get('sulfur', 0) + 1
                    self.game_map.grid[self.player.y][self.player.x] = '.'
                else:
                    # print(f"DEBUG: Gather failed - wrong tool.")
                    reward = -10 # Big penalty for wrong tool
            else:
                reward = -2  # Penalty for trying to gather from an empty tile

        return reward
