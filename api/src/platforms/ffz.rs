use hashbrown::HashMap;
use reqwest::header::ACCEPT;
use serde::Deserialize;

use crate::{cache::EmoteCache, emote::Emote};

use super::PlatformError;

#[derive(Debug, Clone)]
pub struct FfzClient {
    client: reqwest::Client,
    emote_cache: EmoteCache,
}

impl FfzClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            emote_cache: EmoteCache::new(),
        }
    }

    pub async fn get_channel_emotes(&self, twitch_id: &str) -> Result<RoomEmotes, PlatformError> {
        Ok(self
            .client
            .get(format!(
                "https://api.frankerfacez.com/v1/room/id/{twitch_id}"
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
            .get(format!("https://cdn.frankerfacez.com/emote/{id}/4"))
            .header(ACCEPT, "image/png, image/webp, image/gif")
            .send()
            .await
            .map_err(|e| e.without_url())?;

        Ok(Emote::try_from_response(resp, id).await?)
    }
}

impl Default for FfzClient {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize)]
pub struct RoomEmotes {
    pub sets: HashMap<String, FfzSet>,
}

#[derive(Deserialize)]
pub struct FfzSet {
    pub emoticons: Vec<FfzEmote>,
}

#[derive(Deserialize)]
pub struct FfzEmote {
    pub id: String,
    pub name: String,
}
