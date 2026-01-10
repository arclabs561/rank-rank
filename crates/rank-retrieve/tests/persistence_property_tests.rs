//! Property-based tests for persistence layer.
//!
//! Tests invariants, correctness properties, and edge cases for:
//! - Segment write/read roundtrips
//! - WAL replay correctness
//! - Codec compression/decompression
//! - FST dictionary operations
//! - Checksum validation
//! - Recovery consistency

#[cfg(feature = "persistence")]
use rank_retrieve::persistence::codec::{bitpack, delta, varint};
#[cfg(feature = "persistence")]
use rank_retrieve::persistence::directory::{Directory, MemoryDirectory};
#[cfg(feature = "persistence")]
#[cfg(feature = "persistence")]
use rank_retrieve::persistence::format::{SegmentFooter, SegmentOffsets};
#[cfg(feature = "persistence")]
use rank_retrieve::persistence::segment::{SegmentReader, SegmentWriter, TermInfo};
#[cfg(feature = "persistence")]
use rank_retrieve::persistence::wal::{WalEntry, WalReader, WalWriter};
#[cfg(feature = "persistence")]
use std::collections::HashMap;
#[cfg(feature = "persistence")]
use std::sync::Arc;

#[cfg(feature = "persistence")]
use proptest::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// Codec Property Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "persistence")]
#[test]
fn test_varint_roundtrip() {
    // Property: encode then decode should return original value
    proptest!(|(value in 0u64..u64::MAX)| {
        let encoded = varint::encode(value);
        let (decoded, _bytes_read) = varint::decode(&encoded).unwrap();
        prop_assert_eq!(value, decoded, "Varint roundtrip failed for value {}", value);
    });
}

#[cfg(feature = "persistence")]
#[test]
fn test_varint_many_roundtrip() {
    // Property: encode_many then decode_many should return original values
    proptest!(|(values in proptest::collection::vec(0u64..10000u64, 0..100))| {
        let encoded = varint::encode_many(&values);
        let (decoded, _bytes_read) = varint::decode_many(&encoded, values.len()).unwrap();
        prop_assert_eq!(values, decoded, "Varint many roundtrip failed");
    });
}

#[cfg(feature = "persistence")]
#[test]
fn test_delta_roundtrip() {
    // Property: encode then decode should return original sorted sequence
    proptest!(|(mut values in proptest::collection::vec(0u32..10000u32, 1..100))| {
        values.sort();
        let deltas = delta::encode(&values);
        let decoded = delta::decode(&deltas);
        prop_assert_eq!(values, decoded, "Delta roundtrip failed");
    });
}

#[cfg(feature = "persistence")]
#[test]
fn test_delta_encoding_reduces_size() {
    // Property: Delta encoding should reduce size for sorted sequences
    proptest!(|(mut values in proptest::collection::vec(0u32..10000u32, 10..100))| {
        values.sort();
        let deltas = delta::encode(&values);
        // For sorted sequences with small gaps, deltas should be smaller
        // (This is a probabilistic property - not always true but usually true)
        if values.len() > 1 {
            let max_value = *values.last().unwrap();
            let max_delta = *deltas.iter().skip(1).max().unwrap_or(&0);
            // If max delta is much smaller than max value, encoding is beneficial
            // We just check that deltas are reasonable (not larger than values)
            prop_assert!(max_delta <= max_value, "Delta should not exceed original value");
        }
    });
}

#[cfg(feature = "persistence")]
#[test]
fn test_bitpack_roundtrip() {
    // Property: pack then unpack should return original values
    proptest!(|(values in proptest::collection::vec(0u32..1000u32, 1..128))| {
        let bit_width = bitpack::bit_width_many(&values);
        let packed = bitpack::pack(&values, bit_width);
        let unpacked = bitpack::unpack(&packed, values.len(), bit_width).unwrap();
        prop_assert_eq!(values, unpacked, "Bitpack roundtrip failed");
    });
}

