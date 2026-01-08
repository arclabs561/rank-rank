# Typst Debugging Guide

## Issue: "unclosed delimiter" errors in list items

### Root Cause

Typst's parser has issues when list items contain certain keywords or patterns that appear in math expressions above them. Specifically:

1. **"in" keyword conflict**: When a math expression contains `sum_(r in R)` or similar, list items containing the word "in" ANYWHERE (even in phrases like "in ranking" or "available in") trigger "unclosed delimiter" errors. This is a parser bug where Typst incorrectly associates the "in" in the list item with the math expression's "in" keyword.

2. **Equals sign**: The `=` character in list items (e.g., `k = 60`) can cause issues, especially when followed by a colon.

3. **Colon after certain patterns**: Colons in list items can cause issues in some contexts.

### Solutions

#### 1. Avoid "in" keyword after math expressions

**Problem:**
```typst
$ sum_(r in R) $

where:
* rank of d in r  // ❌ Error: unclosed delimiter
```

**Solution:**
```typst
$ sum_(r in R) $

#v(0.5em)
where:
* rank of d for r  // ✅ Works
* rank of d within r  // ✅ Alternative
* rank of d for r is the rank of document d within ranking r  // ✅ Use "within" instead of "in"
* Method visualizations are available within hack/viz directory  // ✅ Use "within" instead of "in"
```

#### 2. Replace equals with words

**Problem:**
```typst
* k = 60: Default  // ❌ Error
```

**Solution:**
```typst
* k equals 60: Default  // ✅ Works
* k is 60: Default  // ✅ Alternative
```

#### 3. Replace colons with dashes or rephrase

**Problem:**
```typst
* Fast: Optimized  // ❌ Error in some contexts
```

**Solution:**
```typst
* Fast performance - Optimized  // ✅ Works
* Fast performance, optimized  // ✅ Alternative
```

#### 4. Add vertical spacing after math

Adding `#v(0.5em)` or a blank line between math expressions and lists helps:

```typst
$ formula $

#v(0.5em)
where:
* Item 1
```

### Testing Strategy

Use minimal test files to isolate issues:

```bash
cat > /tmp/test.typ << 'EOF'
#set page(margin: 2cm)
= Test
$ sum_(r in R) $

where:
* rank of d in r
EOF
typst compile /tmp/test.typ /tmp/test.pdf
```

### Common Patterns to Avoid

1. `* variable = value: description` → Use `* variable equals value: description`
2. `* text in list` after math with `in` → Use `* text for list` or `* text within list`
3. `* Label: Description` in some contexts → Use `* Label - Description`

### Best Practices

1. **Add spacing**: Always add `#v(0.5em)` or blank line after math expressions before lists
2. **Rephrase**: Use natural language alternatives to avoid keyword conflicts
3. **Test incrementally**: Test each list item separately to isolate issues
4. **Use dashes**: Prefer dashes over colons for separators in list items

### Fixed Examples

All fixed examples in our Typst files:
- `rank of d for r` instead of `rank of d in r`
- `k equals 60` instead of `k = 60`
- `k from 20 to 40` instead of `k = 20-40`
- `Fast performance - Optimized` instead of `Fast: Optimized`

