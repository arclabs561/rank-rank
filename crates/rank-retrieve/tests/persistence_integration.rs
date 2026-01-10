//! Integration tests for persistence layer.
//!
//! Tests the full persistence pipeline: codec → segment → WAL → checkpoint → recovery.

#[cfg(feature = "persistence")]
mod tests {
    use rank_retrieve::persistence::codec::{bitpack, delta, varint};
    use rank_retrieve::persistence::directory::{Directory, FsDirectory, MemoryDirectory};
    use rank_retrieve::persistence::segment::{SegmentReader, SegmentWriter};
    use rank_retrieve::persistence::wal::{WalEntry, WalReader, WalWriter};
    use rank_retrieve::persistence::checkpoint::{CheckpointReader, CheckpointWriter, SegmentMetadata};
    use std::collections::HashMap;
    use std::sync::Arc;

    #[test]
    fn test_codec_roundtrip() {
        // Test varint
        let values = vec![1u64, 127, 128, 255, 256, 65535, u32::MAX as u64];
        for &value in &values {
            let encoded = varint::encode(value);
            let (decoded, _) = varint::decode(&encoded).unwrap();
            assert_eq!(decoded, value, "Varint roundtrip failed for {}", value);
        }

        // Test delta encoding
        let sorted = vec![5u32, 7, 9, 12, 15];
        let deltas = delta::encode(&sorted);
        let decoded = delta::decode(&deltas);
        assert_eq!(decoded, sorted);

        // Test bitpacking
        let values = vec![1u32, 2, 3, 4, 5, 6, 7, 8];
        let bit_width = bitpack::bit_width_many(&values);
        let packed = bitpack::pack(&values, bit_width);
        let unpacked = bitpack::unpack(&packed, values.len(), bit_width).unwrap();
        assert_eq!(unpacked, values);
    }

    #[test]
    fn test_bm25_segment_persistence() {
        let dir = Box::new(MemoryDirectory::new());
        let segment_id = 1;

        // Create test index
        let mut postings = HashMap::new();
        let mut term_postings = HashMap::new();
        term_postings.insert(0u32, 2u32);
        term_postings.insert(1u32, 1u32);
        postings.insert("machine".to_string(), term_postings);

        let mut doc_lengths = HashMap::new();
        doc_lengths.insert(0u32, 5u32);
        doc_lengths.insert(1u32, 3u32);

        let mut doc_frequencies = HashMap::new();
        doc_frequencies.insert("machine".to_string(), 2u32);

        // Write segment
        let mut writer = SegmentWriter::new(dir.clone(), segment_id);
        writer
            .write_bm25_index(&postings, &doc_lengths, &doc_frequencies)
            .unwrap();
        writer.finalize().unwrap();

        // Read segment (use same directory)
        let reader = SegmentReader::load(dir, segment_id).unwrap();
        assert_eq!(reader.doc_length(0), Some(5));
        assert_eq!(reader.doc_length(1), Some(3));
        assert!(reader.term_info("machine").is_some());
    }

    #[test]
    fn test_wal_persistence() {
        let dir_mem = MemoryDirectory::new();
        let dir_arc: Arc<dyn Directory> = Arc::new(dir_mem);
        dir_arc.create_dir_all("wal").unwrap();

        let mut writer = WalWriter::new(dir_arc.clone());

        // Write multiple entries
        let entry1 = WalEntry::AddSegment {
            entry_id: 1,
            segment_id: 1,
            doc_count: 100,
        };
        let entry2 = WalEntry::AddSegment {
            entry_id: 2,
            segment_id: 2,
            doc_count: 200,
        };
        let entry3 = WalEntry::DeleteDocuments {
            entry_id: 3,
            deletes: vec![(1, 5), (1, 10)],
        };

        writer.append(entry1).unwrap();
        writer.append(entry2).unwrap();
        writer.append(entry3).unwrap();

        // Replay WAL (use same directory)
        let reader = WalReader::new(dir_arc);
        let entries = reader.replay().unwrap();
        assert_eq!(entries.len(), 3);

        // Verify entry types
        match &entries[0] {
            WalEntry::AddSegment { segment_id, .. } => assert_eq!(*segment_id, 1),
            _ => panic!("Expected AddSegment"),
        }
        match &entries[2] {
            WalEntry::DeleteDocuments { deletes, .. } => assert_eq!(deletes.len(), 2),
            _ => panic!("Expected DeleteDocuments"),
        }
    }

