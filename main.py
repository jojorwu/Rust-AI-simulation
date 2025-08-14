import sys
import os

# This allows us to import from the src directory
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), 'src')))

from game.game import Game

def main():
    """
    Main entry point for the simulation.
    Initializes and runs the game.
    """
    game = Game()
    game.run()

if __name__ == "__main__":
    main()
