# Cursor Rules Setup Complete ✅

## Summary

Shared Cursor rules have been successfully set up for all rank-* repositories.

## What Was Done

1. **Created template structure** in `rank-rank/.cursor/rules/`:
   - `shared-base.mdc` - Shared rules with proper frontmatter
   - `README.md` - Documentation for rule structure

2. **Created sync script** (`scripts/sync_cursor_rules.sh`):
   - Automatically copies rules to all rank-* repositories
   - Creates `.cursor/rules/` directories as needed
   - Preserves repo-specific rules

3. **Created verification script** (`scripts/verify_cursor_rules.sh`):
   - Validates rule setup in all repositories
   - Checks for proper frontmatter
   - Reports missing or misconfigured rules

4. **Added repo-specific rules** to each repository:
   - `rank-fusion/.cursor/rules/fusion-specific.mdc`
   - `rank-refine/.cursor/rules/refine-specific.mdc`
   - `rank-relax/.cursor/rules/relax-specific.mdc`
   - `rank-eval/.cursor/rules/eval-specific.mdc`

## Current Structure

```
rank-rank/
├── .cursor/
│   └── rules/
│       ├── shared-base.mdc    # Template (source of truth)
│       └── README.md
├── scripts/
│   ├── sync_cursor_rules.sh   # Sync template to repos
│   └── verify_cursor_rules.sh # Verify setup
└── CURSOR_CONFIG.md            # Complete documentation

rank-fusion/
├── .cursor/
│   └── rules/
│       ├── shared-base.mdc    # Synced from rank-rank
│       └── fusion-specific.mdc # Repo-specific
└── ...

rank-refine/
├── .cursor/
│   └── rules/
│       ├── shared-base.mdc
│       └── refine-specific.mdc
└── ...

# (same for rank-relax and rank-eval)
```

## How It Works

### Rule Discovery

Cursor automatically discovers rules when you open a repository:
1. Scans for `.cursor/rules/*.mdc` files
2. Reads frontmatter to determine when to apply
3. Applies rules based on `alwaysApply`, `globs`, or manual `@ruleName`

### Rule Precedence

1. **Local (manual)**: `@ruleName` explicitly included
2. **Auto-attached**: Rules with `globs` matching current files
3. **Agent-requested**: Rules AI chooses to include
4. **Always**: Rules with `alwaysApply: true`

### Shared vs Repo-Specific

- **Shared rules** (`shared-base.mdc`): Common standards across all repos
- **Repo-specific rules**: Unique to each repository's needs
- Both are automatically discovered and applied

## Maintenance

### Updating Shared Rules

1. Edit `rank-rank/.cursor/rules/shared-base.mdc`
2. Run sync script:
   ```bash
   cd rank-rank
   ./scripts/sync_cursor_rules.sh
   ```
3. Commit changes to each repository

### Adding New Repositories

1. Copy rules template:
   ```bash
   cd rank-rank
   ./scripts/sync_cursor_rules.sh
   ```
2. Add repo-specific rules in `.cursor/rules/repo-specific.mdc`
3. Verify setup:
   ```bash
   ./scripts/verify_cursor_rules.sh
   ```

### Verifying Setup

Run the verification script periodically:

```bash
cd rank-rank
./scripts/verify_cursor_rules.sh
```

## Best Practices

1. **Keep rules focused**: Max ~500 lines per file
2. **Use appropriate attachment**:
   - `alwaysApply: true` for fundamental rules
   - `globs: "**/*.rs"` for Rust-specific rules
   - Manual `@ruleName` for context-specific rules
3. **Document why**: Explain reasoning behind rules
4. **Version control**: Commit `.cursor/rules/` to each repo
5. **Review changes**: Treat rules like code, review in PRs

## Troubleshooting

### Rules Not Being Applied

1. Check rule location: Must be in `.cursor/rules/*.mdc` within repository
2. Check frontmatter: Must have valid YAML frontmatter
3. Check `alwaysApply`: Set to `true` for automatic inclusion
4. Check `.cursorignore`: Ensure rules directory isn't ignored

### Sync Issues

1. Run verification: `./scripts/verify_cursor_rules.sh`
2. Re-sync if needed: `./scripts/sync_cursor_rules.sh`
3. Check file permissions: Rules should be readable

## Next Steps

1. ✅ Rules synced to all repositories
2. ✅ Repo-specific rules added
3. ✅ Verification script confirms setup
4. 🔄 Test in Cursor IDE to confirm rules are discovered
5. 🔄 Update rules as needed based on usage

## See Also

- [CURSOR_CONFIG.md](CURSOR_CONFIG.md) - Complete configuration guide
- [Cursor Rules Documentation](https://cursor.com/docs/context/rules)
- [rank-rank/README.md](README.md) - Overview of rank-rank

