use reqwest::header::ACCEPT;
use serde::Deserialize;

use crate::{
    cache::{self, EmoteCache},
    emote::Emote,
};

use super::PlatformError;

#[derive(Debug, Clone)]
pub struct SevenTvClient {
    client: reqwest::Client,
    emote_cache: cache::EmoteCache,
}

impl SevenTvClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            emote_cache: EmoteCache::new(),
        }
    }

    pub async fn get_channel_emotes(&self, twitch_id: &str) -> Result<UserEmotes, PlatformError> {
        Ok(self
            .client
            .get(format!("https://7tv.io/v3/users/twitch/{twitch_id}"))
            .send()
            .await?
            .json()
            .await
            .map_err(|e| e.without_url())?)
    }

    pub async fn get_emote_by_id(&self, id: &str) -> Result<Emote, PlatformError> {
        if let Some(hit) = self.emote_cache.get(id) {
            return Ok(hit.clone());
        }

        let resp = self
            .client
            .get(format!("https://cdn.7tv.app/emote/{id}/4x.webp"))
            .header(ACCEPT, "image/png, image/webp, image/gif")
            .send()
            .await
            .map_err(|e| e.without_url())?;

        let emote = Emote::try_from_response(resp, id).await?;
        self.emote_cache.insert(id, emote.clone());
        Ok(emote)
    }
}

impl Default for SevenTvClient {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize)]
pub struct UserEmotes {
    pub emote_set: EmoteSet,
}

#[derive(Deserialize)]
pub struct EmoteSet {
    pub emotes: Vec<SevenTvEmote>,
}

#[derive(Deserialize)]
pub struct SevenTvEmote {
    pub id: String,
    pub name: String,
    pub data: EmoteData,
}

#[derive(Deserialize)]
pub struct EmoteData {
    pub listed: bool,
    pub animated: bool,
}
