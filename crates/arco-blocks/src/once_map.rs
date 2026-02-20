//! Internal once-map utility used by block orchestration.

use std::borrow::Borrow;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::hash::Hash;
use std::sync::Mutex;

/// Minimal register/done cache with "compute once" semantics.
#[derive(Debug, Default)]
pub(crate) struct OnceMap<K, V> {
    items: Mutex<HashMap<K, Option<V>>>,
}

impl<K: Eq + Hash, V> OnceMap<K, V> {
    /// Register a key as in-flight. Returns true only for the first caller.
    pub(crate) fn register(&self, key: K) -> bool {
        let mut items = match self.items.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        match items.entry(key) {
            Entry::Occupied(_) => false,
            Entry::Vacant(entry) => {
                entry.insert(None);
                true
            }
        }
    }

    /// Mark key as complete. Returns true only when transitioning from in-flight.
    pub(crate) fn done(&self, key: K, value: V) -> bool {
        let mut items = match self.items.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        match items.entry(key) {
            Entry::Occupied(mut entry) => {
                if entry.get().is_none() {
                    entry.insert(Some(value));
                    true
                } else {
                    false
                }
            }
            Entry::Vacant(entry) => {
                entry.insert(Some(value));
                false
            }
        }
    }
}

impl<K: Eq + Hash, V: Clone> OnceMap<K, V> {
    /// Get a completed value.
    pub(crate) fn get<Q: ?Sized + Eq + Hash>(&self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
    {
        let items = match self.items.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        items.get(key).and_then(std::option::Option::clone)
    }
}

#[cfg(test)]
mod tests {
    use super::OnceMap;

    #[test]
    fn register_and_get() {
        let map: OnceMap<&str, i32> = OnceMap::default();
        assert!(map.register("a"));
        assert!(!map.register("a"));
        assert_eq!(map.get("a"), None);
        assert!(map.done("a", 1));
        assert_eq!(map.get("a"), Some(1));
    }

    #[test]
    fn done_without_register_stores_value() {
        let map: OnceMap<&str, i32> = OnceMap::default();
        assert!(!map.done("a", 2));
        assert_eq!(map.get("a"), Some(2));
    }
}
