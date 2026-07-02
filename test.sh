#!/bin/bash
set -e  # Exit immediately if any command fails

# 1. Spawn Bitcoind
docker compose up -d
sleep 15

echo "Waiting for bitcoind to be fully initialized..."
# ... (Keep your existing while-loop check here) ...

# 2. Run your Rust project
chmod +x ./rust/run-rust.sh
./rust/run-rust.sh

# 3. DEBUG: Print the file contents so we can see what Rust wrote
echo "--- CONTENT OF OUT.TXT ---"
if [ -f "out.txt" ]; then
    cat out.txt
else
    echo "ERROR: out.txt was not found!"
fi
echo "--- END OF CONTENT ---"

# 4. Clean up
docker compose down -v
