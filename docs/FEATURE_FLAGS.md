# Feature Flags Guide

This document describes the feature flags used across the `rank-rank` workspace and best practices for their usage.

## Principles

1. **Minimal Defaults**: All crates use `default = []` to keep builds minimal
2. **Additive Features**: Features are additive - enabling a feature never breaks existing code
3. **Explicit Dependencies**: Use `dep:` syntax for optional dependencies
4. **Logical Grouping**: Related features are grouped together
5. **Documentation**: Features are documented in Cargo.toml and code

## rank-rerank

### Core Features

- **`default`**: Empty (minimal build)
- **`hierarchical`**: Enables hierarchical clustering with `kodama` crate
- **`serde`**: Enables serialization support for data structures
- **`wasm`**: Enables WebAssembly bindings for browser/Node.js usage

### ML Framework Integration

- **`candle`**: Enables Candle tensor operations for GPU-accelerated MaxSim
  - Adds: `candle-core`
  - Use: GPU-accelerated similarity computations

### Cross-Encoder Features

- **`crossencoder`**: Enables tokenization support for cross-encoders
  - Adds: `tokenizers` crate
  - Use: When you need proper BERT-style tokenization
  - **Independent**: Can be used without `ort`

- **`ort`**: Enables ONNX Runtime integration for cross-encoder inference
  - Adds: `ort`, `tokenizers` (via `crossencoder` dependency)
  - Use: When you have ONNX models and want native Rust inference
  - **Note**: Includes tokenization support automatically

### Feature Combinations

```toml
# Minimal build (default)
rank-rerank = { path = "..." }

# With tokenization (no ONNX)
rank-rerank = { path = "...", features = ["crossencoder"] }

# With ONNX Runtime (includes tokenization)
rank-rerank = { path = "...", features = ["ort"] }

# GPU acceleration
rank-rerank = { path = "...", features = ["candle"] }

# Full feature set
rank-rerank = { path = "...", features = ["ort", "candle", "serde"] }
```

## rank-soft

### Core Features

- **`default`**: Empty (minimal build)

### ML Framework Integration

- **`candle`**: Enables Candle tensor operations
  - Adds: `candle-core`
  - Use: GPU-accelerated soft ranking with Candle

- **`burn`**: Enables Burn tensor operations for multi-backend support
  - Adds: `burn`, `burn-tensor`
  - Use: When you need CUDA/Metal/Vulkan/WebGPU support via Burn
  - **Note**: Can be used alongside `candle` (different frameworks)

### Performance Features

- **`parallel`**: Enables parallel computation with Rayon
  - Adds: `rayon`
  - Use: For CPU-bound operations on large datasets

- **`gumbel`**: Enables Gumbel-Softmax and relaxed top-k methods
  - Adds: `rand`
  - Use: For stochastic ranking operations

### Feature Combinations

```toml
# Minimal build (default)
rank-soft = { path = "..." }

# With Candle (GPU support)
rank-soft = { path = "...", features = ["candle"] }

# With Burn (multi-backend)
rank-soft = { path = "...", features = ["burn"] }

# With parallel processing
rank-soft = { path = "...", features = ["parallel"] }

# Full feature set
rank-soft = { path = "...", features = ["candle", "burn", "parallel", "gumbel"] }
```

## Python Bindings

### rank-rerank-python

- **`default`**: Empty
- **`crossencoder`**: Enables cross-encoder tokenization in Python bindings
  - Requires: `rank-rerank/crossencoder`
- **`ort`**: Enables ONNX Runtime in Python bindings
  - Requires: `rank-rerank/ort`, `crossencoder`
  - **Note**: Automatically includes `crossencoder` for convenience

### rank-soft-python

- **`default`**: Empty
- **`pytorch`**: Placeholder for future PyTorch integration
  - Currently: No-op (for future use)

## Best Practices

### For Library Authors

1. **Always use `dep:` syntax**:
   ```toml
   ort = { version = "2.0", optional = true }
   [features]
   ort = ["dep:ort"]  # ✅ Good
   ort = ["ort"]      # ❌ Bad (ambiguous)
   ```

2. **Document features in Cargo.toml**:
   ```toml
   [features]
   # ONNX Runtime integration for cross-encoder inference
   ort = ["dep:ort", "dep:tokenizers"]
   ```

3. **Use `#[cfg(feature = "...")]` consistently**:
   ```rust
   #[cfg(feature = "ort")]
   pub mod ort;
   ```

4. **Group related features**:
   ```toml
   # ML Framework Integration
   candle = ["dep:candle-core"]
   burn = ["dep:burn", "dep:burn-tensor"]
   ```

### For Library Users

1. **Enable only what you need**:
   ```toml
   # ✅ Good: Minimal
   rank-rerank = { path = "...", features = ["candle"] }
   
   # ❌ Bad: Unnecessary features
   rank-rerank = { path = "...", features = ["ort", "candle", "serde", "wasm"] }
   ```

2. **Check feature dependencies**:
   - Some features automatically enable others (e.g., `ort` includes `crossencoder`)
   - Check documentation for feature relationships

3. **Use feature combinations wisely**:
   - `candle` and `burn` can coexist (different frameworks)
   - `ort` and `candle` can coexist (different use cases)

## Migration Guide

### From Old Feature Names

If you were using features before this refactoring:

- No breaking changes - all existing features work the same
- New features are additive only

### Adding New Features

When adding a new feature:

1. Add optional dependency: `dep = { version = "...", optional = true }`
2. Add feature: `feature_name = ["dep:dep"]`
3. Document in Cargo.toml comment
4. Use `#[cfg(feature = "feature_name")]` in code
5. Update this document

