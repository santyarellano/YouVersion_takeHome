use std::collections::HashMap;
use std::sync::RwLock;

use crate::models::BibleVersionSummary;

/// Thread-safe in-memory cache for Verse of the Day metadata, passage text, and Bible versions.
///
/// ### Concurrency Model
/// This cache employs `std::sync::RwLock` over internal `HashMap` collections. Multiple concurrent
/// client requests can read cached verses simultaneously without blocking one another (`.read()` lock).
/// An exclusive write lock (`.write()`) is only acquired momentarily when inserting new data after an
/// upstream cache miss.
///
/// ### Multi-Version Caching
/// - `day_to_passage` stores the universal day-to-passage mapping (e.g., day 195 -> "REV.3.20") which
///   is shared across all Bible versions.
/// - `passage_to_text` stores translation-specific text keyed by `(version_id, passage_id)`,
///   allowing concurrent caching and retrieval across multiple Bible versions.
#[derive(Default)]
pub struct VotdCache {
    /// Maps day ordinal (1–366) to passage ID (e.g., 195 -> "REV.3.20").
    day_to_passage: RwLock<HashMap<u32, String>>,
    /// Maps compound key (version_id, passage_id) to (reference, text).
    passage_to_text: RwLock<HashMap<(u64, String), (String, String)>>,
    /// Cached list of available English Bible translations for GET /versions.
    versions: RwLock<Option<Vec<BibleVersionSummary>>>,
}

impl VotdCache {
    /// Creates a new, empty `VotdCache` instance with initialized read-write locks.
    pub fn new() -> Self {
        Self::default()
    }

    /// Retrieves the cached passage identifier for a given day ordinal (1–366).
    ///
    /// Acquires a shared read lock, allowing concurrent non-blocking reads across worker threads.
    /// Returns `Some(passage_id)` if cached, or `None` on cache miss.
    pub fn get_passage_id(&self, day: u32) -> Option<String> {
        let guard = self.day_to_passage.read().unwrap();
        guard.get(&day).cloned()
    }

    /// Stores the passage identifier for a given day ordinal in the cache.
    ///
    /// Acquires an exclusive write lock to safely insert the entry without data races.
    pub fn set_passage_id(&self, day: u32, passage_id: String) {
        let mut guard = self.day_to_passage.write().unwrap();
        guard.insert(day, passage_id);
    }

    /// Retrieves the human-readable reference and text for a specific Bible version and passage identifier.
    ///
    /// Acquires a shared read lock, allowing concurrent reads across threads.
    /// Returns `Some((reference, text))` if cached, or `None` on cache miss.
    pub fn get_passage_text(&self, version_id: u64, passage_id: &str) -> Option<(String, String)> {
        let guard = self.passage_to_text.read().unwrap();
        guard.get(&(version_id, passage_id.to_string())).cloned()
    }

    /// Stores the human-readable reference and verse text for a specific Bible version and passage ID.
    ///
    /// Acquires an exclusive write lock to safely insert the entry after an upstream fetch.
    pub fn set_passage_text(
        &self,
        version_id: u64,
        passage_id: String,
        reference: String,
        text: String,
    ) {
        let mut guard = self.passage_to_text.write().unwrap();
        guard.insert((version_id, passage_id), (reference, text));
    }

    /// Retrieves the cached list of available Bible versions for `GET /versions`.
    ///
    /// Acquires a shared read lock, allowing concurrent non-blocking reads.
    /// Returns `Some(Vec<BibleVersionSummary>)` if cached, or `None` on cache miss.
    pub fn get_versions(&self) -> Option<Vec<BibleVersionSummary>> {
        let guard = self.versions.read().unwrap();
        guard.clone()
    }

    /// Stores the list of available Bible versions in the cache.
    ///
    /// Acquires an exclusive write lock to update the cached versions list.
    pub fn set_versions(&self, versions: Vec<BibleVersionSummary>) {
        let mut guard = self.versions.write().unwrap();
        *guard = Some(versions);
    }
}
