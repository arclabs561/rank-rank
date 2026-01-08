# Remaining Issues and Known Limitations

This document tracks remaining issues, known limitations, and planned improvements.

## Compilation Status

### ✅ Fixed Issues
- **Burn Integration**: All compilation errors resolved
  - Removed unnecessary `f64: ElementConversion` trait bound
  - Using unsafe transmute workaround for numeric type conversion (documented)
  - All feature-flagged code compiles successfully

- **ONNX Runtime**: Type mismatches fixed
  - Fixed `from_file` method signature issues
  - Fixed tokenizer API usage (using default pad token ID)
  - Removed duplicate doc comments

- **Syntax Errors**: Fixed
  - Fixed unclosed delimiter in `contextual.rs` (indentation issue)
  - Fixed missing closing brace in Thompson sampling loop

### ⚠️ Known Limitations

#### Burn Integration
**Status**: Working with workaround
**Issue**: ElementConversion API type inference issues in Burn 0.19
**Current Solution**: Using unsafe transmute for numeric type conversion
**Safety**: Safe for numeric types (f32, f64) - just reinterpreting memory
**Future**: Implement native Burn tensor operations when API stabilizes

**Files**:
- `crates/rank-soft/src/burn.rs`

**Documentation**: 
- `docs/BURN_INTEGRATION_NOTE.md`
- Module-level docs in `burn.rs`

#### ONNX Runtime
**Status**: Stub implementation with fallback
**Issue**: ort crate API needs investigation for proper session creation
**Current Solution**: Placeholder implementation with Jaccard similarity fallback
**Future**: Complete ONNX Runtime inference when API is finalized

**Files**:
- `crates/rank-rerank/src/crossencoder/ort.rs`

**Documentation**:
- Module-level docs explain implementation status
- All TODOs documented

## Code Quality

### Unsafe Code Usage
**Location**: `crates/rank-soft/src/burn.rs`
**Reason**: Workaround for ElementConversion API issues
**Safety**: Documented, safe for numeric type conversion
**Review**: All unsafe blocks have safety comments

### Unwrap/Expect Usage
**Status**: Needs review
**Action**: Review all unwrap/expect usage in production code
**Priority**: Medium

### Documentation
**Status**: Good coverage
**Missing**: Some internal functions could use more detailed doc comments
**Priority**: Low

## Testing

### Current Status
- ✅ Unit tests comprehensive
- ✅ Property-based tests
- ✅ Integration tests
- ⚠️ Performance regression tests needed
- ⚠️ Cross-platform testing needed

## Feature Flags

### Status: ✅ Complete
- All features properly documented
- Consistent `dep:` syntax
- Logical feature grouping
- Minimal default features
- Comprehensive documentation in `docs/FEATURE_FLAGS.md`

## Next Actions

1. **Review unwrap/expect usage**: Audit production code for proper error handling
2. **Performance benchmarks**: Add regression tests for performance-critical paths
3. **Documentation**: Add more detailed function-level documentation where needed
4. **ONNX Runtime**: Monitor ort crate API changes, complete implementation when stable
5. **Burn Integration**: Monitor Burn API changes, implement native operations when possible

