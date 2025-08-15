import json
import os
from datetime import datetime

def save_q_table(q_table):
    """
    Saves the Q-table to a new generation-specific folder.
    """
    models_dir = "models"
    os.makedirs(models_dir, exist_ok=True)

    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    generation_dir = os.path.join(models_dir, f"generation_{timestamp}")
    os.makedirs(generation_dir)

    file_path = os.path.join(generation_dir, "q_table.json")

    q_table_str_keys = {str(k): v for k, v in q_table.items()}
    with open(file_path, 'w') as f:
        json.dump(q_table_str_keys, f, indent=4)
    print(f"Q-table saved to {file_path}")

def load_q_table():
    """
    Loads the Q-table from the latest generation folder.
    """
    models_dir = "models"
    if not os.path.exists(models_dir):
        print("No models directory found, starting fresh.")
        return {}

    generations = [d for d in os.listdir(models_dir) if os.path.isdir(os.path.join(models_dir, d)) and d.startswith("generation_")]
    if not generations:
        print("No saved generations found, starting fresh.")
        return {}

    latest_generation = sorted(generations)[-1]
    file_path = os.path.join(models_dir, latest_generation, "q_table.json")

    if not os.path.exists(file_path):
        print(f"Error: q_table.json not found in latest generation folder: {latest_generation}")
        return {}

    with open(file_path, 'r') as f:
        q_table_str_keys = json.load(f)
        q_table = {eval(k): v for k, v in q_table_str_keys.items()}
        print(f"Q-table loaded from latest generation: {latest_generation}")
        return q_table
