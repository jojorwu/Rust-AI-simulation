import random
import copy
from map import Map
from player import Player
import config
import recipes

def main():
    """Main function to run the Rust-like simulation."""
    print("Simulation starting...")
    # --- 1. Setup the environment ---
    game_map = Map(width=config.WIDTH, height=config.HEIGHT)
    game_map.generate_random_map(obstacle_density=config.OBSTACLE_DENSITY)
    # Add some trees
    for _ in range(config.NUM_TREES):
        tx, ty = random.randint(0, game_map.width - 1), random.randint(0, game_map.height - 1)
        game_map.add_tree(tx, ty)

    # --- 2. Initialize the player ---
    start_x, start_y = 0, 0
    while game_map.grid[start_y][start_x] in ['#', 'T']:
        start_x = random.randint(0, game_map.width - 1)
        start_y = random.randint(0, game_map.height - 1)

    player = Player(game_map, start_x, start_y,
                    learning_rate=config.LEARNING_RATE,
                    discount_factor=config.DISCOUNT_FACTOR,
                    epsilon=config.INITIAL_EPSILON)

    # Save the original map state
    original_map_grid = copy.deepcopy(game_map.grid)

    print("Initial Map:")
    game_map.display(player)
    print(f"Player starts at ({start_x}, {start_y})")

    # --- 3. Run the training loop ---
    print(f"\n--- Starting Training (Goal: Gather {config.WOOD_GOAL} wood) ---")
    for episode in range(config.EPISODES):
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
            state = player.get_state()
            action = player.choose_action()

            reward = -0.1  # Cost of living

            if action == 'gather':
                if game_map.grid[player.y][player.x] == 'T':
                    reward = 20  # Big reward for gathering wood
                    player.inventory['wood'] = player.inventory.get('wood', 0) + 1
                    game_map.grid[player.y][player.x] = '.' # Remove the tree for this episode
                else:
                    reward = -2 # Penalty for trying to gather from nothing
            else: # Movement action
                moved = player.move(action)
                if not moved:
                    reward = -5 # Penalty for bumping into a wall
                elif game_map.grid[player.y][player.x] == 'T':
                    reward = 1 # Small reward for being on a tree tile

            next_state = player.get_state()
            player.update_q_table(state, action, reward, next_state)

            # Check if goal is met
            if player.inventory.get('wood', 0) >= config.WOOD_GOAL:
                break

        # Decay epsilon
        if player.epsilon > config.MIN_EPSILON:
            player.epsilon *= config.EPSILON_DECAY

        if (episode + 1) % 200 == 0:
            print(f"Episode {episode + 1}/{config.EPISODES} | Wood gathered in this episode: {player.inventory.get('wood', 0)} | Epsilon: {player.epsilon:.3f}")

    print("--- Training Finished ---")

    # --- 4. Display results ---
    print(f"\nFinal inventory from last episode: {player.inventory}")
    print("Final Q-table size:", len(player.q_table))


if __name__ == "__main__":
    main()
