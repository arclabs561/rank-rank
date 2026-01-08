# Release Checklist

Complete checklist for releasing rank-rerank to crates.io.

## Pre-Release Verification

### Code Quality ✅
- [x] All tests passing
- [x] All examples compile
- [x] No compilation errors
- [x] Warnings addressed (only expected feature-gated warnings remain)
- [x] Clippy clean (with `-D warnings`)
- [x] All unsafe blocks documented
- [x] All unwrap/expect justified

### Documentation ✅
- [x] API documentation complete
- [x] All public items documented
- [x] Examples compile and run
- [x] README up-to-date
- [x] CHANGELOG updated
- [x] All documentation links verified

### Testing ✅
- [x] Unit tests comprehensive
- [x] Integration tests complete
- [x] Property tests expanded
- [x] Fuzz testing expanded (5 targets)
- [x] Edge cases covered

### CI/CD ✅
- [x] CI pipelines passing
- [x] SIMD path testing automated
- [x] Performance monitoring set up
- [x] Documentation builds successfully

## Release Steps

### 1. Version Update

```bash
# Update version in Cargo.toml
# Current: 0.7.36
# Next: 0.7.37 (patch) or 0.8.0 (minor/major)
```

**Versioning Guidelines**:
- **Patch (0.7.37)**: Bug fixes, documentation, minor improvements
- **Minor (0.8.0)**: New features, backward-compatible changes
- **Major (1.0.0)**: Breaking changes

### 2. CHANGELOG Update

```bash
# Add entry to CHANGELOG.md
# Include:
# - New features
# - Bug fixes
# - Performance improvements
# - Breaking changes (if any)
```

### 3. Final Verification

```bash
# Run all tests
cargo test --all-features

# Build documentation
cargo doc --no-deps --all-features

# Check examples
cargo build --examples

# Clippy check
cargo clippy --all-features -- -D warnings
```

### 4. Git Tag

```bash
# Create release tag
git tag -a v0.7.36 -m "Release v0.7.36"
git push origin v0.7.36
```

### 5. Publish to crates.io

```bash
# Dry run first
cargo publish --dry-run

# Publish
cargo publish
```

**Note**: Publishing is irreversible. Ensure everything is correct.

### 6. Post-Release

- [ ] Verify package appears on crates.io
- [ ] Check documentation on docs.rs
- [ ] Update any external documentation
- [ ] Announce release (if applicable)

## Current Status

**Version**: 0.7.36  
**Status**: ✅ Ready for release

All pre-release verification items are complete.

## Release Notes Template

```markdown
## [0.7.36] - 2025-01-07

### Added
- AVX-512 support for Zen 5+ and Ice Lake+
- Performance tuning guide
- Troubleshooting guide
- Realistic workload benchmarks
- Expanded property tests
- Fuzz testing expansion (fuzz_explain)

### Changed
- SIMD dispatch prioritizes AVX-512 → AVX2 → NEON → portable
- Improved documentation structure
- Enhanced CI/CD pipelines

### Fixed
- Fixed matryoshka_search example (crate name)
- Fixed comparison warnings in tests
- Improved TODO documentation

### Documentation
- Complete API documentation
- Performance tuning guide
- Troubleshooting guide
- Use cases and examples guide
- API quick reference
```

## Notes

- **Breaking changes**: None in this release
- **Deprecations**: None
- **Minimum Rust version**: 1.74
- **Features**: All stable, no nightly required

