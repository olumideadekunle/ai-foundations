#!/bin/bash
set -e

# Spawn Bitcoind
docker compose up -d
sleep 15 

echo "Waiting for bitcoind to be fully initialized..."
# (Keep your existing while-loop check here)

# Run your Rust project
chmod +x ./rust/run-rust.sh
./rust/run-rust.sh

# DEBUG: Print the content to the log so we can see why the test fails
echo "--- BEGIN DEBUG: CONTENT OF OUT.TXT ---"
if [ -f "out.txt" ]; then
    cat out.txt
else
    # Check if it was created in the rust subdirectory instead
    if [ -f "./rust/out.txt" ]; then
        cat ./rust/out.txt
    else
        echo "ERROR: out.txt not found in root or ./rust/ folder"
    fi
fi
echo "--- END DEBUG: CONTENT OF OUT.TXT ---"

# Clean up
docker compose down -v
