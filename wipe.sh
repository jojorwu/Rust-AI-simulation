#!/bin/bash

# Get the directory where this script is located to ensure we delete from the correct project root.
SCRIPT_DIR="$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"

echo "Wiping simulation state from '$SCRIPT_DIR'..."
rm -rf "$SCRIPT_DIR/models"
rm -f "$SCRIPT_DIR/q_table.json"
rm -f "$SCRIPT_DIR/simulation_output.log"
rm -rf "$SCRIPT_DIR/rust_simulation/target"
echo "Wipe complete."
