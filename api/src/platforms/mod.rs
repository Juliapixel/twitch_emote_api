use std::{fmt::Display, sync::Arc, time::Duration};

use axum::response::IntoResponse;
use channel::{ChannelEmote, ChannelEmotes};
use dashmap::DashMap;
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::emote::{Emote, EmoteError};

pub mod bttv;
pub mod channel;
pub mod ffz;
pub mod seventv;
pub mod twitch;

pub use bttv::BttvClient;
pub use ffz::FfzClient;
pub use seventv::SevenTvClient;
pub use twitch::TwitchClient;

pub const EMOTE_CACHE_MAX_AGE: Duration = Duration::from_secs(60 * 60 * 8);
pub const EMOTE_CACHE_EVICTION_INTERVAL: Duration = Duration::from_secs(60 * 15);
pub const USER_CACHE_MAX_AGE: Duration = Duration::from_secs(60 * 15);
pub const USER_CACHE_EVICTION_INTERVAL: Duration = Duration::from_secs(60 * 15);

#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    #[error("the requested channel wasn't found")]
    ChannelNotFound,
    #[error("the requested emote wasn't found")]
    EmoteNotFound,
    #[error(transparent)]
    RequestFailure(#[from] reqwest::Error),
    #[error("requesting the emote from {0} returned an error")]
    PlatformError(Platform),
    #[error("requesting the emote from {0} was rejected")]
    Unauthorized(Platform),
    #[error(transparent)]
    DecodeError(#[from] EmoteError),
}

impl IntoResponse for PlatformError {
    fn into_response(self) -> axum::response::Response {
        match self {
            PlatformError::ChannelNotFound => (StatusCode::NOT_FOUND, ()).into_response(),
            PlatformError::EmoteNotFound => (StatusCode::NOT_FOUND, ()).into_response(),
            PlatformError::RequestFailure(_) => {
                (StatusCode::BAD_GATEWAY, self.to_string()).into_response()
            }
            PlatformError::PlatformError(_) => {
                (StatusCode::BAD_GATEWAY, self.to_string()).into_response()
            }
            PlatformError::Unauthorized(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
            }
            PlatformError::DecodeError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Twitch,
    #[serde(rename = "7tv")]
    SevenTv,
    #[serde(rename = "bttv")]
    BetterTtv,
    #[serde(rename = "ffz")]
    FrancerFaceZ,
}

impl Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Platform::Twitch => "Twitch",
                Platform::SevenTv => "7TV",
                Platform::BetterTtv => "BetterTTV",
                Platform::FrancerFaceZ => "FrankerFaceZ",
            }
        )
    }
}

#[derive(Debug, Clone)]
pub struct EmoteManager {
    twitch: TwitchClient,
    seventv: SevenTvClient,
    ffz: FfzClient,
    bttv: BttvClient,
    channel_emotes: ChannelEmotes,
}

impl EmoteManager {
    pub async fn new(
        twitch_client_id: impl Into<Arc<str>>,
        twitch_client_secret: impl Into<Arc<str>>,
    ) -> Result<Self, PlatformError> {
        Ok(Self {
            twitch: TwitchClient::new(twitch_client_id.into(), twitch_client_secret.into()).await?,
            seventv: Default::default(),
            ffz: Default::default(),
            bttv: Default::default(),
            channel_emotes: Default::default(),
        })
    }

    pub async fn get_emote(&self, platform: Platform, id: &str) -> Result<Emote, PlatformError> {
        match platform {
            Platform::Twitch => todo!(),
            Platform::SevenTv => self.seventv.get_emote_by_id(id).await,
            Platform::BetterTtv => self.bttv.get_emote_by_id(id).await,
            Platform::FrancerFaceZ => todo!(),
        }
    }

    pub async fn get_channel_emotes(
        &self,
        channel: &str,
    ) -> Result<Arc<DashMap<String, ChannelEmote>>, PlatformError> {
        self.channel_emotes.get_or_track(channel, self).await
    }
}

mod cache {
    use std::{hash::Hash, sync::Weak, time::Duration};

    use futures::FutureExt;
    use tokio::time::MissedTickBehavior;

    use crate::cache::Cache;

    pub async fn platform_cache_evictor<K1: Hash + Eq, V1, K2: Hash + Eq, V2>(
        user_cache: Weak<Cache<K1, V1>>,
        user_cache_interval: Duration,
        emote_cache: Weak<Cache<K2, V2>>,
        emote_cache_interval: Duration,
    ) {
        let mut user_interval = tokio::time::interval(user_cache_interval);
        user_interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
        let mut emote_interval = tokio::time::interval(emote_cache_interval);
        emote_interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            futures::select! {
                _ = user_interval.tick().fuse() => {
                    if let Some(cache) = user_cache.upgrade() {
                        cache.evict_stale()
                    } else {
                        return
                    }
                }
                _ = emote_interval.tick().fuse() => {
                    if let Some(cache) = emote_cache.upgrade() {
                        cache.evict_stale()
                    } else {
                        return
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    //! TuCuando
    #![allow(clippy::unwrap_used, reason = "bruh this is a tests module")]

    use crate::platforms::{bttv::BttvClient, ffz::FfzClient, seventv::SevenTvClient};

    use super::TwitchClient;

    // id for Julialuxel (me)
    const TWITCH_ID: &str = "173685614";

    #[tokio::test]
    async fn seventv_test() {
        // FeelsGoodMan
        const STATIC_EMOTE_ID: &str = "630660200e929d2fde44db5b";
        // RareParrot
        const ANIMATED_EMOTE_ID: &str = "63071ba3449e6f5ff95cca6d";

        let client = SevenTvClient::new();
        client.get_channel_emotes(TWITCH_ID).await.unwrap();

        client.get_emote_by_id(STATIC_EMOTE_ID).await.unwrap();
        client.get_emote_by_id(ANIMATED_EMOTE_ID).await.unwrap();
    }

    #[tokio::test]
    async fn bttv_test() {
        // bttvNice
        const STATIC_EMOTE_ID: &str = "54fab7d2633595ca4c713abf";
        const ANIMATED_EMOTE_ID: &str = "566ca38765dbbdab32ec0560";

        let client = BttvClient::new();
        client.get_channel_emotes(TWITCH_ID).await.unwrap();

        client.get_emote_by_id(STATIC_EMOTE_ID).await.unwrap();
        client.get_emote_by_id(ANIMATED_EMOTE_ID).await.unwrap();
    }

    #[tokio::test]
    async fn ffz_test() {
        // LilZ
        const STATIC_EMOTE_ID: &str = "28136";

        let client = FfzClient::new();
        client.get_channel_emotes(TWITCH_ID).await.unwrap();

        client.get_emote_by_id(STATIC_EMOTE_ID).await.unwrap();
    }

    #[tokio::test]
    async fn twitch_test() {
        let client_id = dotenvy::var("TWITCH_CLIENT_ID").unwrap();
        let client_secret = dotenvy::var("TWITCH_CLIENT_SECRET").unwrap();

        let client = TwitchClient::new(client_id, client_secret).await.unwrap();

        assert_eq!(client.get_channel_id("twitch").await.unwrap(), "12826");
    }
}
