import json
import os

def save_q_table(q_table, filename="q_table.json"):
    """Saves the Q-table to a JSON file."""
    # Convert tuple keys to strings
    q_table_str_keys = {str(k): v for k, v in q_table.items()}
    with open(filename, 'w') as f:
        json.dump(q_table_str_keys, f, indent=4)
    print(f"Q-table saved to {filename}")

def load_q_table(filename="q_table.json"):
    """Loads the Q-table from a JSON file."""
    if not os.path.exists(filename):
        print("No Q-table found, starting fresh.")
        return {}
    with open(filename, 'r') as f:
        q_table_str_keys = json.load(f)
        # Convert string keys back to tuples
        q_table = {eval(k): v for k, v in q_table_str_keys.items()}
        print(f"Q-table loaded from {filename}")
        return q_table
