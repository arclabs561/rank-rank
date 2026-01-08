# GPU Acceleration Implementation Plan

## Overview

This document outlines the plan for adding GPU acceleration to the rank-* crates, focusing on Candle integration for encoding and MaxSim operations.

## Current State

- ✅ SIMD acceleration (CPU) - implemented in `rank-rerank`
- ❌ GPU acceleration - not implemented
- ✅ PyO3 optimizations - GIL release for expensive operations

## Research Findings

### Benefits

- **Encoding**: 3x faster with FP16 (Vespa benchmarks)
- **MaxSim**: Can benefit for large batches (100+ documents)
- **Training**: 40% faster with Burn 0.15 (if we add training)

### Implementation Options

1. **Candle Integration** (Recommended)
   - Native GPU support (CUDA, Metal)
   - Framework-agnostic
   - Good Rust ecosystem support
   - ✅ Best for encoding operations

2. **ONNX Runtime GPU Providers**
   - CUDA Execution Provider
   - TensorRT (NVIDIA)
   - ✅ Best for inference with existing ONNX models

3. **Custom CUDA Kernels**
   - Maximum performance
   - Complex to maintain
   - ❌ Not recommended for initial implementation

## Implementation Plan

### Phase 1: Candle Integration for Encoding (Week 1-2)

#### Dependencies

```toml
[dependencies]
candle-core = { version = "0.4", optional = true, features = ["cuda", "metal"] }
candle-nn = { version = "0.4", optional = true }
candle-transformers = { version = "0.4", optional = true }
tokenizers = "0.15"  # Already added
```

#### Features

```toml
[features]
default = []
gpu = ["dep:candle-core", "dep:candle-nn", "dep:candle-transformers"]
gpu-cuda = ["gpu", "candle-core/cuda"]
gpu-metal = ["gpu", "candle-core/metal"]
```

#### Tasks

1. **Device Detection**
   - Create `gpu/mod.rs` module
   - Detect available GPU (CUDA, Metal, CPU fallback)
   - Device selection API

2. **GPU-Accelerated Encoding**
   - Create `CandleEncoder` struct
   - Load models from HuggingFace or local
   - FP16 support for faster inference
   - Batch encoding support

3. **Integration Points**
   - Cross-encoder encoding
   - ColBERT token encoding
   - Fallback to CPU if GPU unavailable

#### Files to Create

- `crates/rank-rerank/src/gpu/mod.rs` - Device detection
- `crates/rank-rerank/src/gpu/candle_encoder.rs` - Candle-based encoder
- `crates/rank-rerank/src/gpu/candle_crossencoder.rs` - GPU cross-encoder

### Phase 2: GPU-Accelerated MaxSim (Week 3-4)

#### Tasks

1. **GPU MaxSim Implementation**
   - Batch matrix operations on GPU
   - Efficient reduction operations
   - Memory management

2. **Performance Optimization**
   - Batch size tuning
   - Memory pooling
   - Async operations

#### Files to Create

- `crates/rank-rerank/src/gpu/maxsim.rs` - GPU-accelerated MaxSim

### Phase 3: Python Bindings (Week 5)

#### Tasks

1. **Python API**
   - Device selection
   - GPU encoding functions
   - Performance monitoring

2. **Examples**
   - GPU encoding example
   - Performance comparison (CPU vs GPU)
   - Batch processing guide

#### Expected API

```python
import rank_rerank

# Check GPU availability
device = rank_rerank.get_gpu_device()  # "cuda", "metal", or "cpu"

# GPU-accelerated encoding
encoder = rank_rerank.CandleEncoder.from_huggingface(
    "colbert-ir/colbertv2.0",
    device=device
)

# Encode with GPU
query_tokens = encoder.encode("query text", device=device)
doc_tokens = encoder.encode_batch(["doc1", "doc2"], device=device)

# MaxSim with GPU (if implemented)
scores = rank_rerank.maxsim_vecs_gpu(query_tokens, doc_tokens)
```

## Performance Targets

### Encoding

- **CPU (baseline)**: ~10ms per document
- **GPU (FP32)**: ~5ms per document (2x speedup)
- **GPU (FP16)**: ~3ms per document (3x speedup)

### MaxSim

- **CPU (baseline)**: ~1ms for 100 documents
- **GPU**: ~0.1ms for 100 documents (10x speedup for large batches)

## Implementation Details

### Device Selection

```rust
pub enum Device {
    Cpu,
    Cuda(usize),  // GPU index
    Metal,
}

impl Device {
    pub fn auto() -> Self {
        if cfg!(feature = "gpu-cuda") && cuda_available() {
            Device::Cuda(0)
        } else if cfg!(feature = "gpu-metal") && metal_available() {
            Device::Metal
        } else {
            Device::Cpu
        }
    }
}
```

### Candle Encoder

```rust
#[cfg(feature = "gpu")]
pub struct CandleEncoder {
    model: candle_nn::VarBuilder,
    device: Device,
    tokenizer: Tokenizer,
}

#[cfg(feature = "gpu")]
impl CandleEncoder {
    pub fn from_huggingface(model_name: &str, device: Device) -> Result<Self>;
    pub fn encode(&self, text: &str) -> Result<Vec<Vec<f32>>>;
    pub fn encode_batch(&self, texts: &[&str]) -> Result<Vec<Vec<Vec<f32>>>>;
}
```

## Testing Strategy

1. **Unit Tests**
   - Device detection
   - Encoding correctness (GPU vs CPU)
   - Error handling

2. **Performance Tests**
   - Benchmark CPU vs GPU
   - Batch size optimization
   - Memory usage

3. **Integration Tests**
   - End-to-end pipeline
   - Fallback behavior
   - Multi-GPU support

## Dependencies

### Required

- `candle-core` - Core tensor operations
- `candle-nn` - Neural network primitives
- `candle-transformers` - Transformer models
- `tokenizers` - Tokenization (already added)

### Optional

- CUDA toolkit (for CUDA support)
- Metal (for macOS GPU support)

## Risks and Mitigations

### Risk 1: Candle API Changes

**Mitigation**: Pin version, test regularly, maintain compatibility layer

### Risk 2: GPU Memory Issues

**Mitigation**: Batch size limits, memory pooling, fallback to CPU

### Risk 3: Cross-Platform Compatibility

**Mitigation**: Feature flags, conditional compilation, comprehensive testing

## Success Criteria

- [ ] GPU encoding 2-3x faster than CPU
- [ ] MaxSim GPU acceleration for large batches
- [ ] Graceful fallback to CPU
- [ ] Python bindings complete
- [ ] Documentation and examples
- [ ] Performance benchmarks

## Future Enhancements

1. **Multi-GPU Support**
   - Distribute batches across GPUs
   - Model parallelism

2. **Quantization**
   - INT8 quantization for GPU
   - Mixed precision (FP16/FP32)

3. **Training Support**
   - Burn integration for training
   - Fine-tuning workflows

## References

- [Candle Documentation](https://github.com/huggingface/candle)
- [Vespa ColBERT GPU Benchmarks](https://blog.vespa.ai/)
- [PyLate Training Framework](https://arxiv.org/abs/2508.03555)

