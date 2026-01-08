# Changelog Update Notes

## Version 0.7.36 (Current)

### Added
- **AVX-512 support** - Full 512-bit SIMD acceleration for Zen 5+ and Ice Lake+
  - ~2x performance improvement over AVX2
  - Runtime feature detection
  - Manual horizontal sum reduction (no `_mm512_reduce_add_ps` dependency)
- **Performance tuning guide** - Complete optimization guide (`docs/PERFORMANCE_TUNING.md`)
- **Troubleshooting guide** - Common issues and solutions (`docs/TROUBLESHOOTING.md`)
- **Realistic workload benchmarks** - Real-world RAG pipeline patterns
- **Expanded property tests** - Comprehensive edge case coverage
- **AVX-512 testing infrastructure** - Manual testing script and AWS m8a guide
- **Performance monitoring** - CI workflow for continuous benchmarking
- **Fuzz testing expansion** - Added `fuzz_explain` target

### Changed
- **SIMD dispatch strategy** - Prioritizes AVX-512 → AVX2 → NEON → portable
- **Documentation structure** - Consolidated SIMD docs, added navigation
- **CI improvements** - SIMD path testing, performance monitoring

### Fixed
- Fixed `matryoshka_search` example (was using `rank_refine` instead of `rank_rerank`)
- Fixed comparison warnings in integration tests
- Improved TODO documentation in `crossencoder/ort.rs`

### Documentation
- Created unified `SIMD.md` guide
- Added `ROADMAP.md` with phased forward plan
- Added `NEXT_STEPS.md` and `PRODUCTION_READINESS.md`
- Consolidated SIMD strategy and testing guides

## Next Release (0.7.37 or 0.8.0)

Consider bumping to 0.8.0 if:
- Breaking API changes
- Major feature additions
- Significant performance improvements

Consider 0.7.37 if:
- Bug fixes only
- Documentation improvements
- Minor feature additions

