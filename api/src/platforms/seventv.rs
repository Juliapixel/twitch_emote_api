use std::{
    iter::Map,
    ops::Deref,
    sync::Arc,
    time::{Duration, Instant},
};

use dashmap::DashMap;
use log::info;
use reqwest::header::ACCEPT;
use serde::Deserialize;
use tokio::sync::OnceCell;

use crate::{cache::Cache, emote::Emote};

use super::{
    cache::platform_cache_evictor, channel::ChannelEmote, EmotePlatform, PlatformError,
    EMOTE_CACHE_MAX_AGE, USER_CACHE_MAX_AGE,
};

#[derive(Debug, Clone)]
pub struct SevenTvClient {
    client: reqwest::Client,
    emote_cache: Arc<Cache<String, Emote>>,
    user_cache: Arc<Cache<String, Arc<UserEmotes>>>,
}

impl SevenTvClient {
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
}

impl EmotePlatform for SevenTvClient {
    type InternalEmoteType = UserEmotes;

    async fn get_emote_by_id(&self, id: &str) -> Result<Emote, PlatformError> {
        if let Some(hit) = self.emote_cache.get(id) {
            info!("cache hit for 7TV emote {id}");

            return Ok(hit.clone());
        }

        info!("requesting 7TV emote {id}");
        let resp = self
            .client
            .get(format!("https://cdn.7tv.app/emote/{id}/4x.webp"))
            .header(
                ACCEPT,
                "image/png, imErr(PlatformError::ChannelNotFound)age/webp, image/gif",
            )
            .send()
            .await
            .map_err(|e| e.without_url())?;

        let emote = Emote::try_from_response(resp, id).await?;
        self.emote_cache.insert(id.into(), emote.clone());
        Ok(emote)
    }

    async fn get_global_emotes(&self) -> Result<Arc<DashMap<String, ChannelEmote>>, PlatformError> {
        static SEVENTV_GLOBALS: OnceCell<(Arc<DashMap<String, ChannelEmote>>, Instant)> =
            OnceCell::const_new();

        let gotten = SEVENTV_GLOBALS
            .get_or_try_init(|| async {
                let resp = self
                    .client
                    .get("https://7tv.io/v3/emote-sets/global")
                    .send()
                    .await?
                    .json::<EmoteSet>()
                    .await?;
                let emotes = resp.emotes.into_iter().map(|e| (e.name.clone(), e.into()));
                Result::<(Arc<DashMap<String, ChannelEmote>>, Instant), PlatformError>::Ok((
                    Arc::new(emotes.collect()),
                    Instant::now(),
                ))
            })
            .await?;

        Ok(gotten.0.clone())
    }

    async fn get_channel_emotes(
        &self,
        twitch_id: &str,
    ) -> Result<impl Deref<Target = Self::InternalEmoteType>, PlatformError>
    where
        for<'a> &'a Self::InternalEmoteType: IntoIterator<Item = ChannelEmote>,
    {
        if let Some(hit) = self.user_cache.get(twitch_id) {
            info!("7TV channel emotes cache hit for {twitch_id}");
            return Ok(hit.clone());
        }

        info!("requesting 7TV channel emotes for {twitch_id}");

        let emotes: Arc<UserEmotes> = Arc::new(
            self.client
                .get(format!("https://7tv.io/v3/users/twitch/{twitch_id}"))
                .send()
                .await?
                .json()
                .await
                .map_err(|e| e.without_url())?,
        );

        self.user_cache.insert(twitch_id.into(), emotes.clone());
        Ok(emotes)
    }
}

impl Default for SevenTvClient {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserEmotes {
    pub emote_set: EmoteSet,
}

impl<'a> IntoIterator for &'a UserEmotes {
    type Item = ChannelEmote;

    type IntoIter = Map<std::slice::Iter<'a, SevenTvEmote>, fn(&SevenTvEmote) -> ChannelEmote>;

    fn into_iter(self) -> Self::IntoIter {
        self.emote_set.emotes.iter().map(|e| e.into())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmoteSet {
    pub emotes: Vec<SevenTvEmote>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SevenTvEmote {
    pub id: String,
    pub name: String,
    pub data: EmoteData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmoteData {
    pub listed: bool,
    pub animated: bool,
}
