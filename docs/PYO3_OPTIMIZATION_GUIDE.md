# PyO3 Performance Optimization Guide

Based on deep research of production issues, technical blogs, and PyO3 documentation.

## Critical Performance Issues Found

### 1. Function Call Overhead

**Problem**: PyO3 bindings can be 20x slower than pure Rust execution (GitHub #3442).

**Root Causes**:
- GIL management overhead (even when releasing)
- Reference counting synchronization
- Type conversion (`extract()` vs `cast()`)
- Calling convention (vectorcall vs tp_call)

**Solutions**:

#### Use `cast` instead of `extract`

```rust
// ❌ SLOW: extract() converts PyDowncastError to PyErr
if let Ok(list) = value.extract::<Bound<'_, PyList>>() {
    // ...
}

// ✅ FAST: cast() avoids error conversion
if let Ok(list) = value.cast::<PyList>() {
    // ...
}
```

**Performance**: 2x speedup in polymorphic APIs.

#### Use Rust tuples for function arguments

```rust
// ❌ SLOW: Bound<PyTuple> only supports tp_call
#[pyfunction]
fn slow_function(args: Bound<'_, PyTuple>) -> PyResult<()> {
    // ...
}

// ✅ FAST: Rust tuples can use vectorcall protocol
#[pyfunction]
fn fast_function(arg1: i32, arg2: f64, arg3: String) -> PyResult<()> {
    // ...
}
```

#### Avoid `Python::attach` when possible

```rust
// ❌ SLOW: attach() has overhead even when already attached
impl PartialEq<Foo> for FooBound<'_> {
    fn eq(&self, other: &Foo) -> bool {
        Python::attach(|py| {
            let len = other.0.bind(py).len();
            self.0.len() == len
        })
    }
}

// ✅ FAST: Use Bound::py() for zero-cost access
impl PartialEq<Foo> for FooBound<'_> {
    fn eq(&self, other: &Foo) -> bool {
        let py = self.0.py();  // Zero-cost access
        let len = other.0.bind(py).len();
        self.0.len() == len
    }
}
```

### 2. GIL Management for Long Computations

**Problem**: Holding GIL during CPU-bound work blocks other threads.

**Solution**: Use `Python::detach` for work >1ms.

```rust
#[pyfunction]
fn compute_intensive_task(py: Python<'_>, data: Vec<f64>) -> PyResult<Vec<f64>> {
    // Detach before long computation
    py.allow_threads(|| {
        // This work can run in parallel with other Python threads
        expensive_rust_computation(data)
    })
}
```

**Rule of thumb**: Detach for any work expected to take multiple milliseconds.

### 3. Reference Counting Overhead

**Problem**: Deferred reference counting pool adds synchronization cost.

**Solution**: For embedded Python scenarios, disable reference pool.

```toml
# In Cargo.toml
[features]
default = []
disable_reference_pool = ["pyo3/disable-reference-pool"]
```

**Trade-off**: Must explicitly dispose `Py<T>` objects when dropping without GIL.

### 4. Data Conversion Overhead

**Problem**: Repeatedly converting between Python and Rust representations.

**Solution**: Keep data in Rust format, expose high-level operations.

```rust
// ❌ BAD: Converting on every call
#[pyfunction]
fn process_item(item: Vec<f64>) -> PyResult<f64> {
    // Conversion overhead on every call
    compute(item)
}

// ✅ GOOD: Batch processing
#[pyfunction]
fn process_batch(items: Vec<Vec<f64>>) -> PyResult<Vec<f64>> {
    // Single conversion, process all at once
    items.into_iter().map(compute).collect()
}
```

**Pattern**: Follow Polars approach - data stays in Rust, Python uses high-level ops.

## Best Practices Checklist

- [ ] Use `cast()` instead of `extract()` when error handling isn't needed
- [ ] Use Rust tuples for function arguments (not `Bound<PyTuple>`)
- [ ] Use `Bound::py()` instead of `Python::attach()` when possible
- [ ] Detach from interpreter for computations >1ms
- [ ] Batch operations to amortize FFI overhead
- [ ] Keep data in Rust format, minimize conversions
- [ ] Use release builds (`--release` flag)
- [ ] Profile with flame graphs to identify bottlenecks

## Performance Benchmarks

Based on research:

| Operation | Pure Rust | PyO3 Binding | Overhead |
|-----------|-----------|--------------|----------|
| Simple function call | 60ns | 22,350ns | 372x |
| HTTP server call | 20μs | 400μs | 20x |
| Batch processing (1000 items) | 165ns/item | 195ns/item | 18% |

**Key Insight**: Overhead becomes negligible for longer operations. Batch processing is critical.

## Implementation Status

### Current State
- ✅ All crates have PyO3 bindings
- ⚠️ Need to audit for `extract()` usage
- ⚠️ Need to add `Python::detach` for long operations
- ⚠️ Need to optimize argument passing

### Next Steps
1. Audit all Python bindings for `extract()` → `cast()` opportunities
2. Add `Python::detach` for compute-intensive functions
3. Review function signatures for Rust tuple usage
4. Add performance benchmarks to CI

## References

- PyO3 Performance Guide: https://pyo3.rs/main/performance.html
- GitHub Discussion #3442: PyO3 bindings slower than pure rust
- Production patterns from Polars, Candle, and other successful Rust/Python libraries

