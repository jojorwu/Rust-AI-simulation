import random
import copy
from map import Map
from player import Player

def main():
    """Main function to run the Rust-like simulation."""
    print("Simulation starting...")
    # --- 1. Setup the environment ---
    game_map = Map(width=10, height=10)
    game_map.generate_random_map(obstacle_density=0.1)
    # Add some trees
    for _ in range(15):
        tx, ty = random.randint(0, game_map.width - 1), random.randint(0, game_map.height - 1)
        game_map.add_tree(tx, ty)

    # --- 2. Initialize the player ---
    start_x, start_y = 0, 0
    while game_map.grid[start_y][start_x] in ['#', 'T']:
        start_x = random.randint(0, game_map.width - 1)
        start_y = random.randint(0, game_map.height - 1)

    player = Player(game_map, start_x, start_y, learning_rate=0.1, discount_factor=0.9, epsilon=1.0)

    # Save the original map state
    original_map_grid = copy.deepcopy(game_map.grid)

    print("Initial Map:")
    game_map.display(player)
    print(f"Player starts at ({start_x}, {start_y})")

    # --- 3. Run the training loop ---
    episodes = 2000
    max_steps_per_episode = 100
    epsilon_decay = 0.999
    min_epsilon = 0.05
    wood_goal = 5

    print(f"\n--- Starting Training (Goal: Gather {wood_goal} wood) ---")
    for episode in range(episodes):
        # Reset environment and player for each episode
        game_map.grid = copy.deepcopy(original_map_grid)
        player.reset()

        # Place player in a random non-obstacle spot
        player.x = random.randint(0, game_map.width - 1)
        player.y = random.randint(0, game_map.height - 1)
        while game_map.grid[player.y][player.x] == '#':
            player.x = random.randint(0, game_map.width - 1)
            player.y = random.randint(0, game_map.height - 1)

        for step in range(max_steps_per_episode):
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
            if player.inventory.get('wood', 0) >= wood_goal:
                break

        # Decay epsilon
        if player.epsilon > min_epsilon:
            player.epsilon *= epsilon_decay

        if (episode + 1) % 200 == 0:
            print(f"Episode {episode + 1}/{episodes} | Wood gathered in this episode: {player.inventory.get('wood', 0)} | Epsilon: {player.epsilon:.3f}")

    print("--- Training Finished ---")

    # --- 4. Display results ---
    print(f"\nFinal inventory from last episode: {player.inventory}")
    print("Final Q-table size:", len(player.q_table))


if __name__ == "__main__":
    main()
