# Typst Documentation Generation

This directory contains scripts and templates for generating PDF and HTML documentation from Typst source files for all rank-* repositories.

## Overview

Each rank-* repository has a `docs/main.typ` file that contains the Typst source for generating documentation. The build system converts these to both PDF and HTML formats.

## Quick Start

```bash
# Generate docs for all repos
cd rank-rank
./scripts/generate_typst_docs.sh

# Generate docs for specific repo
./scripts/generate_typst_docs.sh rank-relax
```

## Requirements

1. **Typst**: Install with `cargo install typst-cli` or `brew install typst`
2. **Python 3.8+**: For HTML conversion script

## Output

Generated documentation is placed in each repo's `docs/output/` directory:

- `{repo}_documentation.pdf` - PDF version
- `{repo}_documentation.html` - HTML version

## Typst Source Files

Each repository has a `docs/main.typ` file:

- `rank-fusion/docs/main.typ` - Rank fusion algorithms
- `rank-refine/docs/main.typ` - Late interaction scoring
- `rank-relax/docs/main.typ` - Differentiable ranking
- `rank-eval/docs/main.typ` - Evaluation metrics

## HTML Conversion

The HTML conversion script (`scripts/typst_to_html.py`) extracts content from Typst source and generates styled HTML. It handles:

- Headings and sections
- Code blocks
- Inline code
- Math expressions (simplified)
- Bold/italic text

## Customization

To customize documentation:

1. Edit the `docs/main.typ` file in each repository
2. Adjust styling in the `#set` directives at the top
3. Regenerate with `./scripts/generate_typst_docs.sh`

## Integration

The documentation generation can be integrated into CI/CD:

```yaml
- name: Generate Typst Docs
  run: |
    cd rank-rank
    ./scripts/generate_typst_docs.sh
  env:
    TYPST_FONT_PATHS: /path/to/fonts
```

## Fonts

Typst uses "Linux Libertine" by default. To use custom fonts:

1. Set `TYPST_FONT_PATHS` environment variable
2. Or modify the `#set text(font: ...)` directive in each `main.typ`

## See Also

- [Typst Documentation](https://typst.app/docs/)
- Individual repo READMEs for content details

