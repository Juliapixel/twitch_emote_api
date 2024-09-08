use reqwest::header::ACCEPT;
use serde::Deserialize;

use crate::{cache::EmoteCache, emote::Emote};

use super::PlatformError;

#[derive(Debug, Clone)]
pub struct BttvClient {
    client: reqwest::Client,
    emote_cache: EmoteCache,
}

impl BttvClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            emote_cache: EmoteCache::new(),
        }
    }

    pub async fn get_channel_emotes(&self, twitch_id: &str) -> Result<UserEmotes, PlatformError> {
        Ok(self
            .client
            .get(format!(
                "https://api.betterttv.net/3/cached/users/twitch/{twitch_id}"
            ))
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
            .get(format!("https://cdn.betterttv.net/emote/{id}/3x"))
            .header(ACCEPT, "image/png, image/webp, image/gif")
            .send()
            .await
            .map_err(|e| e.without_url())?;

        Ok(Emote::try_from_response(resp, id).await?)
    }
}

impl Default for BttvClient {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserEmotes {
    pub shared_emotes: Vec<BttvEmote>,
}

#[derive(Deserialize)]
pub struct BttvEmote {
    pub id: String,
    pub code: String,
}
