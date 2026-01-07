# Typst Documentation Guide

Guide for creating and maintaining Typst documentation across rank-* repositories.

## Structure

Each repository should have:
- `docs/main.typ` - Typst source file
- `docs/output/` - Generated PDF and HTML (gitignored)

## Typst Basics

### Document Setup

```typst
#set page(margin: (x: 2.5cm, y: 2cm))
#set text(font: "Linux Libertine", size: 11pt)
#set heading(numbering: "1.")
#set par(justify: true, leading: 0.65em)
```

### Headings

```typst
= Level 1 Heading
== Level 2 Heading
=== Level 3 Heading
```

### Code Blocks

```typst
```rust
let x = 5;
```
```

### Math

```typst
$ x = sum_(i=1)^n a_i $
```

### Lists

```typst
* Item 1
* Item 2
* Item 3
```

## Content Guidelines

### From README.md

Extract key sections:
- Introduction
- Features
- Quick Start examples
- Installation
- Usage examples

### From docs/*.md

Include:
- Algorithm explanations
- Mathematical formulations
- Implementation details
- Research connections

### Keep in Sync

When updating README.md or docs/*.md:
1. Update corresponding sections in `docs/main.typ`
2. Regenerate with `./scripts/generate_typst_docs.sh`
3. Verify PDF and HTML output

## Generating Documentation

```bash
# All repos
cd rank-rank
./scripts/generate_typst_docs.sh

# Specific repo
./scripts/generate_typst_docs.sh rank-relax
```

## Output Files

Generated in `docs/output/`:
- `{repo}_documentation.pdf` - For printing/sharing
- `{repo}_documentation.html` - For web viewing

## Styling

Customize in `#set` directives:
- Fonts: `#set text(font: "...")`
- Margins: `#set page(margin: ...)`
- Headings: `#set heading(...)`

## Math Rendering

Typst supports LaTeX-like math:
- Inline: `$ x = 5 $`
- Display: `$ x = sum_(i=1)^n a_i $`

## Best Practices

1. **Keep it concise**: PDFs should be readable, not exhaustive
2. **Focus on key concepts**: Algorithm explanations, formulas, examples
3. **Link to full docs**: Reference README.md and docs/*.md for details
4. **Update regularly**: Keep Typst in sync with markdown sources
5. **Test generation**: Always verify PDF and HTML output

## Troubleshooting

**Problem**: Typst not found
- **Solution**: Install with `cargo install typst-cli` or `brew install typst`

**Problem**: Font not found
- **Solution**: Use system fonts or set `TYPST_FONT_PATHS`

**Problem**: HTML looks wrong
- **Solution**: Check `typst_to_html.py` conversion logic

**Problem**: Math not rendering
- **Solution**: Verify math syntax, use proper delimiters

## Examples

See existing `docs/main.typ` files in:
- `rank-fusion/docs/main.typ`
- `rank-refine/docs/main.typ`
- `rank-relax/docs/main.typ`
- `rank-eval/docs/main.typ`

## Resources

- [Typst Documentation](https://typst.app/docs/)
- [Typst Tutorial](https://typst.app/docs/tutorial/)
- [Typst Reference](https://typst.app/docs/reference/)

