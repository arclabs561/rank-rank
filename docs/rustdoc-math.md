# Math Rendering in Rust Documentation

All rank-* crates support LaTeX math notation in rustdoc comments, rendered via MathJax.

## Usage

Use standard LaTeX math syntax in doc comments:

```rust
/// Compute the dot product: $a \cdot b = \sum_{i=1}^{d} a_i b_i$
///
/// For display math:
///
/// $$\cos(a, b) = \frac{a \cdot b}{\|a\| \cdot \|b\|}$$
pub fn dot_product(a: &[f64], b: &[f64]) -> f64 {
    // ...
}
```

### Syntax

- Inline math: `$...$` or `\(...\)`
- Display math: `$$...$$` or `\[...\]`
- Escaping: Use `\$` for literal dollar signs

## Building Documentation

### Local Builds

For local `cargo doc` builds, use the `RUSTDOCFLAGS` environment variable:

```bash
# From crate directory
cd crates/rank-soft
RUSTDOCFLAGS="--html-in-header ../../rustdoc-header.html" cargo doc --open

# Or from workspace root
RUSTDOCFLAGS="--html-in-header rustdoc-header.html" cargo doc --workspace --open
```

### docs.rs Builds

The `[package.metadata.docs.rs]` section in each crate's `Cargo.toml` automatically includes the math header when documentation is built on docs.rs.

### Configuration

The math rendering is configured in `rustdoc-header.html` at the workspace root. It uses MathJax 3 with:
- Inline delimiters: `$...$` and `\(...\)`
- Display delimiters: `$$...$$` and `\[...\]`
- Automatic escaping enabled

## Examples

See existing math usage in:
- `crates/rank-soft/docs/MATHEMATICAL_DETAILS.md`
- `crates/rank-rerank/docs/REFERENCE.md`
- `crates/rank-soft/src/lib.rs` (doc comments)

