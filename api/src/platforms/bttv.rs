use std::{
    iter::{Chain, Map},
    sync::Arc,
    time::{Duration, Instant},
};

use dashmap::DashMap;
use log::info;
use reqwest::header::ACCEPT;
use serde::Deserialize;
use tokio::sync::OnceCell;

use crate::{cache::Cache, emote::Emote, platforms::channel::ChannelEmote};

use super::{
    cache::platform_cache_evictor, EmotePlatform, PlatformError, EMOTE_CACHE_MAX_AGE,
    USER_CACHE_MAX_AGE,
};

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
            Duration::from_secs(60 * 15),
        ));

        Self {
            client: reqwest::Client::new(),
            emote_cache,
            user_cache,
        }
    }
}

impl EmotePlatform for BttvClient {
    type InternalEmoteType = UserEmotes;

    async fn get_channel_emotes(
        &self,
        twitch_id: &str,
    ) -> Result<impl std::ops::Deref<Target = Self::InternalEmoteType>, PlatformError>
    where
        for<'a> &'a Self::InternalEmoteType: IntoIterator<Item = super::channel::ChannelEmote>,
    {
        if let Some(hit) = self.user_cache.get(twitch_id) {
            return Ok(hit.clone());
        }

        let emotes: Arc<UserEmotes> = Arc::new(
            self.client
                .get(format!(
                    "https://api.betterttv.net/3/cached/users/twitch/{twitch_id}"
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

    async fn get_global_emotes(&self) -> Result<Arc<DashMap<String, ChannelEmote>>, PlatformError> {
        static BTTV_GLOBALS: OnceCell<(Arc<DashMap<String, ChannelEmote>>, Instant)> =
            OnceCell::const_new();

        let gotten = BTTV_GLOBALS
            .get_or_try_init(|| async {
                let resp = self
                    .client
                    .get("https://api.betterttv.net/3/cached/emotes/global")
                    .send()
                    .await?
                    .json::<Vec<BttvEmote>>()
                    .await?;
                let emotes = resp
                    .into_iter()
                    .map(|e| (e.code.clone(), e.into()))
                    .collect();
                Result::<(Arc<DashMap<String, ChannelEmote>>, Instant), PlatformError>::Ok((
                    Arc::new(emotes),
                    Instant::now(),
                ))
            })
            .await?;

        Ok(gotten.0.clone())
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
    pub channel_emotes: Vec<BttvEmote>,
}

impl<'a> IntoIterator for &'a UserEmotes {
    type Item = ChannelEmote;

    type IntoIter = Map<
        Chain<std::slice::Iter<'a, BttvEmote>, std::slice::Iter<'a, BttvEmote>>,
        fn(&BttvEmote) -> ChannelEmote,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.shared_emotes
            .iter()
            .chain(self.channel_emotes.iter())
            .map(|e| e.into())
    }
}

#[derive(Debug, Deserialize)]
pub struct BttvEmote {
    pub id: String,
    pub code: String,
    pub animated: bool,
}
