# Cursor Configuration for rank-* Workspace

## Overview

**Important**: Cursor discovers rules **per-repository**, not from shared directories. This directory contains a **template** that should be copied to each rank-* repository.

## Architecture

Cursor uses a **hierarchical, per-repository rule system**:

1. **User Rules** (Global) - Personal preferences across all projects
2. **Project Rules** (Per-Repo) - Stored in `.cursor/rules/*.mdc` files in each repository
3. **Legacy `.cursorrules`** (Deprecated) - Still supported but not recommended

## Current Setup

```
rank-rank/
├── .cursor/
│   └── rules/
│       ├── shared-base.mdc    # Template for shared rules
│       └── README.md          # This file
└── CURSOR_CONFIG.md           # This documentation
```

## How It Works

### Rule Discovery

Cursor **automatically discovers** rules when you open a repository by:
1. Scanning for `.cursor/rules/` directories within the repository
2. Indexing `.mdc` files based on their configuration
3. Applying rules based on `alwaysApply`, `globs`, and manual `@ruleName` references

### Important Limitation

**Cursor does NOT discover rules from shared directories outside the repository.** If you put rules in `rank-rank/.cursor/rules/`, Cursor will NOT automatically use them when working in `rank-fusion/`.

## Recommended Approach

### Option 1: Copy Template to Each Repo (Recommended)

Each rank-* repository should have its own `.cursor/rules/` directory:

```bash
# For each rank-* repo:
cp -r rank-rank/.cursor/rules rank-fusion/.cursor/rules
cp -r rank-rank/.cursor/rules rank-refine/.cursor/rules
# etc.
```

**Pros:**
- ✅ Cursor automatically discovers rules
- ✅ Rules version-controlled with each repo
- ✅ Each repo can customize independently
- ✅ Follows Cursor's intended architecture

**Cons:**
- ⚠️ Need to update multiple repos when shared rules change
- ⚠️ Slight duplication

### Option 2: Symlink (Not Recommended)

You could symlink, but this breaks when repos are cloned separately:

```bash
# Don't do this - breaks when repos are separate
ln -s ../../rank-rank/.cursor/rules rank-fusion/.cursor/rules
```

### Option 3: Manual Reference (For Specific Cases)

In repo-specific rules, you can manually reference shared patterns:

```markdown
---
title: "Fusion-Specific Rules"
id: fusion-rules
---

# Reference shared patterns from rank-rank
When creating visualizations, follow patterns in rank-rank/hack/viz/

# Fusion-specific rules
- RRF k parameter should default to 60
- Always validate eval_results.json structure
```

## Setup Instructions

### Initial Setup

1. **Copy template to each repository:**
   ```bash
   for repo in rank-fusion rank-refine rank-relax rank-eval; do
     mkdir -p "$repo/.cursor/rules"
     cp rank-rank/.cursor/rules/shared-base.mdc "$repo/.cursor/rules/"
   done
   ```

2. **Add repo-specific rules** in each repo's `.cursor/rules/repo-specific.mdc`

3. **Commit rules** to each repository

### Updating Shared Rules

When updating shared rules:

1. Update `rank-rank/.cursor/rules/shared-base.mdc` (the template)
2. Copy updated template to each rank-* repository
3. Commit changes to each repo

**Future**: Consider a script to automate this sync.

## Rule File Structure

Each rule file uses `.mdc` format with frontmatter:

```markdown
---
title: "Rule Title"
id: unique-rule-id
description: "What this rule does"
priority: 100              # Higher = more important
alwaysApply: true          # Auto-include in all contexts
globs: "**/*.rs"           # Optional: auto-attach for matching files
---

# Rule content (markdown)
```

## Rule Precedence

Cursor applies rules in this order:

1. **Local (manual)**: Rules explicitly included with `@ruleName`
2. **Auto Attached**: Rules with `globs` matching current files
3. **Agent Requested**: Rules the AI chooses to include
4. **Always**: Rules with `alwaysApply: true`

## Best Practices

### Keep Rules Focused

- Maximum ~500 lines per rule file
- Split by purpose: base rules, language-specific, security, etc.
- Use multiple focused files rather than one large file

### Use Appropriate Attachment

- `alwaysApply: true` for fundamental rules (coding standards, error handling)
- `globs: "**/*.rs"` for Rust-specific rules
- `globs: "**/*.py"` for Python-specific rules
- Manual `@ruleName` for context-specific rules

### Document Why

- Explain the reasoning behind rules
- Link to relevant documentation
- Note exceptions or special cases

### Version Control

- Commit `.cursor/rules/` to each repository
- Review rule changes in PRs
- Track rule evolution alongside code

## Example Repository Structure

```
rank-fusion/
├── .cursor/
│   ├── rules/
│   │   ├── shared-base.mdc      # Copied from rank-rank
│   │   ├── fusion-specific.mdc # Repo-specific rules
│   │   └── rust-rules.mdc      # Rust-specific (globs: "**/*.rs")
│   └── docs/
│       └── project-context.md
└── src/
```

## Migration from .cursorrules

If you have existing `.cursorrules` files:

1. Create `.cursor/rules/` directory
2. Convert `.cursorrules` content to `.mdc` format with frontmatter
3. Add `alwaysApply: true` if it should always be included
4. Keep `.cursorrules` temporarily for backward compatibility
5. Remove `.cursorrules` once `.mdc` rules are working

## Troubleshooting

### Rules Not Being Applied

1. **Check rule location**: Must be in `.cursor/rules/*.mdc` within the repository
2. **Check frontmatter**: Must have valid YAML frontmatter
3. **Check `alwaysApply`**: Set to `true` if you want automatic inclusion
4. **Check `.cursorignore`**: Ensure rules directory isn't ignored

### Rules from rank-rank Not Working

**This is expected!** Cursor doesn't discover rules from shared directories. You must copy rules to each repository.

## See Also

- [Cursor Rules Documentation](https://cursor.com/docs/context/rules)
- [Cursor Best Practices](https://github.com/digitalchild/cursor-best-practices)
- [rank-rank/README.md](README.md) - Overview of rank-rank
- [rank-rank/USAGE.md](USAGE.md) - Usage guide
