# Disk Persistence Design for rank-retrieve

This document outlines the design for adding disk persistence to `rank-retrieve`, reusing best practices from Tantivy, SQLite WAL mode, LMDB, and established inverted index formats. This is a comprehensive design document that tracks constraints, architecture, design decisions, and implementation details similar to how Nakala's PLAN.md documents its design.

**Design Philosophy**: This persistence layer is designed to be a production-ready, crash-safe, concurrent storage engine for `rank-retrieve` indexes. It prioritizes correctness and durability over raw performance, while still maintaining excellent performance characteristics through careful design choices.

## Table of Contents

* [What rank-retrieve persistence should NOT do](#what-rank-retrieve-persistence-should-not-do)
* [What rank-retrieve persistence should do](#what-rank-retrieve-persistence-should-do)
* [Design Constraints and Trade-offs](#design-constraints-and-trade-offs)
* [Index Structure and File Organization](#index-structure-and-file-organization)
* [Handling Document Identifiers](#handling-document-identifiers)
* [Handling Deletes](#handling-deletes)
* [Segment Merging Strategy](#segment-merging-strategy)
* [Write-Ahead Log (WAL) Design](#write-ahead-log-wal-design)
* [File Locking and Concurrency](#file-locking-and-concurrency)
* [Index Synchronization](#index-synchronization)
* [ACID Guarantees](#acid-guarantees)
* [Recovery Procedures](#recovery-procedures)
* [On-Disk Format Specifications](#on-disk-format-specifications)
* [Performance Considerations](#performance-considerations)
* [Implementation Phases](#implementation-phases)

## Research Summary

### Best Practices from Existing Systems

#### 1. Tantivy (Rust Search Engine)
- **Directory-based storage**: One directory per index, segments as immutable units
- **Atomic commits**: Write to temp files, then atomic rename (`fsync` + `rename`)
- **Segment-based architecture**: Immutable segments, periodic merging
- **Crash safety**: After successful `commit()`, data is durable; crashes roll back to last commit
- **Reader/writer separation**: One writer per process, multiple readers via `Index::reader()`
- **Versioning**: Format version in metadata, migration path for upgrades

#### 2. Serde + Bincode Patterns
- **Header + payload**: Magic bytes, format version, checksum, then serialized data
- **Atomic writes**: Write to `.tmp`, `fsync`, then `rename()` (atomic on POSIX)
- **Versioning**: Format version in header, backward-compatible schema evolution
- **Incremental updates**: WAL (write-ahead log) for mutations, periodic checkpoints
- **Chunking**: Separate files for different logical parts (users.bin, config.bin, index.bin)

#### 3. Inverted Index Formats
- **Segmented layout**: Immutable segments with footer containing offsets
- **Compressed postings**: Gap-encoded docIDs, varint/PForDelta compression
- **Term dictionary**: FST (finite state transducer) or prefix-compressed blocks
- **Block-based structure**: Fixed-size blocks (128-512 docs) for skipping and partial reads
- **Skip lists**: Block metadata for fast conjunction queries

## Design Constraints and Trade-offs

### Memory Constraints

* **Constant-memory merging**: Segment merging must use O(k) memory where k is the number of segments being merged, not O(total_index_size).
* **Bounded writer buffer**: Writer buffers documents in memory before flushing to segments. Default: 50-100MB, configurable.
* **Lazy segment loading**: Segments are loaded on-demand, not all at startup.

### Durability vs Performance

* **Default: Full durability**: Every commit is fsynced to disk. This ensures data survives power failures.
* **Optional: Reduced durability**: Allow disabling fsync for better performance (application crashes only, not power failures).
* **Checkpoint frequency**: Balance between checkpoint overhead and recovery time. Default: Every 1000 documents or 10MB.

### Concurrency Model

* **Single writer, multiple readers**: Only one process can write at a time. Multiple processes can read simultaneously.
* **Snapshot isolation**: Each reader sees a consistent snapshot of the index at the time it opened.
* **No lock-free reads**: Readers acquire shared locks to ensure consistency. This is acceptable because reads are fast and don't block each other.

### Storage Efficiency

* **Compression**: Use delta encoding + bitpacking for postings (Tantivy approach). Achieve <10% of raw document size for inverted index.
* **Segment overhead**: Accept some overhead from multiple segments. Target: 5-10 segments per index (not hundreds).
* **WAL growth**: WAL files grow until checkpointed. Monitor and checkpoint regularly to prevent unbounded growth.

## Proposed Architecture

### High-Level Design

```
rank-retrieve/
├── src/
│   ├── persistence/
│   │   ├── mod.rs              # Persistence module
│   │   ├── directory.rs        # Directory abstraction (in-memory, disk, etc.)
│   │   ├── segment.rs          # Segment format and management
│   │   ├── wal.rs              # Write-ahead log for incremental updates
│   │   ├── checkpoint.rs       # Checkpoint creation and loading
│   │   ├── format.rs           # Binary format definitions
│   │   └── codec.rs            # Compression codecs (varint, PForDelta, etc.)
│   ├── bm25/
│   │   └── persistent.rs      # Persistent BM25 index implementation
│   ├── dense/
│   │   └── persistent.rs       # Persistent dense retriever
│   └── ...
```

## Index Structure and File Organization

### Directory Structure

```
index_dir/
├── meta.json                   # Index metadata (version, doc_count, segment list)
├── transaction.log             # Transaction log (single source of truth for active segments)
├── [transaction.log.lock]?    # Lock file (non-Linux/Windows platforms)
├── [transaction.log.backup]    # Backup during compaction (temporary)
├── segments/
│   ├── segment_{segmentID}/
│   │   ├── term_dict.fst       # FST term dictionary (term → ordinal)
│   │   ├── term_info.bin       # TermInfoStore (ordinal → metadata)
│   │   ├── postings.bin        # Compressed postings lists
│   │   ├── doc_lengths.bin    # Document length array
│   │   ├── docid_to_userid.fst # FST mapping docID → userID (optional, if user IDs provided)
│   │   ├── userid_to_docid.fst # FST mapping userID → docID range (for deletes)
│   │   ├── tombstones.bin      # Deleted document bitset (1 byte per doc)
│   │   └── footer.bin          # Segment footer (offsets, metadata, checksum)
│   └── ...
├── wal/
│   ├── wal_{startEntryID}.log  # Write-ahead log segments
│   └── ...
├── checkpoints/
│   ├── checkpoint_{entryID}.bin # Full checkpoint snapshots
│   └── ...
├── merges/
│   ├── {transactionID}.active  # Active merge indicator (with advisory lock)
│   └── ...
└── handles/
    ├── {handleID}.active        # Active reader handle (with advisory lock)
    └── ...
```

### File Naming Conventions

* **Segments**: `segment_{segmentID}/` where `segmentID` is a unique u64 identifier
* **WAL files**: `wal_{startEntryID}.log` where `startEntryID` is the first entry ID in that segment
* **Checkpoints**: `checkpoint_{entryID}.bin` where `entryID` is the last entry covered by the checkpoint
* **Merge indicators**: `{transactionID}.active` where `transactionID` identifies the merge transaction
* **Handle indicators**: `{handleID}.active` where `handleID` is a unique identifier for the reader handle

### File Locking Strategy

The file locking strategy varies by platform:

* **Linux**: Use `fcntl` open file description locks (byte-range locking, thread-safe)
* **Windows**: Use `LockFileEx` (byte-range locking, process-safe)
* **macOS/Other Unix**: Use `flock` for transaction log (whole-file, shared/exclusive), `fcntl` for merge/handle indicators (byte-range, process-safe)

**Rationale**: Linux's open file description locks provide the best semantics (thread-safe, byte-range). Windows' `LockFileEx` is process-safe and sufficient. On other platforms, we use `flock` for the transaction log (simpler, works for shared/exclusive) and `fcntl` for other locks (byte-range needed for merge/handle coordination).

### File Format Design

#### 1. Index Metadata (`meta.json`)

```json
{
  "version": 1,
  "format_version": "1.0.0",
  "doc_count": 1000000,
  "created_at": "2025-01-15T10:00:00Z",
  "last_commit": "2025-01-15T12:00:00Z",
  "segments": ["segment_0", "segment_1"],
  "active_wal": "wal_0002.log"
}
```

#### 2. Segment Format

**Footer (fixed size, at end of segment file):**
```rust
#[repr(C)]
struct SegmentFooter {
    magic: [u8; 4],              // b"RANK"
    format_version: u32,         // Format version
    term_dict_offset: u64,       // Offset to term dictionary
    term_dict_len: u64,          // Length of term dictionary
    postings_offset: u64,        // Offset to postings section
    postings_len: u64,           // Length of postings section
    doc_lengths_offset: u64,     // Offset to doc lengths
    doc_lengths_len: u64,        // Length of doc lengths
    doc_count: u32,              // Number of documents in segment
    max_doc_id: u32,             // Maximum document ID
    checksum: u32,               // CRC32 checksum
}
```

**Term Dictionary (FST):**
- Use `fst` crate (same as Tantivy)
- Maps term (UTF-8 bytes) → `u64` offset into postings metadata
- Postings metadata: `(postings_offset, postings_len, doc_freq)`

**Postings Lists:**
- Block-based structure (128 docs per block)
- Gap-encoded docIDs (varint or PForDelta)
- Term frequencies (varint)
- Optional: positions, payloads

**Document Lengths:**
- Simple array: `[doc_length_0, doc_length_1, ...]`
- Varint-encoded for space efficiency

#### 3. Write-Ahead Log (WAL)

**WAL Entry Format:**
```rust
#[derive(Serialize, Deserialize)]
enum WalEntry {
    AddSegment {
        entry_id: u64,
        segment_id: u64,
        doc_count: u32,
    },
    StartMerge {
        entry_id: u64,
        transaction_id: u64,
        segment_ids: Vec<u64>,
    },
    CancelMerge {
        entry_id: u64,
        transaction_id: u64,
        segment_ids: Vec<u64>,
    },
    EndMerge {
        entry_id: u64,
        transaction_id: u64,
        new_segment_id: u64,
        old_segment_ids: Vec<u64>,
        remapped_deletes: Vec<(u64, u32)>, // (segment_id, doc_id)
    },
    DeleteDocuments {
        entry_id: u64,
        deletes: Vec<(u64, u32)>, // (segment_id, doc_id)
    },
    Checkpoint {
        entry_id: u64,
        checkpoint_path: String,
        last_entry_id: u64,
    },
}
```

**WAL File Format (Segmented):**
```
wal_{startEntryID}.log:
[header][entry_0][entry_1]...[entry_n]
```

**Header:**
```rust
#[repr(C)]
struct WalSegmentHeader {
    magic: [u8; 4],              // b"WAL\0"
    version: u32,                // Format version
    start_entry_id: u64,         // First entry ID in this segment
    segment_id: u64,             // Unique segment identifier
}
```

**Entry On-Disk Format:**
```rust
struct WalEntryOnDisk {
    length: u32,                 // Total length (including this header)
    entry_type: u8,              // 0=AddSegment, 1=StartMerge, etc.
    checksum: u32,               // CRC32 of payload
    payload: [u8],               // Serialized entry (bincode)
}
```

**Segmentation**: WAL is split into multiple files when size exceeds threshold (e.g., 10MB). New writes go to current segment. Full segments become read-only.

#### 4. Checkpoint Format

**Checkpoint Structure:**
```
checkpoint_{entryID}.bin:
[header][segment_list][segment_0_data][segment_1_data]...[segment_n_data][footer]
```

**Header:**
```rust
#[repr(C)]
struct CheckpointHeader {
    magic: [u8; 4],              // b"CHKP"
    format_version: u32,
    entry_id: u64,               // Last entry ID covered by checkpoint
    segment_count: u32,
    segment_list_offset: u64,   // Offset to segment list
    doc_count: u64,
    created_at: u64,             // Unix timestamp
    checksum: u32,               // CRC32 of header
}
```

**Segment List:**
```rust
struct SegmentList {
    segments: Vec<SegmentMetadata>,
}

struct SegmentMetadata {
    segment_id: u64,
    path: String,                // Relative path to segment directory
    doc_count: u32,
    max_doc_id: u32,
    size_bytes: u64,
}
```

**Checkpoint Content**:
- Full copy of all segment files (or hard links if filesystem supports)
- Complete transaction log state (which segments are active)
- All metadata needed to reconstruct index

**Checkpoint Creation Process**:
1. Acquire exclusive lock on transaction log
2. Read current state (active segments, deletes, etc.)
3. Copy all segment files to checkpoint directory
4. Write checkpoint metadata
5. `fsync()` all checkpoint files
6. Write `Checkpoint` entry to WAL
7. `fsync()` WAL
8. Release lock

**Checkpoint Restoration**:
1. Load checkpoint header
2. Verify checksum
3. Load segment list
4. Load all segments referenced in list
5. Replay WAL entries after checkpoint's `entry_id`

## Implementation Plan

### Phase 1: Core Persistence Infrastructure

#### 1.1 Directory Abstraction

```rust
pub trait Directory: Send + Sync {
    fn create_file(&self, path: &str) -> Result<Box<dyn Write>, PersistError>;
    fn open_file(&self, path: &str) -> Result<Box<dyn Read>, PersistError>;
    fn exists(&self, path: &str) -> bool;
    fn delete(&self, path: &str) -> Result<(), PersistError>;
    fn atomic_rename(&self, from: &str, to: &str) -> Result<(), PersistError>;
}

pub struct FsDirectory {
    root: PathBuf,
}

pub struct MemoryDirectory {
    files: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}
```

#### 1.2 Format Definitions

```rust
pub mod format {
    pub const MAGIC_BYTES: [u8; 4] = *b"RANK";
    pub const FORMAT_VERSION: u32 = 1;
    
    pub struct IndexHeader {
        pub magic: [u8; 4],
        pub version: u32,
        pub format_version: u32,
        pub checksum: u32,
    }
    
    // Segment footer, WAL header, checkpoint header, etc.
}
```

#### 1.3 Serialization with Serde

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct PersistentInvertedIndex {
    pub postings: HashMap<String, Vec<(u32, u32)>>, // term -> [(doc_id, tf)]
    pub doc_lengths: HashMap<u32, u32>,
    pub doc_frequencies: HashMap<String, u32>,
    pub num_docs: u32,
    pub avg_doc_length: f32,
}

impl PersistentInvertedIndex {
    pub fn save(&self, path: &Path) -> Result<(), PersistError> {
        // Write header
        // Serialize with bincode
        // Write checksum
        // Atomic rename
    }
    
    pub fn load(path: &Path) -> Result<Self, PersistError> {
        // Read header
        // Verify magic and version
        // Verify checksum
        // Deserialize
    }
}
```

### Phase 2: BM25 Persistent Index

#### 2.1 PersistentInvertedIndex

```rust
pub struct PersistentInvertedIndex {
    directory: Box<dyn Directory>,
    segments: Vec<Segment>,
    wal: Option<Wal>,
    // In-memory cache for active writes
    pending_docs: Vec<(u32, Vec<String>)>,
}

impl PersistentInvertedIndex {
    pub fn create(directory: Box<dyn Directory>) -> Result<Self, PersistError> {
        // Create new index
    }
    
    pub fn open(directory: Box<dyn Directory>) -> Result<Self, PersistError> {
        // Load existing index
        // Replay WAL if needed
    }
    
    pub fn add_document(&mut self, doc_id: u32, terms: &[String]) -> Result<(), PersistError> {
        // Add to pending_docs
        // Write to WAL
        // Optionally flush if threshold reached
    }
    
    pub fn commit(&mut self) -> Result<(), PersistError> {
        // Flush pending docs to new segment
        // Write segment to disk
        // Update metadata
        // Atomic commit
        // Rotate WAL
    }
    
    pub fn retrieve(&self, query: &[String], k: usize, params: Bm25Params) 
        -> Result<Vec<(u32, f32)>, RetrieveError> {
        // Search across all segments
        // Merge results
    }
}
```

#### 2.2 Segment Loading

```rust
impl Segment {
    pub fn load(directory: &dyn Directory, segment_name: &str) -> Result<Self, PersistError> {
        // Read footer
        // Load term dictionary (FST)
        // Memory-map postings (optional)
        // Load doc lengths
    }
    
    pub fn search(&self, query: &[String], params: Bm25Params) -> Vec<(u32, f32)> {
        // Lookup terms in FST
        // Decode postings lists
        // Score documents
    }
}
```

### Phase 3: Dense Retrieval Persistence

#### 3.1 PersistentDenseRetriever

```rust
pub struct PersistentDenseRetriever {
    directory: Box<dyn Directory>,
    embeddings: Vec<Vec<f32>>,  // Or memory-mapped
    doc_ids: Vec<u32>,
}

impl PersistentDenseRetriever {
    pub fn save(&self, path: &Path) -> Result<(), PersistError> {
        // Save embeddings (bincode or custom format)
        // Save doc_ids
        // Save metadata
    }
    
    pub fn load(path: &Path) -> Result<Self, PersistError> {
        // Load embeddings
        // Load doc_ids
        // Verify integrity
    }
}
```

### Phase 4: WAL and Incremental Updates

#### 4.1 Write-Ahead Log

```rust
pub struct Wal {
    directory: Box<dyn Directory>,
    file: File,
    entries: Vec<WalEntry>,
}

impl Wal {
    pub fn append(&mut self, entry: WalEntry) -> Result<(), PersistError> {
        // Serialize entry
        // Append to WAL file
        // Flush to disk
    }
    
    pub fn replay(&self, index: &mut PersistentInvertedIndex) -> Result<(), PersistError> {
        // Read all entries
        // Apply to index
    }
}
```

#### 4.2 Checkpointing

**Checkpoint Strategy:**
- **Periodic checkpoints**: After N entries or N bytes written
- **Background thread**: Checkpointing doesn't block writes
- **Incremental**: Checkpoint contains all segments + metadata
- **WAL truncation**: After successful checkpoint, delete WAL segments covered by checkpoint

```rust
impl PersistentInvertedIndex {
    /// Create a checkpoint covering all entries up to entry_id
    pub fn checkpoint(&self, checkpoint_name: &str, up_to_entry_id: u64) -> Result<(), PersistError> {
        // 1. Write all segments to checkpoint directory
        // 2. Write metadata (segment list, doc_count, etc.)
        // 3. Write checkpoint record to WAL
        // 4. fsync checkpoint files
        // 5. fsync WAL
        // 6. Delete WAL segments with entries <= up_to_entry_id
    }
    
    /// Restore from checkpoint + replay WAL
    pub fn restore_from_checkpoint(checkpoint_path: &Path) -> Result<Self, PersistError> {
        // 1. Load checkpoint (segments + metadata)
        // 2. Find last entry_id in checkpoint
        // 3. Replay WAL entries with entry_id > checkpoint's last_entry_id
        // 4. Reconstruct in-memory index state
    }
}
```

**Segment Merging Strategy (Tiered Merge Policy):**

Based on Lucene/Tantivy best practices:

```rust
pub struct MergePolicy {
    /// Maximum number of segments per tier
    pub segments_per_tier: usize,  // Default: 10
    
    /// Maximum segment size (bytes)
    pub max_segment_size: u64,     // Default: 5GB
    
    /// Maximum number of segments to merge at once
    pub max_merge_at_once: usize,   // Default: 10
    
    /// Minimum segment size to consider for merging
    pub min_segment_size: u64,     // Default: 10MB
}

impl MergePolicy {
    /// Select segments to merge using tiered strategy
    /// 
    /// Strategy:
    /// - Group segments by size tier
    /// - Merge segments of similar size within each tier
    /// - Prefer merging segments with high delete ratios
    /// - Avoid full index merges unless necessary
    pub fn select_segments_to_merge(&self, segments: &[Segment]) -> Vec<Vec<SegmentId>> {
        // Tiered merge: group by size, merge within tiers
        // Returns list of segment groups to merge
    }
}
```

**Best Practices:**
- **Target**: Keep small, stable number of segments (low tens, not hundreds)
- **Avoid**: Constant full merges (only for static indexes or bulk loads)
- **Monitor**: Segment count, merge CPU/IO, query latency
- **Throttle**: Limit concurrent merges to avoid starving queries

## API Design

### Builder Pattern

```rust
// In-memory (existing API)
let mut index = InvertedIndex::new();
index.add_document(0, &["term1".to_string()]);

// Persistent (new API)
let index = PersistentInvertedIndex::builder()
    .directory(FsDirectory::new("/path/to/index")?)
    .create()?;
index.add_document(0, &["term1".to_string()])?;
index.commit()?;

// Open existing
let index = PersistentInvertedIndex::open(FsDirectory::new("/path/to/index")?)?;
```

### Unified API (Optional)

```rust
pub enum IndexBackend {
    Memory(InvertedIndex),
    Persistent(PersistentInvertedIndex),
}

impl IndexBackend {
    pub fn retrieve(&self, query: &[String], k: usize, params: Bm25Params) 
        -> Result<Vec<(u32, f32)>, RetrieveError> {
        match self {
            Self::Memory(idx) => idx.retrieve(query, k, params),
            Self::Persistent(idx) => idx.retrieve(query, k, params),
        }
    }
}
```

## Implementation Details

### 1. Atomic Commits

```rust
fn atomic_write(directory: &dyn Directory, path: &str, data: &[u8]) -> Result<(), PersistError> {
    let temp_path = format!("{}.tmp", path);
    let mut file = directory.create_file(&temp_path)?;
    file.write_all(data)?;
    file.sync_all()?;  // fsync
    drop(file);
    directory.atomic_rename(&temp_path, path)?;
    Ok(())
}
```

### 2. Format Versioning

```rust
pub struct FormatVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl FormatVersion {
    pub fn is_compatible(&self, other: &FormatVersion) -> bool {
        // Same major version = compatible
        self.major == other.major
    }
    
    pub fn migrate(&self, data: &[u8]) -> Result<Vec<u8>, PersistError> {
        // Migration logic
    }
}
```

### 3. Compression (Tantivy-Inspired)

Based on Tantivy's proven approach, use **delta encoding + SIMD bitpacking** for optimal compression and decompression performance:

```rust
pub mod codec {
    /// Block size for bitpacking (matches Tantivy's 128-doc blocks)
    pub const BLOCK_SIZE: usize = 128;
    
    /// Encode document IDs using delta encoding + bitpacking
    /// 
    /// Format for a block of 128 doc IDs:
    /// - [bit_width: u8] (1 byte indicating bits per value)
    /// - [bitpacked_deltas: variable] (128 delta values, bitpacked)
    /// 
    /// For partial blocks (< 128 docs), use varint encoding instead
    pub fn encode_docids(docids: &[u32]) -> Vec<u8> {
        if docids.is_empty() {
            return Vec::new();
        }
        
        // Delta encoding: store differences between consecutive IDs
        let mut deltas = Vec::with_capacity(docids.len());
        deltas.push(docids[0]); // First ID stored as-is
        for i in 1..docids.len() {
            deltas.push(docids[i] - docids[i - 1]);
        }
        
        // For full blocks (128 docs), use bitpacking
        if docids.len() == BLOCK_SIZE {
            let max_delta = *deltas.iter().max().unwrap();
            let bit_width = (max_delta as f64).log2().ceil() as u8;
            let bit_width = bit_width.max(1); // At least 1 bit
            
            // Bitpack: pack 128 values using bit_width bits each
            let mut encoded = vec![bit_width];
            // ... bitpacking implementation (use SIMD if available)
            encoded
        } else {
            // Partial block: use varint encoding
            encode_varint_deltas(&deltas)
        }
    }
    
    /// Encode term frequencies using bitpacking (no delta encoding)
    /// TFs don't have monotonic pattern, so only bitpack
    pub fn encode_term_frequencies(tfs: &[u32]) -> Vec<u8> {
        if tfs.is_empty() {
            return Vec::new();
        }
        
        if tfs.len() == BLOCK_SIZE {
            let max_tf = *tfs.iter().max().unwrap();
            let bit_width = (max_tf as f64).log2().ceil() as u8;
            let bit_width = bit_width.max(1);
            
            let mut encoded = vec![bit_width];
            // ... bitpacking implementation
            encoded
        } else {
            encode_varint_sequence(tfs)
        }
    }
    
    /// Varint encoding for partial blocks and small values
    pub fn encode_varint(value: u64) -> Vec<u8> {
        let mut result = Vec::new();
        let mut v = value;
        loop {
            let byte = (v & 0x7F) as u8;
            v >>= 7;
            if v == 0 {
                result.push(byte | 0x80); // Last byte has MSB set
                break;
            } else {
                result.push(byte);
            }
        }
        result
    }
    
    /// Decode varint from byte stream
    pub fn decode_varint(data: &[u8]) -> Result<(u64, usize), CodecError> {
        let mut result = 0u64;
        let mut shift = 0;
        let mut bytes_read = 0;
        
        for &byte in data {
            bytes_read += 1;
            result |= ((byte & 0x7F) as u64) << shift;
            if (byte & 0x80) != 0 {
                return Ok((result, bytes_read));
            }
            shift += 7;
            if shift >= 64 {
                return Err(CodecError::Overflow);
            }
        }
        Err(CodecError::Incomplete)
    }
    
    /// Gap-encode document IDs (delta encoding)
    fn encode_varint_deltas(deltas: &[u32]) -> Vec<u8> {
        let mut result = Vec::new();
        for &delta in deltas {
            result.extend_from_slice(&encode_varint(delta as u64));
        }
        result
    }
    
    /// Encode sequence using varint
    fn encode_varint_sequence(values: &[u32]) -> Vec<u8> {
        let mut result = Vec::new();
        for &value in values {
            result.extend_from_slice(&encode_varint(value as u64));
        }
        result
    }
}
```

**Key Design Decisions:**
- **128-doc blocks**: Optimal balance between compression ratio and decompression overhead
- **Delta encoding**: Exploits sorted docID property (deltas are typically small)
- **Bitpacking**: Dense packing using minimum bits per value (SIMD-accelerated when available)
- **Varint fallback**: For partial blocks, avoids wasting bits
- **Interleaved layout**: DocIDs and TFs stored together for cache locality

### 4. Memory Mapping (Optional)

```rust
use memmap2::MmapOptions;

impl Segment {
    pub fn memory_map_postings(&self) -> Result<Mmap, PersistError> {
        let file = File::open(&self.postings_path)?;
        unsafe {
            MmapOptions::new()
                .map(&file)
                .map_err(PersistError::from)
        }
    }
}
```

## On-Disk Format Specifications

### Segment Format (Detailed)

**Complete Segment Structure:**
```
segment_{segmentID}/
├── term_dict.fst          # FST: term → ordinal (u64)
├── term_info.bin          # Array[TermInfo] indexed by ordinal
├── postings.bin           # Compressed postings lists
├── doc_lengths.bin        # Array[u32] of document lengths
├── docid_to_userid.fst    # FST: docID → userID (optional)
├── userid_to_docid.fst    # FST: userID → docID range (optional)
├── tombstones.bin         # Bitset: 1 byte per doc (0=alive, 1=deleted)
└── footer.bin             # SegmentFooter (fixed size, at end)
```

**Term Dictionary (FST):**
- **Format**: Standard FST format (fst crate)
- **Key**: Term string (UTF-8)
- **Value**: Ordinal (u64) used to index into `term_info.bin`
- **Size**: Typically 20-30% of raw term size (prefix compression)

**Term Info Store:**
```rust
#[repr(C)]
struct TermInfo {
    postings_offset: u64,      // Offset into postings.bin
    postings_len: u64,          // Length of postings list
    doc_frequency: u32,         // Number of documents containing term
    collection_frequency: u64,  // Total term frequency across all docs
}
```

**Postings Format:**
```
[block_0][block_1]...[block_n]
```

Each block (128 docs):
```rust
struct PostingsBlock {
    bit_width: u8,              // Bits per delta (docIDs) or value (TFs)
    docid_block: [u8],          // Bitpacked docID deltas (128 values)
    tf_block: [u8],             // Bitpacked term frequencies (128 values)
}
```

For partial blocks (< 128 docs):
```rust
struct PostingsBlockPartial {
    count: u8,                  // Number of docs in block
    docid_varints: [u8],        // Varint-encoded docID deltas
    tf_varints: [u8],           // Varint-encoded term frequencies
}
```

**Document Lengths:**
```rust
// Array of u32, one per document in segment
// Stored as: [u32; doc_count] (little-endian)
// Can be bitpacked if all lengths are small
```

**Tombstones:**
```rust
// Bitset: 1 byte per document
// 0 = document is alive
// 1 = document is deleted
// Size: ceil(doc_count / 8) bytes (or 1 byte per doc for simplicity)
```

**Footer (Fixed Size, 64 bytes):**
```rust
#[repr(C)]
struct SegmentFooter {
    magic: [u8; 4],              // b"RANK"
    format_version: u32,          // Format version (currently 1)
    term_dict_offset: u64,       // Offset to term_dict.fst
    term_dict_len: u64,           // Length of term_dict.fst
    term_info_offset: u64,       // Offset to term_info.bin
    term_info_len: u64,           // Length of term_info.bin
    postings_offset: u64,        // Offset to postings.bin
    postings_len: u64,            // Length of postings.bin
    doc_lengths_offset: u64,     // Offset to doc_lengths.bin
    doc_lengths_len: u64,         // Length of doc_lengths.bin
    tombstones_offset: u64,       // Offset to tombstones.bin
    tombstones_len: u64,          // Length of tombstones.bin
    doc_count: u32,               // Number of documents in segment
    max_doc_id: u32,              // Maximum doc ID in segment
    checksum: u32,                // CRC32 of all data (excluding footer)
    padding: [u8; 4],             // Padding to 64-byte alignment
}
```

### Transaction Log Format

**Transaction Log Structure:**
```
transaction.log:
[header][entry_0][entry_1]...[entry_n]
```

**Header:**
```rust
#[repr(C)]
struct TransactionLogHeader {
    magic: [u8; 4],              // b"TXLO"
    format_version: u32,          // Format version
    entry_count: u64,             // Total number of entries
    last_entry_id: u64,           // Last entry ID written
    last_checkpoint_entry_id: u64, // Last entry ID covered by checkpoint
}
```

**Entry Format:**
```rust
struct TransactionLogEntry {
    entry_id: u64,                // Monotonically increasing entry ID
    entry_type: u8,               // 0=AddSegment, 1=DeleteSegment, 2=StartMerge, etc.
    timestamp: u64,                // Unix timestamp
    checksum: u32,                // CRC32 of payload
    payload_len: u32,             // Length of payload
    payload: [u8],                // Serialized entry (bincode)
}
```

**Entry Types:**
- `0`: `AddSegment { segment_id: u64, doc_count: u32 }`
- `1`: `DeleteSegment { segment_id: u64 }`
- `2`: `StartMerge { transaction_id: u64, segment_ids: Vec<u64> }`
- `3`: `EndMerge { transaction_id: u64, new_segment_id: u64, old_segment_ids: Vec<u64> }`
- `4`: `DeleteDocuments { segment_id: u64, doc_ids: Vec<u32> }`
- `5`: `Checkpoint { checkpoint_path: String, last_entry_id: u64 }`

### ID Generation File Format

**File**: `id_generator.bin` (4 u64 values, 32 bytes total)

```rust
#[repr(C)]
struct IdGenerator {
    next_segment_id: u64,         // Next segment ID to allocate
    next_entry_id: u64,           // Next transaction log entry ID
    next_handle_id: u64,          // Next reader handle ID
    next_transaction_id: u64,     // Next merge transaction ID
}
```

**Write Protocol** (crash-safe):
1. Read current values
2. Increment desired field
3. Write all 4 values to temp file: `id_generator.bin.tmp`
4. `fsync()` temp file
5. Atomic rename: `rename("id_generator.bin.tmp", "id_generator.bin")`
6. `fsync()` parent directory

**Recovery Protocol**:
1. If `id_generator.bin.tmp` exists, delete it (incomplete write)
2. Read `id_generator.bin`
3. Verify values are reasonable (not all zeros, not max values)
4. If invalid, scan transaction log to reconstruct IDs

## Index Synchronization

### Reader-Writer Coordination

**Writer Responsibilities:**
1. Acquire exclusive lock on transaction log before any mutations
2. Write all mutations to transaction log
3. `fsync()` transaction log
4. Update segment files
5. `fsync()` segment files
6. Release lock

**Reader Responsibilities:**
1. Acquire shared lock on transaction log
2. Read transaction log to determine active segments
3. Open all active segments (memory map or load)
4. Create handle file: `handles/{handleID}.active`
5. Release shared lock (but keep handle file open with advisory lock)
6. Perform queries using loaded segments
7. On close: delete handle file

**Handle File Format:**
```rust
#[repr(C)]
struct HandleFile {
    handle_id: u64,               // Unique handle identifier
    process_id: u32,              // Process ID of reader
    thread_id: u64,               // Thread ID (if available)
    opened_at: u64,                // Unix timestamp
    last_seen_entry_id: u64,      // Last transaction log entry ID seen
}
```

**Stale Reader Detection:**
- Writer periodically scans `handles/` directory
- For each handle file, check if process is still alive (platform-specific)
- If process is dead, mark handle as stale
- During merge, skip segments that have active readers (unless forced)

### Merge Coordination

**Merge Process:**
1. Writer selects segments to merge (based on merge policy)
2. Acquire exclusive lock on transaction log
3. Write `StartMerge` entry to transaction log
4. Create merge indicator file: `merges/{transactionID}.active`
5. `fsync()` transaction log
6. Release lock
7. Perform merge (creates new segment)
8. Acquire exclusive lock again
9. Write `EndMerge` entry to transaction log
10. Delete old segment directories
11. Delete merge indicator file
12. `fsync()` transaction log
13. Release lock

**Merge Indicator File:**
```rust
#[repr(C)]
struct MergeIndicator {
    transaction_id: u64,           // Unique merge transaction ID
    segment_ids: Vec<u64>,        // Segments being merged
    started_at: u64,              // Unix timestamp
}
```

**Handling Stale Merges:**
- On startup, scan `merges/` directory
- For each merge indicator, check if merge output exists
- If output exists but merge indicator is old (>1 hour), assume merge completed
- Write `EndMerge` entry to transaction log (if missing)
- Delete merge indicator
- If output doesn't exist, assume merge failed, delete indicator

## Recovery Procedures

### Startup Recovery

**Recovery Steps:**
1. **Check for lock files**: If transaction log is locked, wait or fail (another process is active)
2. **Load ID generator**: Read `id_generator.bin`, verify integrity
3. **Load transaction log**: Read all entries from `transaction.log`
4. **Detect incomplete writes**: Check for `.tmp` files, delete if found
5. **Reconstruct active segments**: From transaction log, determine which segments are active
6. **Detect stale merges**: Scan `merges/` directory, handle stale merge indicators
7. **Detect stale handles**: Scan `handles/` directory, remove handles for dead processes
8. **Replay WAL if needed**: If WAL exists and is newer than last checkpoint, replay entries
9. **Verify segment integrity**: Check segment footers, verify checksums
10. **Mark index as ready**: Create or update `index.ready` file

### WAL Replay

**Replay Process:**
1. Find latest checkpoint (highest `entry_id` in `checkpoints/`)
2. Load checkpoint (if exists)
3. Find WAL segments with `start_entry_id > checkpoint.entry_id`
4. For each WAL segment:
   - Read header, verify magic and version
   - Read entries sequentially
   - For each entry:
     - Verify checksum
     - Apply entry to in-memory state
5. After replay, verify consistency (all referenced segments exist)
6. If consistency check fails, fall back to last checkpoint

**Entry Application:**
- `AddSegment`: Add segment to active list
- `DeleteSegment`: Remove segment from active list
- `StartMerge`: Mark segments as being merged
- `EndMerge`: Remove old segments, add new segment
- `DeleteDocuments`: Mark documents as deleted in segment
- `Checkpoint`: Update last checkpoint reference

### Crash Recovery Scenarios

**Scenario 1: Crash During Segment Write**
- **Detection**: Segment directory exists but `footer.bin` is missing or invalid
- **Recovery**: Delete incomplete segment directory, replay WAL to reconstruct

**Scenario 2: Crash During Transaction Log Write**
- **Detection**: Transaction log has incomplete entry (checksum mismatch)
- **Recovery**: Truncate transaction log to last valid entry, replay WAL

**Scenario 3: Crash During Merge**
- **Detection**: Merge indicator exists but no `EndMerge` entry in transaction log
- **Recovery**: Check if merge output exists:
  - If exists: Write `EndMerge` entry, delete indicator
  - If missing: Delete indicator, segments remain as-is

**Scenario 4: Crash During Checkpoint**
- **Detection**: Checkpoint directory exists but incomplete
- **Recovery**: Delete incomplete checkpoint, use previous checkpoint (if exists)

**Scenario 5: Power Failure During fsync**
- **Detection**: File exists but may be partially written
- **Recovery**: Verify checksums, if invalid, fall back to previous version

### Consistency Verification

**Verification Steps:**
1. **Transaction log consistency**: All entry IDs are sequential, no gaps
2. **Segment references**: All segments referenced in transaction log exist
3. **Segment integrity**: All segment footers are valid, checksums match
4. **ID generator consistency**: IDs in generator are >= max IDs found in segments/log
5. **Handle consistency**: All handle files reference valid processes
6. **Merge consistency**: No merge indicators without corresponding transaction log entries

**Repair Actions:**
- **Missing segment**: Remove from transaction log, log warning
- **Invalid segment**: Mark as deleted, log error
- **ID generator mismatch**: Reconstruct from transaction log
- **Stale handle**: Delete handle file
- **Stale merge**: Complete or cancel merge based on output existence

## Migration Strategy

### Backward Compatibility

1. **Keep in-memory API unchanged**: All existing code continues to work
2. **Feature flag**: `persistence` feature gate for disk support
3. **Gradual migration**: Users can migrate indexes incrementally

### Version Migration

1. **Format version in header**: Check on load
2. **Migration functions**: Convert old format to new format
3. **Automatic migration**: Migrate on first load after upgrade

## Testing Strategy

### Unit Tests
- Format serialization/deserialization
- Atomic commit correctness
- WAL replay
- Checkpoint creation/restoration

### Integration Tests
- Full index save/load cycle
- Crash recovery (simulate crashes during writes)
- Concurrent readers with writer
- Large index persistence (1M+ documents)

### Property Tests
- Round-trip: save → load → verify same results
- WAL replay produces same state as direct writes
- Checkpoint restoration produces same state

## Performance Considerations

### Write Performance
- **Batch commits**: Accumulate multiple document additions before committing
  - Default: Commit every 1000 documents or 10MB, whichever comes first
  - Configurable threshold based on workload
- **Async I/O**: Use `tokio::fs` or `async-std::fs` for non-blocking writes (optional)
- **Compression**: 
  - WAL: No compression (append-only, needs fast writes)
  - Checkpoints: LZ4 compression (fast, good ratio)
  - Segments: Bitpacking (already compressed, no additional compression needed)
- **Preallocation**: Preallocate WAL segment files to avoid fragmentation
- **Batch fsync**: Queue multiple writers, single fsync for throughput

### Read Performance
- **Memory mapping**: Memory-map postings lists for zero-copy reads
  - Use `memmap2` crate for cross-platform memory mapping
  - Map entire segment files or just postings sections
- **Caching**: 
  - Cache frequently accessed segments in memory
  - Cache FST term dictionary (small, fits in memory)
  - Cache TermInfoStore metadata array
- **Lazy loading**: Load segments on-demand (not all at startup)
- **SIMD acceleration**: 
  - Use SIMD bitpacking for 4-8x faster decompression
  - SSE3: 4 integers in parallel
  - AVX2: 8 integers in parallel
  - Fallback to scalar implementation when SIMD unavailable

### Space Efficiency
- **Compression**: 
  - Delta encoding: Typically 50-80% reduction for docIDs
  - Bitpacking: Additional 30-50% reduction (varies by data distribution)
  - Combined: Often <10% of raw document size for inverted index
- **Deduplication**: Share common postings across segments (via merging)
- **Segment merging**: 
  - Merge small segments to reduce overhead
  - Target: 5-10 segments per index (not hundreds)
  - Tiered merge policy avoids expensive full merges

### Query Performance
- **Skip lists**: Enable fast seeking within postings lists
  - Store block metadata (max docID per block)
  - Binary search to find relevant block
  - Branchless binary search within 128-doc blocks (11% improvement)
- **Block-Max WAND**: Skip entire blocks when they can't contribute to top-K
  - Maintain per-block maximum score bounds
  - Skip blocks below threshold
  - 5-10x speedup for large corpora
- **Segment search**: 
  - Search segments in parallel (if multiple segments)
  - Merge results using heap-based top-K
  - Optimized TopNComputer (15% improvement over binary heap)

### Memory Usage
- **Indexing**: 
  - Writer buffer: 50-100MB default (configurable)
  - Document ID deltas instead of absolute IDs: 22% memory reduction
  - CompactDoc instead of TantivyDocument: Reduced overhead
- **Query time**:
  - Load only segments needed for query
  - Memory-map postings (OS manages paging)
  - FST term dictionary: Typically <1MB even for large vocabularies

## Alternative Approaches

### Option 1: Custom Format (Recommended for rank-retrieve)

**Pros:**
- Full control over format and optimization
- Minimal dependencies
- Optimized for retrieval use case
- Can reuse Tantivy patterns (segments, FST)

**Cons:**
- More implementation work
- Need to handle versioning, migration
- Must implement compression, WAL, etc.

**Best for:** Production systems needing maximum control and performance

### Option 2: Embedded Database (sled)

**Pros:**
- Built-in ACID guarantees
- Automatic crash recovery
- Incremental updates via batches/merges
- Thread-safe, concurrent access
- Simple API (`put`, `get`, `scan`)

**Cons:**
- Less control over storage format
- May not be optimized for retrieval patterns
- Additional dependency

**Best for:** Rapid prototyping, when you need ACID guarantees quickly

**Example Integration:**
```rust
use sled::Db;

pub struct SledPersistentIndex {
    db: Db,
    postings_tree: sled::Tree,
    doc_lengths_tree: sled::Tree,
}

impl SledPersistentIndex {
    pub fn new(path: &Path) -> Result<Self, PersistError> {
        let db = sled::open(path)?;
        let postings_tree = db.open_tree("postings")?;
        let doc_lengths_tree = db.open_tree("doc_lengths")?;
        Ok(Self { db, postings_tree, doc_lengths_tree })
    }
    
    pub fn add_document(&self, doc_id: u32, terms: &[String]) -> Result<(), PersistError> {
        let mut batch = sled::Batch::default();
        
        // Store term -> doc_id mappings
        for term in terms {
            let key = format!("{}:{}", term, doc_id);
            batch.insert(key.as_bytes(), &1u32.to_le_bytes());
        }
        
        // Store doc length
        batch.insert(
            &doc_id.to_le_bytes(),
            &(terms.len() as u32).to_le_bytes()
        );
        
        self.postings_tree.apply_batch(batch)?;
        Ok(())
    }
    
    pub fn commit(&self) -> Result<(), PersistError> {
        self.db.flush()?;  // fsync
        Ok(())
    }
}
```

### Option 3: Embedded Database (RocksDB)

**Pros:**
- Battle-tested, production-ready
- Excellent write performance
- Column families for logical separation
- Configurable compaction strategies
- Bloom filters for fast lookups

**Cons:**
- C++ dependency (via FFI)
- More complex configuration
- Larger binary size

**Best for:** Production systems needing maximum write throughput

**Example Integration:**
```rust
use rocksdb::{DB, Options, WriteBatch, ColumnFamilyDescriptor};

pub struct RocksDBPersistentIndex {
    db: DB,
}

impl RocksDBPersistentIndex {
    pub fn new(path: &Path) -> Result<Self, PersistError> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        
        let cfs = vec![
            ColumnFamilyDescriptor::new("postings", Options::default()),
            ColumnFamilyDescriptor::new("doc_lengths", Options::default()),
        ];
        
        let db = DB::open_cf_descriptors(&opts, path, cfs)?;
        Ok(Self { db })
    }
    
    pub fn add_document(&self, doc_id: u32, terms: &[String]) -> Result<(), PersistError> {
        let mut batch = WriteBatch::default();
        let postings_cf = self.db.cf_handle("postings").unwrap();
        
        for term in terms {
            let key = format!("{}:{}", term, doc_id);
            batch.put_cf(postings_cf, key.as_bytes(), &1u32.to_le_bytes());
        }
        
        let doc_lengths_cf = self.db.cf_handle("doc_lengths").unwrap();
        batch.put_cf(doc_lengths_cf, &doc_id.to_le_bytes(), &(terms.len() as u32).to_le_bytes());
        
        self.db.write(batch)?;
        Ok(())
    }
}
```

### Recommendation

**For rank-retrieve, recommend Option 1 (Custom Format)** because:
1. **Optimized for retrieval**: Can use FST for term dictionary, gap-encoded postings
2. **Minimal dependencies**: Only serde, bincode, fst (all pure Rust)
3. **Full control**: Can optimize for specific retrieval patterns
4. **Reuses proven patterns**: Follows Tantivy's successful architecture

**Consider Option 2 (sled) or Option 3 (RocksDB) if:**
- Need rapid prototyping
- Want ACID guarantees without implementation
- Have complex multi-key transactions
- Need to store additional metadata alongside index

## Dependencies

### Option 1: Custom Format (Recommended)

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
bincode = "1.3"
fst = "0.4"              # For term dictionary
memmap2 = "0.9"          # Optional: memory mapping
crc32fast = "1.3"        # For checksums

[features]
persistence = ["dep:serde", "dep:bincode", "dep:fst"]
```

### Option 2: sled

```toml
[dependencies]
sled = "0.34"

[features]
persistence = ["dep:sled"]
persistence-sled = ["persistence", "dep:sled"]
```

### Option 3: RocksDB

```toml
[dependencies]
rocksdb = "0.21"

[features]
persistence = []
persistence-rocksdb = ["persistence", "dep:rocksdb"]
```

## Future Enhancements

1. **Segment merging**: Automatic tiered merging of small segments (Phase 7)
2. **Compression options**: Configurable compression levels (none, LZ4, Zstd) for checkpoints
3. **Distributed storage**: Support for S3, GCS via Directory trait (future: `S3Directory`, `GCSDirectory`)
4. **Incremental indexing**: Add documents without full reindex (Phase 5: WAL)
5. **Snapshot/backup**: Efficient snapshot creation for backups (Phase 6: Checkpointing)
6. **Query caching**: Cache query results for repeated queries (application-level)
7. **Block-Max WAND**: Advanced query optimization for skipping blocks (research phase)
8. **Skip lists**: Block metadata for fast seeking (Phase 8: Optimization)
9. **SIMD bitpacking**: SSE3/AVX2 acceleration for decompression (Phase 8)
10. **Pulsing**: Inline single-doc postings in term dictionary (advanced optimization)
11. **Columnar fast fields**: Efficient random access to document metadata (future)
12. **Position storage**: Optional term positions for phrase queries (future)

## Implementation Priority

### Phase 1: Core Infrastructure (Week 1-2)
1. Directory abstraction trait
2. Format definitions (header, footer, versioning)
3. Basic serialization (serde + bincode)
4. Atomic commit implementation

### Phase 2: BM25 Persistence (Week 3-4)
1. PersistentInvertedIndex implementation
2. Segment format and loading
3. Basic save/load cycle
4. Unit tests for round-trip

### Phase 3: WAL and Incremental Updates (Week 5-6)
1. Write-ahead log implementation
2. Incremental document addition
3. WAL replay on startup
4. Checkpoint creation

### Phase 4: Optimization (Week 7-8)
1. FST term dictionary
2. Compressed postings (varint, gap encoding)
3. Memory mapping for reads
4. Segment merging

### Phase 5: Dense/Sparse Persistence (Week 9-10)
1. PersistentDenseRetriever
2. PersistentSparseRetriever
3. Unified persistence API

## Concrete Implementation Example

### Complete Save/Load Cycle

```rust
use rank_retrieve::persistence::*;
use rank_retrieve::bm25::{InvertedIndex, Bm25Params};

// Create persistent index
let directory = FsDirectory::new("/path/to/index")?;
let mut persistent_index = PersistentInvertedIndex::create(directory)?;

// Add documents (buffered, written to WAL)
persistent_index.add_document(0, &["machine".to_string(), "learning".to_string()])?;
persistent_index.add_document(1, &["artificial".to_string(), "intelligence".to_string()])?;

// Commit (flushes to segment, fsyncs, atomic)
persistent_index.commit()?;

// Query (searches across all segments)
let query = vec!["machine".to_string()];
let results = persistent_index.retrieve(&query, 10, Bm25Params::default())?;

// Later: Open existing index
let directory = FsDirectory::new("/path/to/index")?;
let index = PersistentInvertedIndex::open(directory)?; // Replays WAL if needed
let results = index.retrieve(&query, 10, Bm25Params::default())?;
```

### Segment Format Example

```
segment_abc123/
├── term_dict.fst          # FST mapping term → ordinal
├── term_info.bin          # Array[TermInfo] indexed by ordinal
├── postings.bin           # Compressed postings lists
├── doc_lengths.bin        # Document length array
└── footer.bin             # Segment metadata and offsets
```

**Footer structure:**
```rust
#[repr(C)]
struct SegmentFooter {
    magic: [u8; 4],              // b"RANK"
    format_version: u32,          // 1
    term_dict_offset: u64,        // Offset to term_dict.fst
    term_dict_len: u64,
    term_info_offset: u64,        // Offset to term_info.bin
    term_info_len: u64,
    postings_offset: u64,         // Offset to postings.bin
    postings_len: u64,
    doc_lengths_offset: u64,
    doc_lengths_len: u64,
    doc_count: u32,
    max_doc_id: u32,
    checksum: u32,                // CRC32 of all data
}
```

### WAL Entry Example

```rust
// On-disk format:
// [length: u32][type: u8][checksum: u32][payload: variable]

// Example: AddDocument entry
let entry = WalEntry::AddDocument {
    entry_id: 42,
    doc_id: 100,
    terms: vec!["apple".to_string(), "banana".to_string()],
};

// Serialize with bincode
let payload = bincode::serialize(&entry)?;
let checksum = crc32fast::hash(&payload);
let length = (1 + 4 + payload.len()) as u32; // type + checksum + payload

// Write to WAL
wal_file.write_all(&length.to_le_bytes())?;
wal_file.write_all(&[0u8])?; // type: AddDocument
wal_file.write_all(&checksum.to_le_bytes())?;
wal_file.write_all(&payload)?;
wal_file.sync_all()?; // fsync
```

## Failure Modes and Edge Cases

### Long-Running Readers

**Problem**: If a reader keeps a handle open for a long time, it prevents compaction from removing old segments.

**Solution**:
- Implement auto-update mechanism: Reader can update its snapshot periodically
- Configurable timeout: After N minutes, force snapshot update
- Monitor oldest reader: Log warning if oldest reader is very old

### WAL File Growth

**Problem**: If checkpoints cannot complete (due to long-running readers), WAL files grow unbounded.

**Solution**:
- Monitor WAL size: Log warning if WAL exceeds threshold
- Force checkpoint: If WAL grows too large, use blocking checkpoint (may block readers)
- Manual checkpoint: Provide API for applications to trigger checkpoints

### Merge Starvation

**Problem**: If merges keep getting cancelled or fail, segments accumulate.

**Solution**:
- Retry logic: Automatically retry failed merges
- Merge priority: Prefer merging segments with high delete ratios
- Force merge: Allow forcing merge of specific segments (for maintenance)

### Concurrent Delete and Merge

**Problem**: Document is deleted in segment that's being merged.

**Solution**:
- Remap deletes: During merge, remap deletes from old segments to new segment
- Transaction log coordination: `EndMerge` entry includes remapped deletes
- Tombstone writes: Write tombstones to new segment if delete occurred during merge

### Power Failure During Commit

**Problem**: Power fails after fsync reports success but before data is truly on disk.

**Solution**:
- Trust storage: Once fsync reports success, we trust the storage system
- Checksums: Detect corruption via checksums
- Recovery: Replay WAL on startup, discard corrupted entries

### Network Filesystem Issues

**Problem**: File locking may not work correctly on NFS.

**Solution**:
- Detect NFS: Check filesystem type, warn user
- Degraded mode: On NFS, use more conservative locking (may reduce concurrency)
- Documentation: Document NFS limitations

## Performance Benchmarks and Targets

### Write Performance

**Targets**:
- Indexing: 10K-100K documents/second (depends on document size, tokenization)
- Commit latency: <10ms for typical commits (WAL append + fsync)
- Checkpoint: <1 second per GB of index data

**Optimizations**:
- Batch commits: Accumulate multiple documents before committing
- Async checkpointing: Checkpoint in background thread
- Preallocation: Preallocate WAL segment files

### Read Performance

**Targets**:
- Query latency: <10ms for typical queries (2-5 terms, 10K-1M docs)
- Segment loading: <100ms per segment (lazy loading)
- Memory usage: <100MB for typical index (excluding OS page cache)

**Optimizations**:
- Memory mapping: Map segment files for zero-copy reads
- Caching: Cache frequently accessed segments
- SIMD decompression: Use SIMD for bitpacking decompression

### Storage Efficiency

**Targets**:
- Index size: <10% of raw document size (for inverted index)
- Compression ratio: 50-80% reduction via delta encoding + bitpacking
- Segment overhead: <5% overhead from multiple segments

## Testing Strategy

### Unit Tests

- Format serialization/deserialization
- Compression codecs (varint, bitpacking, delta encoding)
- FST construction and lookup
- WAL entry encoding/decoding

### Integration Tests

- Round-trip: Save index, load index, verify correctness
- Incremental updates: Add documents, commit, verify
- Deletes: Delete documents, verify they're excluded from results
- Merges: Merge segments, verify correctness

### Crash Tests

- Power failure simulation: Kill process during various operations
- Corrupted WAL: Test recovery from corrupted WAL entries
- Incomplete segments: Test handling of incomplete segment writes
- Stale locks: Test detection of stale merges/readers

### Performance Tests

- Benchmark indexing throughput
- Benchmark query latency
- Benchmark merge performance
- Memory usage profiling

### Property Tests

- Fuzz testing: Random operations, verify invariants
- Concurrent access: Multiple readers/writers, verify consistency
- Stress tests: Large indexes, many segments, many deletes

## Migration and Compatibility

### Format Versioning

**Version Header**: All on-disk formats include format version in header.

**Migration Strategy**:
- Support reading old formats (backward compatibility)
- Write new formats when possible
- Provide migration tool for major version changes

### Backward Compatibility

**Guarantees**:
- Indexes created with version N can be read by version N+1
- Indexes created with version N+1 cannot be read by version N (forward compatibility not guaranteed)

**Breaking Changes**:
- Major version bump for breaking format changes
- Provide migration tool or clear error message

## References

### Core Technologies
1. **Tantivy Architecture**: https://github.com/quickwit-oss/tantivy/blob/main/ARCHITECTURE.md
2. **Tantivy Bitpacking**: https://fulmicoton.com/posts/bitpacking/ - SIMD-accelerated compression
3. **Tantivy Inverted Index**: https://fulmicoton.gitbooks.io/tantivy-doc/content/inverted-index.html
4. **FST Crate**: https://docs.rs/fst/ - BurntSushi's FST implementation
5. **FST Blog Post**: https://blog.burntsushi.net/transducers/ - FST explanation

### Compression Techniques
6. **Varint Decoding**: https://www.bazhenov.me/posts/rust-stream-vbyte-varint-decoding/ - SIMD varint
7. **PForDelta**: https://dev.to/madhav_baby_giraffe/detailed-explanation-of-pfor-partitioned-frame-of-reference-compression-3bod
8. **Integer Compression**: Research on varint, bitpacking, PForDelta techniques

### WAL and Durability
9. **OkayWAL**: https://github.com/khonsulabs/okaywal - Segmented WAL implementation
10. **WAL Best Practices**: https://www.architecture-weekly.com/p/the-write-ahead-log-a-foundation
11. **Segmented Log**: https://arindas.github.io/blog/segmented-log-rust/ - Rust WAL patterns

### Segment Merging
12. **Tantivy Merge Policy**: https://github.com/quickwit-oss/tantivy/issues/706 - Merge strategies
13. **Lucene TieredMergePolicy**: Best practices for segment compaction

### General
14. **Serde Best Practices**: https://serde.rs/
15. **Atomic File Operations**: POSIX `rename()` semantics
16. **sled**: https://github.com/spacejam/sled - Embedded database with ACID guarantees
17. **RocksDB Rust**: https://docs.rs/rocksdb - Production-ready embedded database
18. **RocksDB Tuning Guide**: https://github.com/facebook/rocksdb/wiki/RocksDB-Tuning-Guide
6. **sled**: https://github.com/spacejam/sled
7. **RocksDB Rust**: https://docs.rs/rocksdb
8. **RocksDB Tuning Guide**: https://github.com/facebook/rocksdb/wiki/RocksDB-Tuning-Guide

## Conclusion

This design reuses established best practices from:
- **Tantivy**: Directory-based storage, segments, atomic commits
- **Serde/Bincode**: Header + version + checksum, atomic writes
- **Inverted Index Formats**: Segmented layout, compressed postings, FST term dictionary
- **sled**: Incremental updates via batches, ACID guarantees
- **RocksDB**: Write batches, column families, compaction strategies

### Recommended Approach

**Option 1 (Custom Format)** is recommended for `rank-retrieve` because:
- Optimized specifically for retrieval patterns (FST, gap-encoded postings)
- Minimal dependencies (pure Rust)
- Full control over format and performance
- Reuses proven patterns from Tantivy

**Alternative approaches (sled, RocksDB)** are viable for:
- Rapid prototyping
- When ACID guarantees are needed immediately
- Complex multi-key transactions
- Additional metadata storage requirements

The implementation will maintain API compatibility while adding optional persistence, following Rust best practices for error handling, memory safety, and performance. All approaches support incremental updates, crash safety, and efficient reads.
