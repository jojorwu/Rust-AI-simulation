import random

class Player:
    """
    Represents the player in the simulation with Q-learning capabilities.
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
        self.actions = ['up', 'down', 'left', 'right', 'gather']
        self.inventory = {}

    def get_state(self):
        """Returns the current state of the player (its position)."""
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
        """Resets the player's position and inventory."""
        self.x = self.initial_x
        self.y = self.initial_y
        self.inventory = {}
