import random

class Map:
    """
    Represents the simulation map.
    """
    def __init__(self, width, height):
        self.width = width
        self.height = height
        self.grid = []
        self.goal_pos = None

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

    def display(self, agent=None):
        """
        Displays the map in the console, with the agent's position.
        """
        for y, row in enumerate(self.grid):
            display_row = []
            for x, tile in enumerate(row):
                if agent and agent.x == x and agent.y == y:
                    display_row.append('A')  # Agent symbol
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

    def add_goal(self, x, y):
        """
        Adds a goal at the given coordinates.
        """
        if 0 <= x < self.width and 0 <= y < self.height:
            if self.grid[y][x] == '.':
                self.grid[y][x] = 'G'
                self.goal_pos = (x, y)
                return True
        return False

class Agent:
    """
    Represents the agent in the simulation with Q-learning capabilities.
    """
    def __init__(self, map_instance, start_x, start_y, learning_rate=0.1, discount_factor=0.9, epsilon=0.9):
        self.map = map_instance
        self.initial_x = start_x
        self.initial_y = start_y
        self.x = start_x
        self.y = start_y

        self.learning_rate = learning_rate
        self.discount_factor = discount_factor
        self.epsilon = epsilon
        self.q_table = {}
        self.actions = ['up', 'down', 'left', 'right']

    def get_state(self):
        """Returns the current state of the agent (its position)."""
        return (self.x, self.y)

    def choose_action(self):
        """Chooses an action using an epsilon-greedy policy."""
        if random.random() < self.epsilon:
            return random.choice(self.actions)  # Explore
        else:
            state = self.get_state()
            # If state not in q_table, choose randomly
            if state not in self.q_table:
                 return random.choice(self.actions)
            q_values = self.q_table.get(state, {action: 0 for action in self.actions})
            return max(q_values, key=q_values.get) # Exploit

    def move(self, action):
        """Moves the agent based on the chosen action."""
        dx, dy = 0, 0
        if action == 'up': dy = -1
        elif action == 'down': dy = 1
        elif action == 'left': dx = -1
        elif action == 'right': dx = 1

        new_x = self.x + dx
        new_y = self.y + dy

        if 0 <= new_x < self.map.width and 0 <= new_y < self.map.height and self.map.grid[new_y][new_x] not in ['#', 'T']:
            self.x = new_x
            self.y = new_y
            return True
        return False

    def update_q_table(self, state, action, reward, next_state):
        """Updates the Q-table using the Q-learning formula."""
        old_value = self.q_table.get(state, {}).get(action, 0.0)

        next_max = 0.0
        if next_state in self.q_table:
            next_max = max(self.q_table[next_state].values())

        new_value = old_value + self.learning_rate * (reward + self.discount_factor * next_max - old_value)

        if state not in self.q_table:
            self.q_table[state] = {act: 0.0 for act in self.actions}
        self.q_table[state][action] = new_value

    def reset(self):
        """Resets the agent's position to the start."""
        self.x = self.initial_x
        self.y = self.initial_y

def main():
    """Main function to run the Q-learning simulation."""
    print("Simulation starting...")
    # --- 1. Setup the environment ---
    game_map = Map(width=10, height=10)
    game_map.generate_random_map(obstacle_density=0.2)
    game_map.add_tree(3, 4)
    game_map.add_tree(6, 7)

    # Set a goal
    goal_x, goal_y = 9, 9
    while game_map.grid[goal_y][goal_x] in ['#', 'T']:
        goal_x = random.randint(0, game_map.width - 1)
        goal_y = random.randint(0, game_map.height - 1)
    game_map.add_goal(goal_x, goal_y)

    # --- 2. Initialize the agent ---
    start_x, start_y = 0, 0
    while game_map.grid[start_y][start_x] in ['#', 'T', 'G']:
        start_x = random.randint(0, game_map.width - 1)
        start_y = random.randint(0, game_map.height - 1)

    agent = Agent(game_map, start_x, start_y, learning_rate=0.1, discount_factor=0.9, epsilon=1.0)

    print("Initial Map:")
    game_map.display(agent)
    print(f"Agent starts at ({start_x}, {start_y})")
    print(f"Goal is at {game_map.goal_pos}")

    # --- 3. Run the training loop ---
    episodes = 1000
    max_steps_per_episode = 100
    epsilon_decay = 0.995
    min_epsilon = 0.01

    print("\n--- Starting Training ---")
    for episode in range(episodes):
        agent.reset()

        for step in range(max_steps_per_episode):
            state = agent.get_state()
            action = agent.choose_action()

            # Store position before move
            old_x, old_y = agent.x, agent.y

            agent.move(action)

            next_state = agent.get_state()

            # Determine reward
            reward = 0
            if next_state == game_map.goal_pos:
                reward = 10  # High reward for reaching the goal
            elif state == next_state:
                reward = -5 # Penalty for bumping into a wall/obstacle
            else:
                reward = -0.1  # Small penalty for each step

            agent.update_q_table(state, action, reward, next_state)

            if next_state == game_map.goal_pos:
                break # End episode if goal is reached

        # Decay epsilon
        if agent.epsilon > min_epsilon:
            agent.epsilon *= epsilon_decay

        if (episode + 1) % 100 == 0:
            print(f"Episode {episode + 1}/{episodes} | Epsilon: {agent.epsilon:.3f}")

    print("--- Training Finished ---")

    # --- 4. Display results ---
    print("\nAgent's learned path (greedy policy):")
    agent.reset()
    agent.epsilon = 0 # Turn off exploration
    path = []
    for _ in range(max_steps_per_episode):
        state = agent.get_state()
        path.append(state)
        if state == game_map.goal_pos:
            break
        action = agent.choose_action()
        agent.move(action)

    # Print the map with the path
    for y, row in enumerate(game_map.grid):
        display_row = []
        for x, tile in enumerate(row):
            if (x, y) == (agent.initial_x, agent.initial_y):
                display_row.append('S')
            elif (x,y) in path and game_map.grid[y][x] not in ['S', 'G']:
                display_row.append('*')
            else:
                display_row.append(tile)
        print(' '.join(display_row))
    print(f"\nPath taken: {path}")


if __name__ == "__main__":
    main()
