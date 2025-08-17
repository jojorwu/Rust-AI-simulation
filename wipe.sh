#!/bin/bash
echo "Wiping simulation state..."
rm -rf models
rm -f q_table.json
rm -f simulation_output.log
rm -rf rust_simulation/target
echo "Wipe complete."
