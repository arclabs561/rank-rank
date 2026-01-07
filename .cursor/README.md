# Cursor Configuration for rank-* Workspace

This directory contains shared Cursor configuration that applies to all rank-* repositories.

## Structure

- `.cursorrules` - Shared rules for all rank-* repos
- `.cursor/` - Additional cursor configuration files

## Usage

Cursor will automatically pick up `.cursorrules` from:
1. The workspace root (if opened as workspace)
2. Individual repo roots (repo-specific overrides)
3. rank-rank/.cursorrules (shared rules)

## Shared Rules

The `.cursorrules` file in rank-rank contains:
- Visualization standards
- Error handling requirements
- Documentation standards
- Statistical methods guidelines
- Code quality standards

## Repo-Specific Overrides

Individual rank-* repos can have their own `.cursorrules` that extend or override shared rules.

## Best Practices

- Keep shared rules in rank-rank
- Repo-specific rules in individual repos
- Document any overrides or extensions

