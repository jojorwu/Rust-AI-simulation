# Rust Simulation

This project is a simulation of a simple world with agents that can gather resources, craft items, and build structures. The simulation is built using the Bevy game engine and its Entity-Component-System (ECS) architecture.

## Building and Running

A unified script is provided to simplify the process of building, packaging, and running the application across different operating systems. The script will attempt to guide you through installing any missing dependencies, such as the Rust toolchain or necessary system libraries.

### Quick Start

-   **On Linux or macOS:**
    Open your terminal, navigate to the project's root directory, and run:
    ```bash
    ./run.sh
    ```

-   **On Windows:**
    Simply double-click the `run-windows.bat` file. This will launch a PowerShell script to handle the setup process.

The script will handle the following steps for you:
1.  **Dependency Check:** It will check if you have the Rust toolchain installed and guide you through the installation process if you don't. On Linux, it will also check for common system dependencies.
2.  **Build:** It will compile the application in release mode. This might take a few minutes on the first run.
3.  **Package:** It will package the game into a `dist` folder, specific to your operating system.
4.  **Launch:** It will launch the application automatically.

### Script Features

-   **OS Version Detection:** The script begins by detecting and displaying your specific OS version, which can be helpful for debugging.
-   **Interactive Menu:** If you have an existing build, the script will present a menu with the following options:
    -   **Launch existing version:** Skips the build process and runs the app immediately.
    -   **Rebuild the application:** Deletes the old build and creates a new one from the current source code.
    -   **Check for Updates and Rebuild:** If you cloned the project using Git, this option will run `git pull` to fetch the latest changes from the repository and then automatically start the rebuild process. If not installed via Git, it will advise you to update manually.

### Non-Interactive Mode / Automation

For power users or automated environments, the script can be run non-interactively using command-line flags.

**On Linux/macOS:**
```bash
# Build the project but do not run it
./run.sh --build

# Perform a clean build and then run the application
./run.sh --clean --run

# Run the test suite
./run.sh --test
```

**On Windows (from PowerShell or `cmd`):**
```powershell
# Build the project but do not run it
./run-windows.bat -Build

# Perform a clean build and then run the application
./run-windows.bat -Clean -Run

# Run the test suite
./run-windows.bat -Test
```

Available flags:
-   `--build` / `-Build`: Build the application.
-   `--run` / `-Run`: Run the application after building. Implies build.
-   `--test` / `-Test`: Run the project's test suite.
-   `--clean` / `-Clean`: Perform a clean build.
-   `--help` / `-Help`: Display a help message.

### Manual Development

If you are a developer and have already installed Rust and all the required dependencies, you can run the simulation directly with Cargo:
```bash
cd rust_simulation
cargo run
```

## Project Structure

The project is organized as follows:

- `src/`: Contains the main source code for the simulation.
  - `main.rs`: The entry point of the application. It sets up the Bevy app and plugins.
  - `lib.rs`: The main library file, which defines the simulation's core logic, components, systems, and resources.
  - `components/`: Defines the ECS components used in the simulation.
  - `systems/`: Defines the ECS systems that implement the simulation's logic.
- `data/`: Contains JSON data files for items, recipes, biomes, etc.
- `benches/`: Contains criterion benchmarks.
- `tests/`: Contains integration tests.
- `run.sh`: Unified launcher script for Linux and macOS.
- `run-windows.bat`: Wrapper script for Windows users.
- `setup-windows.ps1`: The main PowerShell logic for the Windows setup.
