#!/bin/bash
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export LD_LIBRARY_PATH="$SCRIPT_DIR/SoulverWrapper/.build/release:$SCRIPT_DIR/SoulverWrapper/Vendor/SoulverCore-linux"
exec "$@"
