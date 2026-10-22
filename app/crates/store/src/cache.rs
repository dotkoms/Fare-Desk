use std::collections::HashMap;
use std::sync::RwLock;
use std::time::Duration;

use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Cached<V> {
    pub value: V,
    pub stored_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl<V> Cached<V> {
    pub fn is_fresh(&self, now: DateTime<Utc>) -> bool {
        now < self.expires_at
    }

    pub fn age_seconds(&self, now: DateTime<Utc>) -> u64 {
        (now - self.stored_at).num_seconds().max(0) as u64
    }
}

/// A small in-process TTL cache. Expired entries are kept until evicted so a
/// failed source can still serve the last good answer (stale-if-error).
#[derive(Debug)]
pub struct TimedCache<V> {
    entries: RwLock<HashMap<String, Cached<V>>>,
    capacity: usize,
}

impl<V: Clone> TimedCache<V> {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            capacity,
        }
    }

    pub fn insert(&self, key: impl Into<String>, value: V, ttl: Duration) {
        let now = Utc::now();
        let ttl = chrono::Duration::from_std(ttl).unwrap_or_else(|_| chrono::Duration::zero());
        let entry = Cached {
            value,
            stored_at: now,
            expires_at: now + ttl,
        };
        let key = key.into();
        let mut entries = self.entries.write().expect("cache lock");
        if entries.len() >= self.capacity && !entries.contains_key(&key) {
            // Evict the oldest entry; expired entries go first.
            if let Some(oldest) = entries
                .iter()
                .min_by_key(|(_, entry)| entry.stored_at)
                .map(|(key, _)| key.clone())
            {
                entries.remove(&oldest);
            }
        }
        entries.insert(key, entry);
    }

    /// Fresh entry only.
    pub fn get(&self, key: &str) -> Option<Cached<V>> {
        let entries = self.entries.read().expect("cache lock");
        entries
            .get(key)
            .filter(|entry| entry.is_fresh(Utc::now()))
            .cloned()
    }

    /// Any entry, fresh or expired. Used when the source fails.
    pub fn get_stale(&self, key: &str) -> Option<Cached<V>> {
        self.entries.read().expect("cache lock").get(key).cloned()
    }

    pub fn len(&self) -> usize {
        self.entries.read().expect("cache lock").len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Drop expired entries. Called by the API on a timer in the live service.
    pub fn sweep(&self) {
        let now = Utc::now();
        self.entries
            .write()
            .expect("cache lock")
            .retain(|_, entry| entry.is_fresh(now));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn fresh_entries_are_served() {
        let cache: TimedCache<u32> = TimedCache::new(4);
        cache.insert("a", 1, Duration::from_secs(60));
        assert_eq!(cache.get("a").unwrap().value, 1);
    }

    #[test]
    fn expired_entries_are_hidden_but_kept_for_stale_reads() {
        let cache: TimedCache<u32> = TimedCache::new(4);
        cache.insert("a", 1, Duration::from_millis(10));
        sleep(Duration::from_millis(25));
        assert!(cache.get("a").is_none());
        assert_eq!(cache.get_stale("a").unwrap().value, 1);
    }

    #[test]
    fn capacity_evicts_oldest_first() {
        let cache: TimedCache<u32> = TimedCache::new(2);
        cache.insert("a", 1, Duration::from_secs(60));
        sleep(Duration::from_millis(5));
        cache.insert("b", 2, Duration::from_secs(60));
        sleep(Duration::from_millis(5));
        cache.insert("c", 3, Duration::from_secs(60));
        assert_eq!(cache.len(), 2);
        assert!(cache.get("a").is_none());
        assert!(cache.get("b").is_some());
        assert!(cache.get("c").is_some());
    }

    #[test]
    fn sweep_drops_expired_entries() {
        let cache: TimedCache<u32> = TimedCache::new(4);
        cache.insert("a", 1, Duration::from_millis(10));
        cache.insert("b", 2, Duration::from_secs(60));
        sleep(Duration::from_millis(25));
        cache.sweep();
        assert_eq!(cache.len(), 1);
        assert!(cache.get_stale("a").is_none());
    }
}
