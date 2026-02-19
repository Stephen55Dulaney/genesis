//! Memory Store for Genesis OS
//!
//! Persistent, searchable memory for agents. Insights accumulate across sessions
//! and connections emerge through BM25-lite keyword search.
//!
//! Inspired by OpenClaw's hybrid memory architecture, adapted for `no_std`:
//! - BTreeMap-based inverted index (no hashing crates needed)
//! - BM25-lite scoring with integer math (no floats/log)
//! - Pipe-delimited serialization (no serde)
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │              MemoryStore                │
//! │                                         │
//! │  entries: BTreeMap<id, MemoryEntry>     │
//! │  index:   BTreeMap<keyword, Set<id>>    │
//! │                                         │
//! │  store() ──> extract keywords ──> index │
//! │  search() ─> BM25 score ──> ranked ids │
//! │  serialize() ──> pipe-delimited text   │
//! │  deserialize() <── filesystem load     │
//! └─────────────────────────────────────────┘
//! ```

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use spin::Mutex;
use lazy_static::lazy_static;

/// Path where memories are persisted in the filesystem
const MEMORY_FILE_PATH: &str = "/storage/memory/memories.dat";

/// Stop words to skip during keyword extraction
const STOP_WORDS: &[&str] = &[
    "a", "an", "the", "is", "of", "to", "in", "and", "for", "that",
    "this", "with", "are", "was", "were", "been", "have", "has", "had",
    "but", "not", "from", "they", "will", "can", "would", "could",
];

/// Minimum word length for keyword extraction
const MIN_WORD_LEN: usize = 4;

/// What kind of memory this is — matches FeedbackType categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryKind {
    /// Ideas/insights from agents
    Spark,
    /// Links between concepts
    Connection,
    /// Useful references
    Resource,
    /// Agent state snapshots
    Feeling,
    /// General notes
    Observation,
}

impl MemoryKind {
    /// Convert to string tag for serialization
    pub fn as_str(&self) -> &'static str {
        match self {
            MemoryKind::Spark => "spark",
            MemoryKind::Connection => "connection",
            MemoryKind::Resource => "resource",
            MemoryKind::Feeling => "feeling",
            MemoryKind::Observation => "observation",
        }
    }

    /// Parse from string tag
    pub fn from_str(s: &str) -> Option<MemoryKind> {
        match s {
            "spark" => Some(MemoryKind::Spark),
            "connection" => Some(MemoryKind::Connection),
            "resource" => Some(MemoryKind::Resource),
            "feeling" => Some(MemoryKind::Feeling),
            "observation" => Some(MemoryKind::Observation),
            _ => None,
        }
    }
}

/// A single memory entry
#[derive(Debug, Clone)]
pub struct MemoryEntry {
    /// Unique identifier
    pub id: u64,
    /// Main text content
    pub content: String,
    /// What kind of memory
    pub kind: MemoryKind,
    /// Which agent or "shell" stored this
    pub source: String,
    /// Extracted keywords for the inverted index
    pub keywords: Vec<String>,
    /// Tick when stored
    pub timestamp: u64,
    /// How often this entry has been retrieved (for ranking boost)
    pub access_count: u64,
}

/// Statistics about the memory store
#[derive(Debug, Clone)]
pub struct MemoryStats {
    /// Total number of entries
    pub entry_count: usize,
    /// Number of unique keywords in the index
    pub index_size: usize,
    /// Top keywords by document frequency
    pub top_keywords: Vec<(String, usize)>,
    /// Estimated memory usage in bytes
    pub estimated_bytes: usize,
}

/// The core memory store — BTreeMap-based inverted index with BM25-lite search
pub struct MemoryStore {
    /// All entries keyed by ID
    entries: BTreeMap<u64, MemoryEntry>,
    /// Inverted index: keyword → set of entry IDs
    index: BTreeMap<String, BTreeSet<u64>>,
    /// Next ID to assign
    next_id: u64,
    /// Maximum number of entries (cap for memory budget)
    max_entries: usize,
}

impl MemoryStore {
    /// Create a new MemoryStore with the given capacity
    pub fn new(max_entries: usize) -> Self {
        MemoryStore {
            entries: BTreeMap::new(),
            index: BTreeMap::new(),
            next_id: 1,
            max_entries,
        }
    }

