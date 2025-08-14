import random

class Agent:
    """
    Represents the AI agent that learns and makes decisions.
    """
    def __init__(self, actions, learning_rate=0.1, discount_factor=0.9, epsilon=1.0):
        self.actions = actions
        self.learning_rate = learning_rate
        self.discount_factor = discount_factor
        self.epsilon = epsilon
        self.q_table = {}

    def choose_action(self, state):
        """Chooses an action using an epsilon-greedy policy."""
        if random.random() < self.epsilon:
            return random.choice(self.actions)  # Explore
        else:
            # If state not in q_table, choose randomly
            if state not in self.q_table:
                 return random.choice(self.actions)
            q_values = self.q_table.get(state, {action: 0 for action in self.actions})
            return max(q_values, key=q_values.get) # Exploit

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
