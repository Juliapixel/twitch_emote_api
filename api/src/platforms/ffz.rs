use std::{sync::Arc, time::{Duration, Instant}};

use hashbrown::HashMap;
use reqwest::header::ACCEPT;
use serde::Deserialize;
use tokio::sync::OnceCell;

use crate::{cache::Cache, emote::Emote, platforms::channel::ChannelEmote};

use super::{
    cache::platform_cache_evictor, EmotePlatform, PlatformError, EMOTE_CACHE_MAX_AGE, USER_CACHE_MAX_AGE
};

#[derive(Debug, Clone)]
pub struct FfzClient {
    client: reqwest::Client,
    emote_cache: Arc<Cache<String, Emote>>,
    user_cache: Arc<Cache<String, Arc<RoomEmotes>>>,
}

impl FfzClient {
    pub fn new() -> Self {
        let emote_cache = Arc::new(Cache::new(EMOTE_CACHE_MAX_AGE));
        let user_cache = Arc::new(Cache::new(USER_CACHE_MAX_AGE));

        tokio::spawn(platform_cache_evictor(
            Arc::downgrade(&user_cache),
            Duration::from_secs(60 * 15),
            Arc::downgrade(&emote_cache),
            Duration::from_secs(60 * 15),
        ));

        Self {
            client: reqwest::Client::new(),
            emote_cache,
            user_cache,
        }
    }

    pub async fn get_channel_emotes(
        &self,
        twitch_id: &str,
    ) -> Result<Arc<RoomEmotes>, PlatformError> {
        if let Some(hit) = self.user_cache.get(twitch_id) {
            return Ok(hit.clone());
        }

        let emotes: Arc<RoomEmotes> = Arc::new(
            self.client
                .get(format!(
                    "https://api.frankerfacez.com/v1/room/id/{twitch_id}"
                ))
                .send()
                .await?
                .json()
                .await
                .map_err(|e| e.without_url())?,
        );

        self.user_cache.insert(twitch_id.into(), emotes.clone());
        Ok(emotes)
    }

    pub async fn get_emote_by_id(&self, id: &str) -> Result<Emote, PlatformError> {
        if let Some(hit) = self.emote_cache.get(id) {
            return Ok(hit.clone());
        }

        let resp = self
            .client
            .get(format!("https://cdn.frankerfacez.com/emote/{id}/4"))
            .header(ACCEPT, "image/png, image/webp, image/gif")
            .send()
            .await
            .map_err(|e| e.without_url())?;

        Ok(Emote::try_from_response(resp, id).await?)
    }
}

impl EmotePlatform for FfzClient {
    type InternalEmoteType = RoomEmotes;

    async fn get_channel_emotes(&self, twitch_id: &str) -> Result<impl std::ops::Deref<Target = Self::InternalEmoteType>, PlatformError>
    where
        for<'a> &'a Self::InternalEmoteType: IntoIterator<Item = super::channel::ChannelEmote> {
        if let Some(hit) = self.user_cache.get(twitch_id) {
            return Ok(hit.clone());
        }

        let emotes: Arc<RoomEmotes> = Arc::new(
            self.client
                .get(format!(
                    "https://api.frankerfacez.com/v1/room/id/{twitch_id}"
                ))
                .send()
                .await?
                .json()
                .await
                .map_err(|e| e.without_url())?,
        );

        self.user_cache.insert(twitch_id.into(), emotes.clone());
        Ok(emotes)
    }

    async fn get_emote_by_id(&self, id: &str) -> Result<Emote, PlatformError> {
        if let Some(hit) = self.emote_cache.get(id) {
            return Ok(hit.clone());
        }

        let resp = self
            .client
            .get(format!("https://cdn.frankerfacez.com/emote/{id}/4"))
            .header(ACCEPT, "image/png, image/webp, image/gif")
            .send()
            .await
            .map_err(|e| e.without_url())?;

        Ok(Emote::try_from_response(resp, id).await?)
    }

    async fn get_global_emotes(&self) -> Result<impl IntoIterator<Item = super::channel::ChannelEmote>, PlatformError> {
        static FFZ_GLOBALS: OnceCell<(Vec<ChannelEmote>, Instant)> = OnceCell::const_new();

        let gotten = FFZ_GLOBALS.get_or_try_init(|| { async {
            let resp = self
                .client
                .get("https://api.frankerfacez.com/v1/set/global/ids")
                .send()
                .await?
                .json::<DefaultSets>().await?;

            // HOLY MOLY
            let emotes: Vec<ChannelEmote> = resp
                .sets
                .into_values()
                .filter(|set| resp.default_sets.contains(&set.id))
                .flat_map(|s| s.emoticons)
                .map(|e| e.into())
                .collect();
            Result::<(Vec<ChannelEmote>, Instant), PlatformError>::Ok((emotes, Instant::now()))
        } }).await?;

        Ok(gotten.0.clone())
    }
}

impl Default for FfzClient {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
pub struct DefaultSets {
    default_sets: Vec<u64>,
    sets: HashMap<String, FfzSet>
}

#[derive(Debug, Deserialize)]
pub struct RoomEmotes {
    pub sets: HashMap<String, FfzSet>,
}

#[derive(Debug, Deserialize)]
pub struct FfzSet {
    pub id: u64,
    pub emoticons: Vec<FfzEmote>,
}

#[derive(Debug, Deserialize)]
pub struct FfzEmote {
    pub id: u64,
    pub name: String,
}