    /// Store a new memory entry. Returns the assigned ID.
    ///
    /// Extracts keywords from content, adds to entries and index,
    /// and enforces the max_entries cap (removes oldest first).
    pub fn store(&mut self, content: &str, kind: MemoryKind, source: &str) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let keywords = extract_keywords(content);

        let entry = MemoryEntry {
            id,
            content: String::from(content),
            kind,
            source: String::from(source),
            keywords: keywords.clone(),
            timestamp: 0, // Will be set by caller if needed
            access_count: 0,
        };

        // Add to entries
        self.entries.insert(id, entry);

        // Add to inverted index
        for keyword in &keywords {
            self.index
                .entry(keyword.clone())
                .or_insert_with(BTreeSet::new)
                .insert(id);
        }

        // Enforce capacity — remove oldest entries
        while self.entries.len() > self.max_entries {
            if let Some(&oldest_id) = self.entries.keys().next() {
                self.remove_entry(oldest_id);
            }
        }

        id
    }

    /// Store with a specific timestamp
    pub fn store_with_timestamp(&mut self, content: &str, kind: MemoryKind, source: &str, timestamp: u64) -> u64 {
        let id = self.store(content, kind, source);
        if let Some(entry) = self.entries.get_mut(&id) {
            entry.timestamp = timestamp;
        }
        id
    }

    /// Search for entries matching the query. Returns (id, score) pairs sorted by score descending.
    ///
    /// BM25-lite scoring:
    ///   score = tf * idf_approx * (1 + access_bonus)
    ///   tf = count of query term matches in entry keywords
    ///   idf_approx = total_entries / doc_freq (integer division)
    ///   access_bonus = min(access_count, 5)
    pub fn search(&self, query: &str) -> Vec<(u64, u32)> {
        let query_terms = extract_keywords(query);
        if query_terms.is_empty() {
            return Vec::new();
        }

        let total_entries = self.entries.len() as u32;
        if total_entries == 0 {
            return Vec::new();
        }

        // Collect candidate entry IDs from all query terms
        let mut scores: BTreeMap<u64, u32> = BTreeMap::new();

        for term in &query_terms {
            if let Some(entry_ids) = self.index.get(term) {
                let doc_freq = entry_ids.len() as u32;
                // IDF approximation: total / doc_freq (at least 1)
                let idf = if doc_freq > 0 { total_entries / doc_freq } else { 1 };

                for &entry_id in entry_ids {
                    if let Some(entry) = self.entries.get(&entry_id) {
                        // TF: count how many times this term appears in entry keywords
                        let tf = entry.keywords.iter()
                            .filter(|kw| kw.as_str() == term.as_str())
                            .count() as u32;

                        // Access bonus: boost frequently-accessed entries (capped at 5)
                        let access_bonus = if entry.access_count > 5 { 5 } else { entry.access_count as u32 };

                        let term_score = tf * idf * (1 + access_bonus);
                        *scores.entry(entry_id).or_insert(0) += term_score;
                    }
                }
            }
        }

        // Sort by score descending
        let mut results: Vec<(u64, u32)> = scores.into_iter().collect();
        results.sort_by(|a, b| b.1.cmp(&a.1));
        results
    }

    /// Get an entry by ID and bump its access count
    pub fn get(&mut self, id: u64) -> Option<&MemoryEntry> {
        if let Some(entry) = self.entries.get_mut(&id) {
            entry.access_count += 1;
        }
        self.entries.get(&id)
    }

    /// Get an entry by ID without bumping access count (for read-only operations)
    pub fn peek(&self, id: u64) -> Option<&MemoryEntry> {
        self.entries.get(&id)
    }

    /// Get statistics about the memory store
    pub fn stats(&self) -> MemoryStats {
        // Find top keywords by document frequency
        let mut keyword_freq: Vec<(String, usize)> = self.index
            .iter()
            .map(|(k, v)| (k.clone(), v.len()))
            .collect();
        keyword_freq.sort_by(|a, b| b.1.cmp(&a.1));
        keyword_freq.truncate(10);

        // Estimate memory usage
        let entry_bytes: usize = self.entries.values()
            .map(|e| {
                // Rough estimate per entry
                8 + e.content.len() + e.source.len()
                    + e.keywords.iter().map(|k| k.len() + 24).sum::<usize>()
                    + 64 // overhead for BTreeMap node, enum, etc.
            })
            .sum();

        let index_bytes: usize = self.index.iter()
            .map(|(k, v)| k.len() + 24 + v.len() * 8 + 48)
            .sum();

        MemoryStats {
            entry_count: self.entries.len(),
            index_size: self.index.len(),
            top_keywords: keyword_freq,
            estimated_bytes: entry_bytes + index_bytes,
        }
    }

    /// Get the N most recent entries
    pub fn recent(&self, count: usize) -> Vec<&MemoryEntry> {
        // BTreeMap is sorted by ID (which is monotonically increasing)
        self.entries.values().rev().take(count).collect()
    }

    /// Serialize the entire store to pipe-delimited text
    ///
    /// Format per line: `id|kind|source|timestamp|access_count|content|kw1,kw2,...\n`
    pub fn serialize(&self) -> String {
        let mut output = String::new();

        for entry in self.entries.values() {
            // Escape pipes and newlines in content
            let escaped_content = entry.content.replace('\\', "\\\\").replace('|', "\\p").replace('\n', "\\n").replace('\r', "\\r");
            let escaped_source = entry.source.replace('\\', "\\\\").replace('|', "\\p").replace('\n', "\\n").replace('\r', "\\r");

            let keywords_str: String = entry.keywords.join(",");

            output.push_str(&format!(
                "{}|{}|{}|{}|{}|{}|{}\n",
                entry.id,
                entry.kind.as_str(),
                escaped_source,
                entry.timestamp,
                entry.access_count,
                escaped_content,
                keywords_str,
            ));
        }

        output
    }

    /// Deserialize from pipe-delimited text, replacing current contents
    pub fn deserialize(&mut self, data: &str) {
        // Clear existing data
        self.entries.clear();
        self.index.clear();
        self.next_id = 1;

        for line in data.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Split on pipe, expecting 7 fields
            let parts: Vec<&str> = line.splitn(7, '|').collect();
            if parts.len() < 7 {
                continue; // Skip malformed lines
            }

            let id = match parts[0].parse::<u64>() {
                Ok(v) => v,
                Err(_) => continue,
            };
            let kind = match MemoryKind::from_str(parts[1]) {
                Some(k) => k,
                None => continue,
            };
            let source = parts[2].replace("\\p", "|").replace("\\r", "\r").replace("\\n", "\n").replace("\\\\", "\\");
            let timestamp = parts[3].parse::<u64>().unwrap_or(0);
            let access_count = parts[4].parse::<u64>().unwrap_or(0);
            let content = parts[5].replace("\\p", "|").replace("\\r", "\r").replace("\\n", "\n").replace("\\\\", "\\");
            let keywords: Vec<String> = if parts[6].is_empty() {
                Vec::new()
            } else {
                parts[6].split(',').map(|s| String::from(s.trim())).collect()
            };

            let entry = MemoryEntry {
                id,
                content,
                kind,
                source,
                keywords: keywords.clone(),
                timestamp,
                access_count,
            };

            // Add to entries
            self.entries.insert(id, entry);

            // Rebuild inverted index
            for keyword in &keywords {
                if !keyword.is_empty() {
                    self.index
                        .entry(keyword.clone())
                        .or_insert_with(BTreeSet::new)
                        .insert(id);
                }
            }

            // Track highest ID for next_id
            if id >= self.next_id {
                self.next_id = id + 1;
            }
        }
    }

    /// Remove an entry and clean up its index references
    fn remove_entry(&mut self, id: u64) {
        if let Some(entry) = self.entries.remove(&id) {
            for keyword in &entry.keywords {
                if let Some(id_set) = self.index.get_mut(keyword) {
                    id_set.remove(&id);
                    if id_set.is_empty() {
                        self.index.remove(keyword);
                    }
                }
            }
        }
    }
}

