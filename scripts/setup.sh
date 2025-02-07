#!/bin/bash

# Create required directories
mkdir -p database logs backups

# Set appropriate permissions
chmod 755 database logs backups

echo "Created required directories:"
echo "  - database/"
echo "  - logs/"
echo "  - backups/" 