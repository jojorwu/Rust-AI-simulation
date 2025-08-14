import random
import copy
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
        self.player = self._initialize_player()
        self.agent = self._initialize_agent()
        self.agent.q_table = load_q_table()

    def _initialize_player(self):
        """Creates the player and places them on a valid starting tile."""
        start_x, start_y = 0, 0
        while self.game_map.grid and self.game_map.grid[start_y][start_x] in ['#', 'T', 'S']:
            start_x = random.randint(0, self.game_map.width - 1)
            start_y = random.randint(0, self.game_map.height - 1)
        return Player(self.game_map, start_x, start_y)

    def _initialize_agent(self):
        """Initializes the AI agent."""
        actions = ['up', 'down', 'left', 'right', 'gather', 'craft']
        return Agent(actions,
                     learning_rate=config.LEARNING_RATE,
                     discount_factor=config.DISCOUNT_FACTOR,
                     epsilon=config.INITIAL_EPSILON)

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

    def run(self):
        """Runs the main training loop."""
        self.setup_new_map()
        original_map_grid = copy.deepcopy(self.game_map.grid)

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
                self.setup_new_map()
                original_map_grid = copy.deepcopy(self.game_map.grid)

            self.game_map.grid = copy.deepcopy(original_map_grid)
            self.player.reset()
            while self.game_map.grid[self.player.y][self.player.x] in ['#', 'T', 'S']:
                self.player.x = random.randint(0, self.game_map.width - 1)
                self.player.y = random.randint(0, self.game_map.height - 1)

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

            if self.agent.epsilon > config.MIN_EPSILON:
                self.agent.epsilon *= config.EPSILON_DECAY

            if (episode + 1) % 200 == 0:
                wood = self.player.inventory.get('wood', 0)
                stone = self.player.inventory.get('stone', 0)
                goal_item = self.player.inventory.get(config.CRAFTING_GOAL, 0)
                print(f"E{episode+1}/{config.EPISODES} | Inv: {wood}W, {stone}S, {goal_item}A | Epsilon: {self.agent.epsilon:.3f}")

        print("--- Training Finished ---")
        print(f"\nFinal inventory from last episode: {self.player.inventory}")
        print("Final Q-table size:", len(self.agent.q_table))
        save_q_table(self.agent.q_table)

    def _perform_action(self, action):
        """Performs the given action and returns the reward."""
        reward = -0.1  # Cost of living

        if action == 'gather':
            tile = self.game_map.grid[self.player.y][self.player.x]
            if tile == 'T':
                reward = 20
                self.player.inventory['wood'] = self.player.inventory.get('wood', 0) + 1
                self.game_map.grid[self.player.y][self.player.x] = '.'
            elif tile == 'S':
                reward = 20
                self.player.inventory['stone'] = self.player.inventory.get('stone', 0) + 1
                self.game_map.grid[self.player.y][self.player.x] = '.'
            else:
                reward = -2
        elif action == 'craft':
            goal = config.CRAFTING_GOAL
            recipe = recipes.RECIPES.get(goal)
            if recipe:
                has_resources = all(self.player.inventory.get(res, 0) >= amount for res, amount in recipe.items())
                if has_resources:
                    for res, amount in recipe.items():
                        self.player.inventory[res] -= amount
                    self.player.inventory[goal] = self.player.inventory.get(goal, 0) + 1
                    reward = 100
                else:
                    reward = -10
            else:
                reward = -1
        else:  # Movement action
            moved = self.player.move(action)
            if not moved:
                reward = -5
            elif self.game_map.grid[self.player.y][self.player.x] in ['T', 'S']:
                reward = 1
        return reward