/// Extract keywords from text for indexing
///
/// - Split on whitespace
/// - Lowercase
/// - Keep words > 3 chars
/// - Skip stop words
fn extract_keywords(text: &str) -> Vec<String> {
    let mut keywords = Vec::new();

    for word in text.split_whitespace() {
        // Strip common punctuation from edges
        let cleaned: String = word.chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect();

        if cleaned.len() < MIN_WORD_LEN {
            continue;
        }

        // Lowercase
        let lower: String = cleaned.chars().map(|c| {
            if c.is_ascii_uppercase() {
                (c as u8 + 32) as char
            } else {
                c
            }
        }).collect();

        // Skip stop words
        let is_stop = STOP_WORDS.iter().any(|sw| *sw == lower.as_str());
        if is_stop {
            continue;
        }

        keywords.push(lower);
    }

    keywords
}

// =========================================================================
// Global Singleton (same pattern as filesystem.rs)
// =========================================================================

lazy_static! {
    /// Global memory store instance
    pub static ref MEMORY: Mutex<MemoryStore> = Mutex::new(MemoryStore::new(200));
}

/// Store a new memory entry (convenience function)
pub fn store(content: &str, kind: MemoryKind, source: &str) -> u64 {
    MEMORY.lock().store(content, kind, source)
}

