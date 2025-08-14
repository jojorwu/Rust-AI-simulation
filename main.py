import random
import copy
from map import Map
from player import Player
import config
import recipes

def setup_new_map(game_map):
    """Generates a new random map and populates it with resources."""
    print("\n--- GENERATING NEW MAP ---")
    game_map.generate_random_map(config.OBSTACLE_DENSITY)
    # Add some trees
    for _ in range(config.NUM_TREES):
        tx, ty = random.randint(0, game_map.width - 1), random.randint(0, game_map.height - 1)
        game_map.add_tree(tx, ty)

    # Add some stone
    for _ in range(config.NUM_STONE):
        sx, sy = random.randint(0, game_map.width - 1), random.randint(0, game_map.height - 1)
        game_map.add_stone(sx, sy)

def main():
    """Main function to run the Rust-like simulation."""
    print("Simulation starting...")
    # --- 1. Setup the environment ---
    game_map = Map(width=config.WIDTH, height=config.HEIGHT)
    setup_new_map(game_map)

    # --- 2. Initialize the player ---
    start_x, start_y = 0, 0
    while game_map.grid[start_y][start_x] in ['#', 'T']:
        start_x = random.randint(0, game_map.width - 1)
        start_y = random.randint(0, game_map.height - 1)

    player = Player(game_map, start_x, start_y,
                    learning_rate=config.LEARNING_RATE,
                    discount_factor=config.DISCOUNT_FACTOR,
                    epsilon=config.INITIAL_EPSILON)

    original_map_grid = copy.deepcopy(game_map.grid)

    print("Initial Map:")
    game_map.display(player)
    print(f"Player starts at ({start_x}, {start_y})")

    # --- 3. Run the training loop ---
    total_ticks = 0
    time_of_day = 'DAY'
    day_cycle_length = config.DAY_LENGTH + config.NIGHT_LENGTH

    print(f"\n--- Starting Training (Goal: Craft a {config.CRAFTING_GOAL}) ---")
    for episode in range(config.EPISODES):
        # --- WIPE MECHANIC ---
        if episode > 0 and episode % config.WIPE_CYCLE == 0:
            print(f"\n\n--- SERVER WIPE AT EPISODE {episode} ---\n")
            setup_new_map(game_map)
            original_map_grid = copy.deepcopy(game_map.grid)
            # Player keeps its learned knowledge (Q-table)

        # Reset environment and player for each episode
        game_map.grid = copy.deepcopy(original_map_grid)
        player.reset()

        # Place player in a random non-obstacle spot
        player.x = random.randint(0, game_map.width - 1)
        player.y = random.randint(0, game_map.height - 1)
        while game_map.grid[player.y][player.x] == '#':
            player.x = random.randint(0, game_map.width - 1)
            player.y = random.randint(0, game_map.height - 1)

        for step in range(config.MAX_STEPS_PER_EPISODE):
            total_ticks += 1

            # Check for day/night change
            current_cycle_tick = total_ticks % day_cycle_length
            new_time_of_day = 'DAY' if current_cycle_tick < config.DAY_LENGTH else 'NIGHT'

            if new_time_of_day != time_of_day:
                time_of_day = new_time_of_day
                print(f"\n--- It is now {time_of_day} (Total Ticks: {total_ticks}) ---")

            state = player.get_state()
            action = player.choose_action()

            reward = -0.1  # Cost of living

            if action == 'gather':
                tile = game_map.grid[player.y][player.x]
                if tile == 'T':
                    reward = 20
                    player.inventory['wood'] = player.inventory.get('wood', 0) + 1
                    game_map.grid[player.y][player.x] = '.' # Deplete resource
                elif tile == 'S':
                    reward = 20
                    player.inventory['stone'] = player.inventory.get('stone', 0) + 1
                    game_map.grid[player.y][player.x] = '.' # Deplete resource
                else:
                    reward = -2
            elif action == 'craft':
                # Attempt to craft the goal item
                goal = config.CRAFTING_GOAL
                recipe = recipes.RECIPES.get(goal)
                if recipe:
                    # Check if player has enough resources
                    has_resources = all(player.inventory.get(res, 0) >= amount for res, amount in recipe.items())

                    if has_resources:
                        # Deduct resources and add crafted item
                        for res, amount in recipe.items():
                            player.inventory[res] -= amount
                        player.inventory[goal] = player.inventory.get(goal, 0) + 1
                        reward = 100 # Huge reward for crafting the goal item!
                        print(f"--- Successfully crafted a {goal}! Episode finished. ---")
                    else:
                        reward = -10 # Penalty for trying to craft without resources
                else:
                    reward = -1 # Should not happen if config is correct
            else: # Movement action
                moved = player.move(action)
                if not moved:
                    reward = -5 # Penalty for bumping into a wall
                elif game_map.grid[player.y][player.x] in ['T', 'S']:
                    reward = 1 # Small reward for being on a resource tile

            next_state = player.get_state()
            player.update_q_table(state, action, reward, next_state)

            # Check if goal is met
            if player.inventory.get(config.CRAFTING_GOAL, 0) > 0:
                break

        # Decay epsilon
        if player.epsilon > config.MIN_EPSILON:
            player.epsilon *= config.EPSILON_DECAY

        if (episode + 1) % 200 == 0:
            wood = player.inventory.get('wood', 0)
            stone = player.inventory.get('stone', 0)
            goal_item = player.inventory.get(config.CRAFTING_GOAL, 0)
            print(f"E{episode+1}/{config.EPISODES} | Inv: {wood} W, {stone} S, {goal_item} Axe | Epsilon: {player.epsilon:.3f}")

    print("--- Training Finished ---")

    # --- 4. Display results ---
    print(f"\nFinal inventory from last episode: {player.inventory}")
    print("Final Q-table size:", len(player.q_table))


if __name__ == "__main__":
    main()
