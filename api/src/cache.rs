use std::sync::Arc;

use dashmap::DashMap;

use crate::emote::Emote;

#[derive(Debug, Clone)]
pub struct EmoteCache {
    cache: Arc<DashMap<String, Emote>>,
}

impl EmoteCache {
    pub fn new() -> Self {
        Self {
            cache: Default::default(),
        }
    }

    pub fn get(&self, id: &str) -> Option<Emote> {
        self.cache.get(id).map(|i| i.clone())
    }

    pub fn insert(&self, id: impl Into<String>, emote: Emote) {
        self.cache.insert(id.into(), emote);
    }
}

impl Default for EmoteCache {
    fn default() -> Self {
        Self::new()
    }
}

// impl Deref for EmoteCache {
//     type Target = DashMap<String, Emote>;

//     fn deref(&self) -> &Self::Target {
//         &self.cache
//     }
// }
