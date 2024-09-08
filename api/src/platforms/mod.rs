use std::fmt::Display;

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

#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    #[error("the requested channel wasn't found")]
    ChannelNotFound,
    #[error(transparent)]
    RequestFailure(#[from] reqwest::Error),
    #[error("requesting the emote from the emote platform returned an error: {0}")]
    PlatformError(Platform),
    #[error(transparent)]
    DecodeError(#[from] EmoteError),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename(serialize = "snake_case", deserialize = "snake_case"))]
pub enum Platform {
    Twitch,
    SevenTv,
    BetterTtv,
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
}

impl EmoteManager {
    pub fn new(twitch_token: String) -> Self {
        Self {
            twitch: TwitchClient::new(twitch_token),
            seventv: Default::default(),
            ffz: Default::default(),
            bttv: Default::default(),
        }
    }

    pub async fn get_emote(&self, platform: Platform, id: &str) -> Result<Emote, PlatformError> {
        match platform {
            Platform::Twitch => todo!(),
            Platform::SevenTv => self.seventv.get_emote_by_id(id).await,
            Platform::BetterTtv => self.bttv.get_emote_by_id(id).await,
            Platform::FrancerFaceZ => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    //! TuCuando

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
        let app_access_token = dotenvy::var("TWITCH_APP_ACCESS_TOKEN").unwrap();

        let client = TwitchClient::new(app_access_token);

        assert_eq!(client.get_channel_id("twitch").await.unwrap(), "12826");
    }
}
