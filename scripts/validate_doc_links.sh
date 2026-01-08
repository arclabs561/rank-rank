#!/usr/bin/env bash
# Validate markdown links in documentation

set -e

ERRORS=0
WARNINGS=0

check_link() {
    local file="$1"
    local link="$2"
    local line_num="$3"
    
    # Extract target (remove anchor if present)
    local target="${link#*](}"
    target="${target%)}"
    local anchor="${target#*#}"
    target="${target%#*}"
    
    # Skip external links
    if [[ "$target" == http* ]] || [[ "$target" == https* ]]; then
        return 0
    fi
    
    # Resolve relative path
    local file_dir="$(dirname "$file")"
    local resolved_target
    
    if [[ "$target" == /* ]]; then
        # Absolute from repo root
        resolved_target="${target#/}"
    elif [[ "$target" == ../* ]]; then
        # Relative to file directory
        resolved_target="$(cd "$file_dir" && realpath --relative-to . "$target" 2>/dev/null || echo "")"
    else
        # Relative to file directory
        resolved_target="$file_dir/$target"
    fi
    
    # Check if file exists
    if [ ! -f "$resolved_target" ] && [ ! -f "crates/$resolved_target" ]; then
        echo "❌ $file:$line_num: Broken link: $link -> $target (resolved: $resolved_target)"
        ((ERRORS++))
        return 1
    fi
    
    # Check anchor if present
    if [ -n "$anchor" ] && [ "$anchor" != "$target" ]; then
        # Note: Anchor validation would require parsing markdown
        # For now, just warn
        echo "⚠️  $file:$line_num: Anchor link (not validated): $link"
        ((WARNINGS++))
    fi
    
    return 0
}

# Find all markdown files
find crates -name "*.md" -type f | grep -v target | grep -v ".venv" | while read file; do
    # Extract links from markdown
    line_num=0
    while IFS= read -r line; do
        ((line_num++))
        # Match markdown links: [text](target) or [text](target#anchor)
        echo "$line" | grep -oE '\[([^\]]+)\]\(([^)]+)\)' | while read -r match; do
            check_link "$file" "$match" "$line_num" || true
        done
    done < "$file"
done

if [ $ERRORS -eq 0 ] && [ $WARNINGS -eq 0 ]; then
    echo "✅ All links validated successfully"
    exit 0
elif [ $ERRORS -eq 0 ]; then
    echo "⚠️  Found $WARNINGS warnings (anchor links not fully validated)"
    exit 0
else
    echo "❌ Found $ERRORS broken links and $WARNINGS warnings"
    exit 1
fi

