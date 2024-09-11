use std::sync::Arc;

use dashmap::DashMap;
use log::error;
use serde::Serialize;

use crate::platforms::EmotePlatform;

use super::{
    bttv::BttvEmote, ffz::FfzEmote, seventv::SevenTvEmote, EmoteManager, Platform, PlatformError,
};

#[derive(Debug, Clone, Serialize)]
pub struct ChannelEmote {
    pub platform: Platform,
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Default)]
pub struct ChannelEmotes {
    channels: DashMap<String, Arc<DashMap<String, ChannelEmote>>>,
}

impl ChannelEmotes {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, channel: &str) -> Option<Arc<DashMap<String, ChannelEmote>>> {
        self.channels.get(channel).map(|i| i.clone())
    }

    pub async fn get_or_track(
        &self,
        channel: &str,
        manager: &EmoteManager,
    ) -> Result<Arc<DashMap<String, ChannelEmote>>, PlatformError> {
        match self.channels.get(channel).map(|i| i.clone()) {
            Some(emotes) => Ok(emotes),
            None => {
                let mut emotes = DashMap::<String, ChannelEmote>::new();

                let user_id = manager.twitch.get_channel_id(channel).await?;

                let (seventv_resp, bttv_resp, ffz_resp) = futures::join!(
                    manager.seventv.get_channel_emotes(&user_id),
                    manager.bttv.get_channel_emotes(&user_id),
                    manager.ffz.get_channel_emotes(&user_id)
                );

                match seventv_resp {
                    Ok(resp) => {
                        emotes.extend(
                            resp.into_iter().map(|e| (e.name.clone(), e)),
                        )
                    },
                    Err(e) => error!("{e}"),
                }
                match bttv_resp {
                    Ok(resp) => {
                        emotes.extend(
                            resp.into_iter().map(|e| (e.name.clone(), e)),
                        )
                    },
                    Err(e) => error!("{e}"),
                }
                match ffz_resp {
                    Ok(resp) => {
                        emotes.extend(
                            resp.into_iter().map(|e| (e.name.clone(), e)),
                        )
                    },
                    Err(e) => error!("{e}"),
                }

                let emotes: Arc<DashMap<String, ChannelEmote>> = emotes.into();
                self.channels.insert(channel.into(), emotes.clone());
                Ok(emotes)
            }
        }
    }
}

impl From<SevenTvEmote> for ChannelEmote {
    fn from(value: SevenTvEmote) -> Self {
        Self {
            platform: Platform::SevenTv,
            id: value.id,
            name: value.name,
        }
    }
}

impl From<&SevenTvEmote> for ChannelEmote {
    fn from(value: &SevenTvEmote) -> Self {
        Self {
            platform: Platform::SevenTv,
            id: value.id.clone(),
            name: value.name.clone(),
        }
    }
}

impl From<BttvEmote> for ChannelEmote {
    fn from(value: BttvEmote) -> Self {
        Self {
            platform: Platform::BetterTtv,
            id: value.id,
            name: value.code,
        }
    }
}

impl From<&BttvEmote> for ChannelEmote {
    fn from(value: &BttvEmote) -> Self {
        Self {
            platform: Platform::BetterTtv,
            id: value.id.clone(),
            name: value.code.clone(),
        }
    }
}

impl From<FfzEmote> for ChannelEmote {
    fn from(value: FfzEmote) -> Self {
        Self {
            platform: Platform::FrancerFaceZ,
            id: value.id.map_left(|id| id.to_string()).into_inner(),
            name: value.name,
        }
    }
}

impl From<&FfzEmote> for ChannelEmote {
    fn from(value: &FfzEmote) -> Self {
        Self {
            platform: Platform::FrancerFaceZ,
            id: value.id.clone().map_left(|id| id.to_string()).into_inner(),
            name: value.name.clone(),
        }
    }
}
