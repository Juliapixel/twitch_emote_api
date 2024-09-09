use std::{sync::Arc, time::Duration};

use log::info;
use reqwest::header::ACCEPT;
use serde::Deserialize;

use crate::{cache::Cache, emote::Emote};

use super::{cache::platform_cache_evictor, PlatformError, EMOTE_CACHE_MAX_AGE, USER_CACHE_MAX_AGE};

#[derive(Debug, Clone)]
pub struct BttvClient {
    client: reqwest::Client,
    emote_cache: Arc<Cache<String, Emote>>,
    user_cache: Arc<Cache<String, Arc<UserEmotes>>>,
}

impl BttvClient {
    pub fn new() -> Self {
        let emote_cache = Arc::new(Cache::new(EMOTE_CACHE_MAX_AGE));
        let user_cache = Arc::new(Cache::new(USER_CACHE_MAX_AGE));

        tokio::spawn(platform_cache_evictor(
            Arc::downgrade(&user_cache),
            Duration::from_secs(60 * 15),
            Arc::downgrade(&emote_cache),
            Duration::from_secs(60 * 15)
        ));

        Self {
            client: reqwest::Client::new(),
            emote_cache,
            user_cache,
        }
    }

    pub async fn get_channel_emotes(&self, twitch_id: &str) -> Result<Arc<UserEmotes>, PlatformError> {
        if let Some(hit) = self.user_cache.get(twitch_id) {
            return Ok(hit.clone())
        }

        let emotes: Arc<UserEmotes> = Arc::new(self
            .client
            .get(format!(
                "https://api.betterttv.net/3/cached/users/twitch/{twitch_id}"
            ))
            .send()
            .await?
            .json()
            .await
            .map_err(|e| e.without_url())?);

        self.user_cache.insert(twitch_id.into(), emotes.clone());
        Ok(emotes)
    }

    pub async fn get_emote_by_id(&self, id: &str) -> Result<Emote, PlatformError> {
        if let Some(hit) = self.emote_cache.get(id) {
            info!("cache hit for BTTV emote {id}");
            return Ok(hit.clone());
        }

        info!("requesting BTTV emote {id}");
        let resp = self
            .client
            .get(format!("https://cdn.betterttv.net/emote/{id}/3x"))
            .header(ACCEPT, "image/png, image/webp, image/gif")
            .send()
            .await
            .map_err(|e| e.without_url())?;

        let emote = Emote::try_from_response(resp, id).await?;
        self.emote_cache.insert(id.into(), emote.clone());
        Ok(emote)
    }
}

impl Default for BttvClient {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserEmotes {
    pub shared_emotes: Vec<BttvEmote>,
}

#[derive(Debug, Deserialize)]
pub struct BttvEmote {
    pub id: String,
    pub code: String,
}