/// Store with timestamp (convenience function)
pub fn store_with_timestamp(content: &str, kind: MemoryKind, source: &str, timestamp: u64) -> u64 {
    MEMORY.lock().store_with_timestamp(content, kind, source, timestamp)
}

/// Search memory (convenience function)
pub fn search(query: &str) -> Vec<(u64, u32)> {
    MEMORY.lock().search(query)
}

/// Get an entry by ID (returns a clone since we release the lock)
pub fn get(id: u64) -> Option<MemoryEntry> {
    MEMORY.lock().get(id).cloned()
}

/// Get recent entries (returns clones)
pub fn recent(count: usize) -> Vec<MemoryEntry> {
    MEMORY.lock().recent(count).into_iter().cloned().collect()
}

/// Get memory statistics
pub fn stats() -> MemoryStats {
    MEMORY.lock().stats()
}

/// Save memory to filesystem (in-memory) and persist via serial bridge
pub fn save() {
    use crate::storage::filesystem;
    use crate::serial_println;

    let data = MEMORY.lock().serialize();

    // Save to in-memory filesystem (for intra-session use)
    let _ = filesystem::create_dir("/storage/memory");
    match filesystem::write_file_string(MEMORY_FILE_PATH, &data) {
        Ok(()) => {}
        Err(e) => {
            serial_println!("[MEMORY_STORE] FS save failed: {:?}", e);
        }
    }

    // Persist via serial bridge (for cross-session persistence)
    persist_to_serial();
}

/// Persist all memory entries via serial port so the bridge can save to disk.
///
/// Protocol:
///   [MEMORY_PERSIST] <serialized_line>   (one per entry)
///   [MEMORY_DONE]                         (signals end of dump)
///
/// The bridge catches these tags and writes them to ~/.genesis/memory.dat
pub fn persist_to_serial() {
    use crate::serial_println;

    let data = MEMORY.lock().serialize();
    let count = MEMORY.lock().entries.len();

    for line in data.lines() {
        if !line.trim().is_empty() {
            serial_println!("[MEMORY_PERSIST] {}", line);
        }
    }
    serial_println!("[MEMORY_DONE]");
    serial_println!("[MEMORY_STORE] Persisted {} entries via serial bridge", count);
}

/// Load memory from data received via serial bridge
///
/// Called by the shell when it receives [MEMORY_LOAD_DONE] after accumulating
/// all [MEMORY_LOAD] lines from the bridge.
pub fn load_from_serial_data(data: &str) {
    use crate::serial_println;

    if data.trim().is_empty() {
        serial_println!("[MEMORY_STORE] No persisted memories from bridge (fresh start)");
        return;
    }

    MEMORY.lock().deserialize(data);
    let count = MEMORY.lock().entries.len();
    serial_println!("[MEMORY_STORE] Loaded {} entries from serial bridge", count);
}

/// Load memory from filesystem
pub fn load() {
    use crate::storage::filesystem;
    use crate::serial_println;

    match filesystem::read_file_string(MEMORY_FILE_PATH) {
        Ok(data) => {
            MEMORY.lock().deserialize(&data);
            let count = MEMORY.lock().entries.len();
            serial_println!("[MEMORY_STORE] Loaded {} entries from {}", count, MEMORY_FILE_PATH);
        }
        Err(_) => {
            serial_println!("[MEMORY_STORE] No persisted memories found (fresh start)");
        }
    }
}
