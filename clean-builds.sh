#!/bin/bash
# Flareup Build Cache Cleanup Script

set -e

cd ~/scratch/flareup

echo "================================"
echo "Flareup Build Cache Cleanup"
echo "================================"
echo ""

# Show current sizes
echo "Current build cache sizes:"
du -sh src-tauri/target/debug 2>/dev/null || echo "  debug: not found"
du -sh src-tauri/target/release 2>/dev/null || echo "  release: not found"
du -sh src-tauri/target/release-fast 2>/dev/null || echo "  release-fast: not found"
echo ""
du -sh src-tauri/target 2>/dev/null
echo ""

# Ask what to clean
echo "What would you like to clean?"
echo ""
echo "1) Debug builds only (21GB) - Recommended for active development"
echo "2) Release builds only (5.1GB + 4.5GB)"
echo "3) Everything (31GB) - Clean slate"
echo "4) Cancel"
echo ""
read -p "Choose option (1-4): " choice

case $choice in
    1)
        echo ""
        echo "Cleaning debug builds (21GB)..."
        rm -rf src-tauri/target/debug
        echo "✓ Debug builds removed"
        ;;
    2)
        echo ""
        echo "Cleaning release builds (9.6GB)..."
        rm -rf src-tauri/target/release
        rm -rf src-tauri/target/release-fast
        echo "✓ Release builds removed"
        ;;
    3)
        echo ""
        echo "Cleaning all builds (31GB)..."
        # Using cargo clean is cleaner than rm -rf
        cd src-tauri
        cargo clean
        cd ..
        echo "✓ All builds removed"
        ;;
    4)
        echo "Cancelled"
        exit 0
        ;;
    *)
        echo "Invalid option"
        exit 1
        ;;
esac

echo ""
echo "After cleanup:"
du -sh src-tauri/target 2>/dev/null || echo "  target: cleaned completely"
echo ""
echo "To rebuild:"
echo "  Development: cargo build (or just run the app)"
echo "  Release: cargo build --release"
echo ""