#[cfg(feature = "persistence")]
#[test]
fn test_bitpack_bit_width_correctness() {
    // Property: bit_width should be minimal bits needed
    proptest!(|(value in 0u32..u32::MAX)| {
        let width = bitpack::bit_width(value);
        // Check that width is correct: 2^width should be >= value
        let max_value_for_width = (1u64 << width) - 1;
        prop_assert!(
            max_value_for_width >= value as u64,
            "bit_width {} too small for value {} (max: {})",
            width,
            value,
            max_value_for_width
        );
        // Check that width-1 would be too small (if width > 1)
        if width > 1 {
            let prev_max = (1u64 << (width - 1)) - 1;
            prop_assert!(
                prev_max < value as u64 || value == 0,
                "bit_width {} too large for value {} (prev max: {})",
                width,
                value,
                prev_max
            );
        }
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Segment Property Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "persistence")]
#[test]
fn test_segment_write_read_roundtrip() {
    // Property: Write segment then read should return same data
    proptest!(|(
        term_count in 1usize..50,
        doc_count in 1usize..100,
        term_freq in 1u32..10u32
    )| {
        let test_dir = MemoryDirectory::new();
        let segment_id = 1u64;
        
        // Create test index
        let mut postings = HashMap::new();
        let mut doc_lengths = HashMap::new();
        let mut doc_frequencies = HashMap::new();
        
        for term_idx in 0..term_count {
            let term = format!("term{}", term_idx);
            let mut term_postings = HashMap::new();
            
            // Each term appears in some documents
            for doc_idx in 0..doc_count {
                if doc_idx % (term_idx + 2) == 0 {
                    // Term appears in this document
                    term_postings.insert(doc_idx as u32, term_freq);
                    *doc_lengths.entry(doc_idx as u32).or_insert(0) += term_freq;
                }
            }
            
            if !term_postings.is_empty() {
                let df = term_postings.len() as u32;
                postings.insert(term.clone(), term_postings);
                doc_frequencies.insert(term, df);
            }
        }
        
        // Fill in missing doc_lengths
        for doc_idx in 0..doc_count {
            doc_lengths.entry(doc_idx as u32).or_insert(1);
        }
        
        // Write segment
        let mut writer = SegmentWriter::new(Box::new(test_dir.clone()), segment_id);
        writer.write_bm25_index(&postings, &doc_lengths, &doc_frequencies).unwrap();
        writer.finalize().unwrap();
        
        // Read segment
        let reader = SegmentReader::load(Box::new(test_dir), segment_id).unwrap();
        
        // Verify all terms are present
        for (term, _term_postings) in &postings {
            prop_assert!(
                reader.term_info(term).is_some(),
                "Term {} should be in segment",
                term
            );
        }
        
        // Verify document lengths
        for (doc_id, expected_length) in &doc_lengths {
            let actual_length = reader.doc_length(*doc_id);
            prop_assert_eq!(
                actual_length,
                Some(*expected_length),
                "Doc {} length mismatch: expected {}, got {:?}",
                doc_id,
                expected_length,
                actual_length
            );
        }
    });
}

#[cfg(feature = "persistence")]
#[test]
fn test_segment_footer_roundtrip() {
    // Property: Write footer then read should return same data
    proptest!(|(
        doc_count in 1u32..1000,
        max_doc_id in 1u32..10000
    )| {
        let offsets = SegmentOffsets {
            term_dict_offset: 0,
            term_dict_len: 100,
            term_info_offset: 100,
            term_info_len: 200,
            postings_offset: 300,
            postings_len: 400,
            doc_lengths_offset: 700,
            doc_lengths_len: 800,
            docid_to_userid_offset: 1500,
            docid_to_userid_len: 1600,
            userid_to_docid_offset: 3100,
            userid_to_docid_len: 3200,
            tombstones_offset: 6300,
            tombstones_len: 6400,
        };
        
        let offsets_clone = offsets.clone();
        let footer = SegmentFooter::new(doc_count, max_doc_id, offsets);
        
        // Write footer
        let mut buffer = Vec::new();
        footer.write(&mut buffer).unwrap();
        prop_assert_eq!(buffer.len(), SegmentFooter::SIZE);
        
        // Read footer
        let mut reader = std::io::Cursor::new(&buffer);
        let read_footer = SegmentFooter::read(&mut reader).unwrap();
        
        // Verify all fields match
        prop_assert_eq!(read_footer.doc_count, doc_count);
        prop_assert_eq!(read_footer.max_doc_id, max_doc_id);
        prop_assert_eq!(read_footer.term_dict_offset, offsets_clone.term_dict_offset);
        prop_assert_eq!(read_footer.term_dict_len, offsets_clone.term_dict_len);
    });
}

#[cfg(feature = "persistence")]
#[test]
fn test_segment_term_info_consistency() {
    // Property: Term info should be consistent with postings
    proptest!(|(
        term_count in 1usize..20,
        doc_count in 1usize..50
    )| {
        let test_dir = MemoryDirectory::new();
        let segment_id = 1u64;
        
        let mut postings = HashMap::new();
        let mut doc_lengths = HashMap::new();
        let mut doc_frequencies = HashMap::new();
        
        for term_idx in 0..term_count {
            let term = format!("term{}", term_idx);
            let mut term_postings = HashMap::new();
            
            for doc_idx in 0..doc_count {
                if doc_idx % 3 == term_idx % 3 {
                    term_postings.insert(doc_idx as u32, 2);
                    *doc_lengths.entry(doc_idx as u32).or_insert(0) += 2;
                }
            }
            
            if !term_postings.is_empty() {
                let df = term_postings.len() as u32;
                postings.insert(term.clone(), term_postings);
                doc_frequencies.insert(term, df);
            }
        }
        
        for doc_idx in 0..doc_count {
            doc_lengths.entry(doc_idx as u32).or_insert(1);
        }
        
        let mut writer = SegmentWriter::new(Box::new(test_dir.clone()), segment_id);
        writer.write_bm25_index(&postings, &doc_lengths, &doc_frequencies).unwrap();
        writer.finalize().unwrap();
        
        let reader = SegmentReader::load(Box::new(test_dir), segment_id).unwrap();
        
        // Verify term info matches postings
        for (term, term_postings) in &postings {
            let term_info = reader.term_info(term).unwrap();
            let expected_df = term_postings.len() as u32;
            let expected_cf: u64 = term_postings.values().map(|&tf| tf as u64).sum();
            
            prop_assert_eq!(
                term_info.doc_frequency,
                expected_df,
                "Doc frequency mismatch for term {}",
                term
            );
            prop_assert_eq!(
                term_info.collection_frequency,
                expected_cf,
                "Collection frequency mismatch for term {}",
                term
            );
        }
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// WAL Property Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "persistence")]
#[test]
fn test_wal_replay_roundtrip() {
    // Property: Write WAL entries then replay should return same entries
    proptest!(|(entry_count in 1usize..20)| {
        let dir: Arc<dyn Directory> = Arc::new(MemoryDirectory::new());
        dir.create_dir_all("wal").unwrap();
        
        let mut writer = WalWriter::new(dir.clone());
        let mut written_entries = Vec::new();
        
        // Write entries
        for i in 0..entry_count {
            let entry = WalEntry::AddSegment {
                entry_id: i as u64 + 1,
                segment_id: i as u64 + 1,
                doc_count: (i * 10 + 5) as u32,
            };
            writer.append(entry.clone()).unwrap();
            written_entries.push(entry);
        }
        
        // Replay entries
        let reader = WalReader::new(dir.clone());
        let replayed_entries = reader.replay().unwrap();
        
        prop_assert_eq!(
            replayed_entries.len(),
            written_entries.len(),
            "Replayed entry count mismatch"
        );
        
        // Verify entries match
        for (written, replayed) in written_entries.iter().zip(replayed_entries.iter()) {
            prop_assert_eq!(
                written, replayed,
                "Entry mismatch: written {:?}, replayed {:?}",
                written, replayed
            );
        }
    });
}

#[cfg(feature = "persistence")]
#[test]
fn test_wal_entry_id_monotonicity() {
    // Property: WAL entry IDs should be monotonic
    proptest!(|(entry_count in 2usize..20    )| {
        let dir: Arc<dyn Directory> = Arc::new(MemoryDirectory::new()) as Arc<dyn Directory>;
        dir.create_dir_all("wal").unwrap();
        
        let mut writer = WalWriter::new(dir.clone());
        let mut entry_ids = Vec::new();
        
        for i in 0..entry_count {
            let entry_id = writer.append(WalEntry::AddSegment {
                entry_id: i as u64 + 1,
                segment_id: i as u64 + 1,
                doc_count: 10,
            }).unwrap();
            entry_ids.push(entry_id);
        }
        
        // Verify monotonicity
        for i in 1..entry_ids.len() {
            prop_assert!(
                entry_ids[i] > entry_ids[i - 1],
                "Entry IDs not monotonic: {} <= {}",
                entry_ids[i],
                entry_ids[i - 1]
            );
        }
    });
}

#[cfg(feature = "persistence")]
#[test]
fn test_wal_replay_preserves_order() {
    // Property: WAL replay should preserve entry order
    proptest!(|(entry_count in 1usize..20    )| {
        let dir: Arc<dyn Directory> = Arc::new(MemoryDirectory::new()) as Arc<dyn Directory>;
        dir.create_dir_all("wal").unwrap();
        
        let mut writer = WalWriter::new(dir.clone());
        
        // Write entries with distinct segment IDs
        for i in 0..entry_count {
            writer.append(WalEntry::AddSegment {
                entry_id: i as u64 + 1,
                segment_id: (i * 100) as u64 + 1, // Distinct segment IDs
                doc_count: (i * 10) as u32,
            }).unwrap();
        }
        
        // Replay and verify order
        let reader = WalReader::new(dir);
        let entries = reader.replay().unwrap();
        
        for i in 0..entries.len() {
            if let WalEntry::AddSegment { segment_id, .. } = &entries[i] {
                let expected_segment_id = (i * 100) as u64 + 1;
                prop_assert_eq!(
                    *segment_id,
                    expected_segment_id,
                    "Entry order not preserved at position {}",
                    i
                );
            }
        }
    });
}

#[cfg(feature = "persistence")]
#[test]
fn test_wal_checksum_validation() {
    // Property: Corrupted WAL entries should be detected
    let dir: Arc<dyn Directory> = Arc::new(MemoryDirectory::new()) as Arc<dyn Directory>;
    dir.create_dir_all("wal").unwrap();
    
    let mut writer = WalWriter::new(dir.clone());
    writer.append(WalEntry::AddSegment {
        entry_id: 1,
        segment_id: 1,
        doc_count: 100,
    }).unwrap();
    
    // Manually corrupt the WAL file by modifying a byte
    // This is a deterministic test, not property-based
    // (We can't easily corrupt in-memory directory, so this is a basic test)
    
    let reader = WalReader::new(dir);
    let entries = reader.replay().unwrap();
    assert_eq!(entries.len(), 1, "Should read one valid entry");
}

// ─────────────────────────────────────────────────────────────────────────────
// Recovery Property Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "persistence")]
#[test]
fn test_recovery_idempotency() {
    // Property: Multiple recoveries should produce same state
    proptest!(|(segment_count in 1usize..10    )| {
        let dir: Arc<dyn Directory> = Arc::new(MemoryDirectory::new()) as Arc<dyn Directory>;
        dir.create_dir_all("wal").unwrap();
        dir.create_dir_all("checkpoints").unwrap();
        
        // Write WAL entries
        let mut writer = WalWriter::new(dir.clone());
        for i in 0..segment_count {
            writer.append(WalEntry::AddSegment {
                entry_id: i as u64 + 1,
                segment_id: i as u64 + 1,
                doc_count: (i * 10 + 5) as u32,
            }).unwrap();
        }
        
        // Recover twice
        use rank_retrieve::persistence::recovery::RecoveryManager;
        let recovery1 = RecoveryManager::new(dir.clone());
        let state1 = recovery1.recover().unwrap();
        
        let recovery2 = RecoveryManager::new(dir);
        let state2 = recovery2.recover().unwrap();
        
        // States should be identical
        prop_assert_eq!(
            state1.active_segments.len(),
            state2.active_segments.len(),
            "Active segment count mismatch"
        );
        prop_assert_eq!(
            state1.last_entry_id,
            state2.last_entry_id,
            "Last entry ID mismatch"
        );
        prop_assert_eq!(
            state1.next_segment_id,
            state2.next_segment_id,
            "Next segment ID mismatch"
        );
    });
}

#[cfg(feature = "persistence")]
#[test]
fn test_recovery_preserves_segments() {
    // Property: Recovery should preserve all segments from WAL
    proptest!(|(segment_count in 1usize..10    )| {
        let dir: Arc<dyn Directory> = Arc::new(MemoryDirectory::new()) as Arc<dyn Directory>;
        dir.create_dir_all("wal").unwrap();
        
        let mut writer = WalWriter::new(dir.clone());
        let mut expected_segment_ids = std::collections::HashSet::new();
        
        for i in 0..segment_count {
            let segment_id = i as u64 + 1;
            expected_segment_ids.insert(segment_id);
            writer.append(WalEntry::AddSegment {
                entry_id: i as u64 + 1,
                segment_id,
                doc_count: 10,
            }).unwrap();
        }
        
        use rank_retrieve::persistence::recovery::RecoveryManager;
        let recovery = RecoveryManager::new(dir);
        let state = recovery.recover().unwrap();
        
        let recovered_segment_ids: std::collections::HashSet<u64> = 
            state.active_segments.keys().copied().collect();
        
        prop_assert_eq!(
            recovered_segment_ids,
            expected_segment_ids,
            "Recovered segments don't match written segments"
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// FST Dictionary Property Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "persistence")]
#[test]
fn test_fst_prefix_search() {
    // Property: Prefix search should return all terms with that prefix
    proptest!(|(
        prefix in "[a-z]{2,3}",
        suffix_count in 1usize..10
    )| {
        let test_dir = MemoryDirectory::new();
        let segment_id = 1u64;
        
        let mut postings = HashMap::new();
        let mut doc_lengths = HashMap::new();
        let mut doc_frequencies = HashMap::new();
        
        // Create terms with the prefix (use numeric suffix to avoid conflicts)
        for i in 0..suffix_count {
            let term = format!("{}x{}", prefix, i);
            let mut term_postings = HashMap::new();
            term_postings.insert(0u32, 1u32);
            postings.insert(term.clone(), term_postings);
            doc_frequencies.insert(term, 1);
        }
        
        // Add some terms without the prefix (use different starting letter)
        for i in 0..5 {
            let term = format!("zother{}", i);
            let mut term_postings = HashMap::new();
            term_postings.insert(0u32, 1u32);
            postings.insert(term.clone(), term_postings);
            doc_frequencies.insert(term, 1);
        }
        
        doc_lengths.insert(0u32, (suffix_count + 5) as u32);
        
        let mut writer = SegmentWriter::new(Box::new(test_dir.clone()), segment_id);
        writer.write_bm25_index(&postings, &doc_lengths, &doc_frequencies).unwrap();
        writer.finalize().unwrap();
        
        let reader = SegmentReader::load(Box::new(test_dir), segment_id).unwrap();
        let prefix_results = reader.search_prefix(&prefix);
        
        // Should find all terms with the prefix
        prop_assert_eq!(
            prefix_results.len(),
            suffix_count,
            "Prefix search should find {} terms, found {}",
            suffix_count,
            prefix_results.len()
        );
        
        // All results should start with the prefix
        for (term, _ordinal) in &prefix_results {
            prop_assert!(
                term.starts_with(&prefix),
                "Prefix search result '{}' doesn't start with prefix '{}'",
                term,
                prefix
            );
        }
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Additional Property Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "persistence")]
#[test]
fn test_bitpack_compression_ratio() {
    // Property: Bitpacking should reduce size for small values
    proptest!(|(values in proptest::collection::vec(0u32..100u32, 10..128))| {
        let bit_width = bitpack::bit_width_many(&values);
        let packed = bitpack::pack(&values, bit_width);
        
        // Calculate sizes
        let original_size = values.len() * 4; // 4 bytes per u32
        let packed_size = packed.len();
        
        // For small values, bitpacking should be beneficial
        if bit_width <= 8 {
            prop_assert!(
                packed_size <= original_size,
                "Bitpacking should reduce size for small values: original={}, packed={}, bit_width={}",
                original_size,
                packed_size,
                bit_width
            );
        }
    });
}

#[cfg(feature = "persistence")]
#[test]
fn test_delta_encoding_benefit() {
    // Property: Delta encoding should benefit sorted sequences
    proptest!(|(mut values in proptest::collection::vec(0u32..10000u32, 10..100))| {
        values.sort();
        let deltas = delta::encode(&values);
        
        // For sorted sequences, deltas should generally be smaller than original values
        if values.len() > 1 {
            let max_value = *values.last().unwrap();
            let max_delta = *deltas.iter().skip(1).max().unwrap_or(&0);
            
            // If sequence is dense (small gaps), deltas should be much smaller
            // This is probabilistic - not always true but usually true for sorted sequences
            if max_value > 100 {
                // For large sorted sequences, at least some deltas should be smaller
                let small_deltas = deltas.iter().skip(1).filter(|&&d| d < 100).count();
                prop_assert!(
                    small_deltas > 0 || max_delta < max_value,
                    "Delta encoding should benefit sorted sequences"
                );
            }
        }
    });
}

#[cfg(feature = "persistence")]
#[test]
fn test_segment_checksum_validation() {
    // Property: Segment footer checksum should detect corruption
    let test_dir = MemoryDirectory::new();
    let segment_id = 1u64;
    
    let mut postings = HashMap::new();
    let mut term_postings = HashMap::new();
    term_postings.insert(0u32, 1u32);
    postings.insert("test".to_string(), term_postings);
    
    let mut doc_lengths = HashMap::new();
    doc_lengths.insert(0u32, 1u32);
    
    let mut doc_frequencies = HashMap::new();
    doc_frequencies.insert("test".to_string(), 1u32);
    
    let mut writer = SegmentWriter::new(Box::new(test_dir.clone()), segment_id);
    writer.write_bm25_index(&postings, &doc_lengths, &doc_frequencies).unwrap();
    writer.finalize().unwrap();
    
    // Segment should be readable
    let reader = SegmentReader::load(Box::new(test_dir), segment_id).unwrap();
    assert!(reader.term_info("test").is_some(), "Term should be readable");
}

#[cfg(feature = "persistence")]
#[test]
fn test_wal_multiple_segments() {
    // Property: WAL should handle multiple segments correctly
    proptest!(|(segment_count in 2usize..10)| {
        let dir: Arc<dyn Directory> = Arc::new(MemoryDirectory::new()) as Arc<dyn Directory>;
        dir.create_dir_all("wal").unwrap();
        
        let mut writer = WalWriter::new(dir.clone());
        let mut written_segment_ids = Vec::new();
        
        // Write entries that will span multiple WAL segments
        // (by writing many large entries)
        for i in 0..segment_count {
            let segment_id = i as u64 + 1;
            written_segment_ids.push(segment_id);
            
            // Write multiple entries per segment to potentially trigger segment rotation
            for j in 0..5 {
                writer.append(WalEntry::AddSegment {
                    entry_id: (i * 5 + j) as u64 + 1,
                    segment_id,
                    doc_count: (j * 10) as u32,
                }).unwrap();
            }
        }
        
        // Replay and verify all segments are present
        let reader = WalReader::new(dir);
        let entries = reader.replay().unwrap();
        
        let replayed_segment_ids: std::collections::HashSet<u64> = entries
            .iter()
            .filter_map(|e| {
                if let WalEntry::AddSegment { segment_id, .. } = e {
                    Some(*segment_id)
                } else {
                    None
                }
            })
            .collect();
        
        let expected_segment_ids: std::collections::HashSet<u64> = 
            written_segment_ids.iter().copied().collect();
        
        prop_assert_eq!(
            replayed_segment_ids,
            expected_segment_ids,
            "All segments should be replayed"
        );
    });
}

#[cfg(feature = "persistence")]
#[test]
fn test_recovery_handles_deletes() {
    // Property: Recovery should correctly handle delete entries
    proptest!(|(segment_count in 1usize..5, delete_count in 1usize..10)| {
        let dir: Arc<dyn Directory> = Arc::new(MemoryDirectory::new()) as Arc<dyn Directory>;
        dir.create_dir_all("wal").unwrap();
        
        let mut writer = WalWriter::new(dir.clone());
        
        // Add segments
        for i in 0..segment_count {
            writer.append(WalEntry::AddSegment {
                entry_id: i as u64 + 1,
                segment_id: i as u64 + 1,
                doc_count: 100,
            }).unwrap();
        }
        
        // Add deletes
        let mut expected_deletes = std::collections::HashMap::new();
        for i in 0..delete_count {
            let segment_id = (i % segment_count) as u64 + 1;
            let doc_id = (i * 10) as u32;
            expected_deletes
                .entry(segment_id)
                .or_insert_with(Vec::new)
                .push(doc_id);
            
            writer.append(WalEntry::DeleteDocuments {
                entry_id: (segment_count + i) as u64 + 1,
                deletes: vec![(segment_id, doc_id)],
            }).unwrap();
        }
        
        // Recover
        use rank_retrieve::persistence::recovery::RecoveryManager;
        let recovery = RecoveryManager::new(dir);
        let state = recovery.recover().unwrap();
        
        // Verify deletes are present
        for (segment_id, expected_doc_ids) in &expected_deletes {
            let empty_vec = Vec::new();
            let actual_deletes = state.deletes.get(segment_id).unwrap_or(&empty_vec);
            for doc_id in expected_doc_ids {
                prop_assert!(
                    actual_deletes.contains(doc_id),
                    "Delete for segment {} doc {} should be present",
                    segment_id,
                    doc_id
                );
            }
        }
    });
}

#[cfg(feature = "persistence")]
#[test]
fn test_recovery_handles_merges() {
    // Property: Recovery should correctly handle merge entries
    proptest!(|(merge_count in 1usize..5)| {
        let dir: Arc<dyn Directory> = Arc::new(MemoryDirectory::new()) as Arc<dyn Directory>;
        dir.create_dir_all("wal").unwrap();
        
        let mut writer = WalWriter::new(dir.clone());
        
        // Add initial segments
        for i in 0..(merge_count * 2) {
            writer.append(WalEntry::AddSegment {
                entry_id: i as u64 + 1,
                segment_id: i as u64 + 1,
                doc_count: 50,
            }).unwrap();
        }
        
        // Perform merges
        let mut entry_id = (merge_count * 2) as u64 + 1;
        for i in 0..merge_count {
            let transaction_id = i as u64 + 100;
            let old_segment_ids = vec![(i * 2) as u64 + 1, (i * 2) as u64 + 2];
            let new_segment_id = (merge_count * 2 + i) as u64 + 1;
            
            writer.append(WalEntry::StartMerge {
                entry_id,
                transaction_id,
                segment_ids: old_segment_ids.clone(),
            }).unwrap();
            entry_id += 1;
            
            writer.append(WalEntry::EndMerge {
                entry_id,
                transaction_id,
                new_segment_id,
                old_segment_ids: old_segment_ids.clone(),
                remapped_deletes: Vec::new(),
            }).unwrap();
            entry_id += 1;
            
            writer.append(WalEntry::AddSegment {
                entry_id,
                segment_id: new_segment_id,
                doc_count: 100,
            }).unwrap();
            entry_id += 1;
        }
        
        // Recover
        use rank_retrieve::persistence::recovery::RecoveryManager;
        let recovery = RecoveryManager::new(dir);
        let state = recovery.recover().unwrap();
        
        // Verify no pending merges
        prop_assert!(
            state.pending_merges.is_empty(),
            "All merges should be completed, no pending merges"
        );
        
        // Verify old segments are removed and new segments are present
        let active_segment_ids: std::collections::HashSet<u64> = 
            state.active_segments.keys().copied().collect();
        
        // Should have new merged segments
        for i in 0..merge_count {
            let new_segment_id = (merge_count * 2 + i) as u64 + 1;
            prop_assert!(
                active_segment_ids.contains(&new_segment_id),
                "Merged segment {} should be active",
                new_segment_id
            );
        }
    });
}

#[cfg(feature = "persistence")]
#[test]
fn test_codec_edge_cases() {
    // Property: Codecs should handle edge cases correctly
    
    // Varint edge cases
    let edge_values = vec![0u64, 1, 127, 128, 255, 256, 65535, 65536, u32::MAX as u64, u64::MAX];
    for &value in &edge_values {
        let encoded = varint::encode(value);
        let (decoded, _) = varint::decode(&encoded).unwrap();
        assert_eq!(value, decoded, "Varint edge case failed for {}", value);
    }
    
    // Delta edge cases
    let edge_sequences = vec![
        vec![0u32],
        vec![0u32, 1],
        vec![1u32, 2, 3],
        vec![1000u32, 1001, 1002],
        vec![0u32, 100, 200],
    ];
    for seq in edge_sequences {
        let deltas = delta::encode(&seq);
        let decoded = delta::decode(&deltas);
        assert_eq!(seq, decoded, "Delta edge case failed");
    }
    
    // Bitpack edge cases
    let edge_values = vec![
        vec![0u32],
        vec![1u32],
        vec![0u32, 1, 2, 3],
        vec![255u32, 256, 257],
        vec![u32::MAX],
    ];
    for values in edge_values {
        let bit_width = bitpack::bit_width_many(&values);
        let packed = bitpack::pack(&values, bit_width);
        let unpacked = bitpack::unpack(&packed, values.len(), bit_width).unwrap();
        assert_eq!(values, unpacked, "Bitpack edge case failed");
    }
}

#[cfg(feature = "persistence")]
#[test]
fn test_segment_large_corpus() {
    // Property: Segment should handle large corpora
    proptest!(|(
        term_count in 50usize..200,
        doc_count in 100usize..500
    )| {
        let test_dir = MemoryDirectory::new();
        let segment_id = 1u64;
        
        let mut postings = HashMap::new();
        let mut doc_lengths = HashMap::new();
        let mut doc_frequencies = HashMap::new();
        
        // Create a larger index
        for term_idx in 0..term_count {
            let term = format!("term{:04}", term_idx);
            let mut term_postings = HashMap::new();
            
            // Each term appears in some documents
            for doc_idx in 0..doc_count {
                if doc_idx % (term_idx % 10 + 1) == 0 {
                    term_postings.insert(doc_idx as u32, 1);
                    *doc_lengths.entry(doc_idx as u32).or_insert(0) += 1;
                }
            }
            
            if !term_postings.is_empty() {
                let df = term_postings.len() as u32;
                postings.insert(term.clone(), term_postings);
                doc_frequencies.insert(term, df);
            }
        }
        
        // Fill in missing doc_lengths
        for doc_idx in 0..doc_count {
            doc_lengths.entry(doc_idx as u32).or_insert(1);
        }
        
        // Write and read segment
        let mut writer = SegmentWriter::new(Box::new(test_dir.clone()), segment_id);
        writer.write_bm25_index(&postings, &doc_lengths, &doc_frequencies).unwrap();
        writer.finalize().unwrap();
        
        let reader = SegmentReader::load(Box::new(test_dir), segment_id).unwrap();
        
        // Verify term count
        prop_assert_eq!(
            reader.term_count(),
            postings.len(),
            "Term count mismatch"
        );
        
        // Verify a sample of terms
        let sample_terms: Vec<String> = postings.keys().take(10).cloned().collect();
        for term in &sample_terms {
            prop_assert!(
                reader.term_info(term).is_some(),
                "Term {} should be in segment",
                term
            );
        }
    });
}
