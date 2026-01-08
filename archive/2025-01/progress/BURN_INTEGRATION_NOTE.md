# Burn Integration Note

## Current Status

The Burn integration for `rank-soft` is partially implemented but has a compilation issue with the `ElementConversion` trait API.

## Issue

The `ElementConversion` trait in Burn 0.19 has a specific API that requires careful handling of type conversions. The current implementation attempts to convert `B::FloatElem` to `f64` using:

```rust
<B::FloatElem as ElementConversion>::elem::<f64>(v)
```

However, this approach has type inference issues that need to be resolved.

## Workaround

For now, the Burn integration is marked as "in progress" and the code structure is in place. The functions `soft_rank_burn` and `spearman_loss_burn` are defined but need the conversion logic to be fixed.

## Next Steps

1. Investigate Burn's `ElementConversion` trait API more thoroughly
2. Check if there's a simpler conversion path (e.g., `Into<f64>` for numeric types)
3. Consider using Burn's native tensor operations instead of converting to `Vec<f64>`
4. Test with actual Burn backends (Wgpu, Cuda, etc.) to verify the API

## References

- Burn documentation: https://burn.dev/docs/burn/tensor/trait.ElementConversion.html
- Burn tensor operations: https://burn.dev/docs/burn/tensor/struct.Tensor.html

