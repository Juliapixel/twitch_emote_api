use std::{borrow::Borrow, hash::Hash};

use dashmap::{
    mapref::one::{MappedRef, MappedRefMut},
    DashMap,
};

#[derive(Debug, Clone)]
pub struct Cache<K: Hash + Eq, V: Sized> {
    map: DashMap<K, CachedItem<V>>,
    max_age: std::time::Duration,
}

impl<K: Hash + Eq, V: Sized> Cache<K, V> {
    pub fn new(max_age: std::time::Duration) -> Self {
        Self {
            map: Default::default(),
            max_age,
        }
    }

    pub fn get<Q>(&self, key: &Q) -> Option<MappedRef<'_, K, CachedItem<V>, V>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if let Some(hit) = self.map.get(key) {
            if std::time::Instant::now() > hit.added_timestamp + self.max_age {
                drop(hit);
                self.map.remove(key);
                return None;
            }
            Some(hit.map(|r| &r.data))
        } else {
            None
        }
    }

    pub fn get_mut<Q>(&self, key: &Q) -> Option<MappedRefMut<'_, K, CachedItem<V>, V>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if let Some(hit) = self.map.get_mut(key) {
            if std::time::Instant::now() > hit.added_timestamp + self.max_age {
                return None;
            }
            Some(hit.map(|r| &mut r.data))
        } else {
            None
        }
    }

    pub fn refresh(&self, key: &K) -> Option<()> {
        if let Some(mut val) = self.map.get_mut(key) {
            val.value_mut().refresh();
            Some(())
        } else {
            None
        }
    }

    pub fn evict_stale(&self) {
        let now = std::time::Instant::now();
        self.map
            .retain(|_, v| now < v.added_timestamp + self.max_age)
    }

    /// gulp
    pub fn shrink_to_fit(&self) {
        self.map.shrink_to_fit()
    }

    pub fn insert(&self, key: K, value: V) -> Option<V> {
        self.map.insert(key, CachedItem::new(value)).map(|r| r.data)
    }
}

#[derive(Debug, Clone)]
pub struct CachedItem<V: Sized> {
    added_timestamp: std::time::Instant,
    data: V,
}

impl<V: Sized> CachedItem<V> {
    pub fn new(data: V) -> Self {
        Self {
            added_timestamp: std::time::Instant::now(),
            data,
        }
    }

    pub fn refresh(&mut self) {
        self.added_timestamp = std::time::Instant::now()
    }
}

// impl Deref for EmoteCache {
//     type Target = DashMap<String, Emote>;

//     fn deref(&self) -> &Self::Target {
//         &self.cache
//     }
// }
