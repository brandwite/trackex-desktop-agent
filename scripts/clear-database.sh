#!/bin/bash

echo "Clearing TrackEx Agent Database..."
echo ""

# Determine OS-specific data directory
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    DB_PATH="$HOME/Library/Application Support/TrackEx/agent.db"
else
    # Linux
    DB_PATH="$HOME/.local/share/TrackEx/agent.db"
fi

if [ -f "$DB_PATH" ]; then
    echo "Found database at: $DB_PATH"
    rm "$DB_PATH"
    if [ $? -eq 0 ]; then
        echo "Database deleted successfully!"
    else
        echo "Failed to delete database. Make sure the TrackEx agent is closed."
    fi
else
    echo "Database not found at: $DB_PATH"
    echo "The database may not exist yet or is in a different location."
fi

echo ""
echo "Database reset complete."

