#!/usr/bin/env bash
# Archive duplicate status/completion documents to archive/ directory

set -e

ARCHIVE_DIR="archive/2025-01/status_docs"
mkdir -p "$ARCHIVE_DIR"

# Find and archive status documents
find crates -name "*STATUS*.md" -o -name "*COMPLETE*.md" -o -name "*DONE*.md" -o -name "*FINAL*.md" | \
    grep -v target | grep -v ".venv" | grep -v "$ARCHIVE_DIR" | \
    while read file; do
        # Skip if already in archive
        if [[ "$file" == archive/* ]]; then
            continue
        fi
        
        # Get relative path and create archive structure
        rel_path="${file#crates/}"
        crate_dir="$(dirname "$rel_path")"
        filename="$(basename "$file")"
        
        # Create archive directory structure
        archive_path="$ARCHIVE_DIR/$crate_dir"
        mkdir -p "$archive_path"
        
        # Move file to archive
        echo "Archiving: $file -> $archive_path/$filename"
        mv "$file" "$archive_path/$filename"
    done

echo "✅ Status documents archived to $ARCHIVE_DIR"
