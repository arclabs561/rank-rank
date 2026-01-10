# FST Usage Optimization

## Summary

After reviewing the [fst crate documentation](https://docs.rs/fst), we've optimized our usage to follow best practices:

## Key Improvements

### 1. **Direct FST Lookups Instead of HashMap Conversion**

**Before**: We loaded the FST and immediately converted it to a `HashMap<String, u64>`, defeating the purpose of using FST.

**After**: We now use the FST directly via `Map::get()` for O(1) lookups without the memory overhead of a HashMap.

```rust
// Old (inefficient):
let fst_map = Map::new(fst_buffer)?;
let mut dict = HashMap::new();
// ... convert entire FST to HashMap ...

// New (efficient):
let term_dict_fst: Option<Map<Vec<u8>>> = Map::new(fst_buffer).ok();
// Use directly: term_dict_fst.get(term.as_bytes())
```

**Benefits**:
- **Memory**: FST is ~10-100x more compact than HashMap for term dictionaries
- **Performance**: O(1) lookups with better cache locality
- **Scalability**: Can handle millions of terms without memory pressure

### 2. **Lexicographic Sorting for Optimal Compression**

**Before**: We inserted terms in arbitrary order (though comment said "sorted").

**After**: We explicitly sort terms lexicographically before building the FST.

```rust
// Sort terms lexicographically before building FST
// This is required for optimal FST compression and correctness
let mut sorted_terms: Vec<_> = self.term_dict.iter().collect();
sorted_terms.sort_by(|a, b| a.0.cmp(b.0));
```

**Benefits**:
- **Compression**: FST achieves maximum compression when keys are sorted
- **Correctness**: MapBuilder requires sorted keys for proper construction
- **Performance**: Sorted keys enable better internal optimizations

### 3. **Prefix Search Support**

**Added**: New `search_prefix()` method for query expansion and autocomplete.

```rust
pub fn search_prefix(&self, prefix: &str) -> Vec<(String, u64)> {
    // Uses FST's efficient range search
    let mut stream = fst_map.range().ge(prefix_bytes).lt(&end_prefix).into_stream();
    // ...
}
```

**Benefits**:
- **Query Expansion**: Find related terms for query expansion
- **Autocomplete**: Fast prefix matching for search suggestions
- **Efficiency**: O(prefix_length) to find start, then O(k) for k matches

### 4. **Proper Offset Tracking**

**Fixed**: We now properly track file offsets during segment writing, enabling accurate footer construction.

## FST Best Practices Applied

1. ✅ **Sorted Keys**: Terms are sorted lexicographically before FST construction
2. ✅ **Direct Lookups**: Use `Map::get()` instead of converting to HashMap
3. ✅ **Memory Efficiency**: FST is kept in memory-mappable format (Vec<u8>)
4. ✅ **Prefix Searches**: Leverage FST's range search for prefix matching
5. ✅ **Stream Iteration**: Use `Streamer` trait for efficient iteration

## Performance Characteristics

- **Lookup**: O(1) average case, O(m) worst case where m is key length
- **Memory**: ~2-5 bytes per key (vs ~50-100 bytes for HashMap entry)
- **Prefix Search**: O(prefix_length + k) where k is number of matches
- **Construction**: O(n log n) for sorting + O(n) for FST building

## Future Optimizations

1. **Memory Mapping**: Use `memmap2` to memory-map FST files for zero-copy access
2. **Lazy Loading**: Only load FST when needed, not during segment load
3. **FST Caching**: Cache frequently accessed FSTs across segment readers
4. **Automaton Queries**: Use FST's automaton support for fuzzy matching (Levenshtein, regex)

## References

- [fst crate documentation](https://docs.rs/fst)
- [fst GitHub repository](https://github.com/BurntSushi/fst)
- [Index 1,600,000,000 Keys with Automata and Rust](https://blog.burntsushi.net/transducers/) - Blog post by fst author
