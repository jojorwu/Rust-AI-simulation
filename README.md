# Rust Simulation

This project is a simulation of a simple world with agents that can gather resources, craft items, and build structures. The simulation is built using the Bevy game engine and its Entity-Component-System (ECS) architecture.

## Building and Running

To build and run the simulation, you need to have Rust and Cargo installed. You also need to install a few system dependencies.

### Dependencies

On Debian-based Linux distributions, you can install the required dependencies with the following command:

```bash
sudo apt-get update && sudo apt-get install -y libasound2-dev libudev-dev
```

### Building

To build the project, run the following command in the project's root directory:

```bash
cargo build
```

### Running

To run the simulation, use the following command:

```bash
cargo run
```

### Running on Windows

For Windows users, a convenience script is provided to build and package the application.

1.  Make sure you have Rust and Cargo installed.
2.  Navigate to the `rust_simulation` directory.
3.  Run the `run.bat` script by double-clicking it or by running it from the command prompt.

This script will compile the application and create a `dist` folder inside the `rust_simulation` directory. This folder will contain the `rust_simulation.exe` executable and the required `data` folder. You can then run the application from the `dist` folder.

## Project Structure

The project is organized as follows:

- `src/`: Contains the main source code for the simulation.
  - `main.rs`: The entry point of the application. It sets up the Bevy app and plugins.
  - `lib.rs`: The main library file, which defines the simulation's core logic, components, systems, and resources.
  - `components/`: Defines the ECS components used in the simulation.
  - `systems/`: Defines the ECS systems that implement the simulation's logic.
  - `map/`: Contains the logic for the game map and procedural generation.
  - `pathfinding/`: Contains the pathfinding logic.
  - `ai/`: Contains the AI logic for the agents.
- `data/`: Contains JSON data files for items, recipes, biomes, etc.
- `benches/`: Contains criterion benchmarks.
- `tests/`: Contains integration tests.