    #[test]
    fn test_checkpoint_persistence() {
        let dir_mem = MemoryDirectory::new();
        let dir = Box::new(dir_mem.clone());

        let writer = CheckpointWriter::new(dir.clone());
        let segments = vec![
            SegmentMetadata {
                segment_id: 1,
                path: "segments/segment_1".to_string(),
                doc_count: 100,
                max_doc_id: 99,
                size_bytes: 10000,
            },
            SegmentMetadata {
                segment_id: 2,
                path: "segments/segment_2".to_string(),
                doc_count: 200,
                max_doc_id: 199,
                size_bytes: 20000,
            },
        ];

        let checkpoint_path = writer.create_checkpoint(100, &segments).unwrap();

        // Read checkpoint (use same directory)
        let reader = CheckpointReader::new(dir);
        let header = reader.load_checkpoint(&checkpoint_path).unwrap();
        assert_eq!(header.entry_id, 100);
        assert_eq!(header.segment_count, 2);
        assert_eq!(header.doc_count, 300);
    }

    #[test]
    fn test_full_pipeline() {
        // Test complete pipeline: index → segment → WAL → checkpoint → recovery
        let dir_mem = MemoryDirectory::new();
        let dir = Box::new(dir_mem.clone());
        dir.create_dir_all("wal").unwrap();
        dir.create_dir_all("checkpoints").unwrap();

        // 1. Create BM25 index and write to segment
        let mut postings = HashMap::new();
        let mut term_postings = HashMap::new();
        term_postings.insert(0u32, 1u32);
        postings.insert("test".to_string(), term_postings);

        let mut doc_lengths = HashMap::new();
        doc_lengths.insert(0u32, 1u32);

        let mut doc_frequencies = HashMap::new();
        doc_frequencies.insert("test".to_string(), 1u32);

        let segment_id = 1;
        let mut segment_writer = SegmentWriter::new(dir.clone(), segment_id);
        segment_writer
            .write_bm25_index(&postings, &doc_lengths, &doc_frequencies)
            .unwrap();
        segment_writer.finalize().unwrap();

        // 2. Write to WAL (use Arc for WAL operations)
        let dir_arc: Arc<dyn Directory> = Arc::new(dir_mem.clone());
        dir_arc.create_dir_all("wal").unwrap();
        let mut wal_writer = WalWriter::new(dir_arc.clone());
        let wal_entry = WalEntry::AddSegment {
            entry_id: 1,
            segment_id,
            doc_count: 1,
        };
        wal_writer.append(wal_entry).unwrap();

        // 3. Create checkpoint (use Box for checkpoint operations)
        let dir_box = Box::new(dir_mem.clone());
        let checkpoint_writer = CheckpointWriter::new(dir_box);
        let segments = vec![SegmentMetadata {
            segment_id,
            path: format!("segments/segment_{}", segment_id),
            doc_count: 1,
            max_doc_id: 0,
            size_bytes: 1000,
        }];
        checkpoint_writer.create_checkpoint(1, &segments).unwrap();

        // 4. Recover from WAL
        let wal_reader = WalReader::new(dir_arc);
        let entries = wal_reader.replay().unwrap();
        assert_eq!(entries.len(), 1);

        // 5. Load segment (use same directory)
        let reader = SegmentReader::load(Box::new(dir_mem), segment_id).unwrap();
        assert_eq!(reader.doc_length(0), Some(1));
        assert!(reader.term_info("test").is_some());
    }
}
