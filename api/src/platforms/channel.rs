use std::sync::Arc;

use dashmap::DashMap;
use serde::Serialize;

use super::{
    bttv::BttvEmote, ffz::FfzEmote, seventv::SevenTvEmote, EmoteManager, Platform, PlatformError,
};

#[derive(Debug, Clone, Serialize)]
pub struct ChannelEmote {
    platform: Platform,
    id: String,
    name: String,
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

impl From<BttvEmote> for ChannelEmote {
    fn from(value: BttvEmote) -> Self {
        Self {
            platform: Platform::BetterTtv,
            id: value.id,
            name: value.code,
        }
    }
}

impl From<FfzEmote> for ChannelEmote {
    fn from(value: FfzEmote) -> Self {
        Self {
            platform: Platform::FrancerFaceZ,
            id: value.id,
            name: value.name,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ChannelEmotes {
    channels: DashMap<String, Arc<[ChannelEmote]>>,
}

impl ChannelEmotes {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, channel: &str) -> Option<Arc<[ChannelEmote]>> {
        self.channels.get(channel).map(|i| i.clone())
    }

    pub async fn get_or_track(
        &self,
        channel: &str,
        manager: &EmoteManager,
    ) -> Result<Arc<[ChannelEmote]>, PlatformError> {
        match self.channels.get(channel).map(|i| i.clone()) {
            Some(emotes) => Ok(emotes),
            None => {
                let mut emotes: Vec<ChannelEmote> = Vec::new();
                let user_id = manager.twitch.get_channel_id(channel).await?;
                let (seventv_resp, bttv_resp, ffz_resp) = futures::join!(
                    manager.seventv.get_channel_emotes(&user_id),
                    manager.bttv.get_channel_emotes(&user_id),
                    manager.ffz.get_channel_emotes(&user_id)
                );
                if let Ok(resp) = seventv_resp {
                    emotes.extend(resp.emote_set.emotes.into_iter().map(Into::into))
                }
                if let Ok(resp) = bttv_resp {
                    emotes.extend(resp.shared_emotes.into_iter().map(Into::into))
                }
                if let Ok(resp) = ffz_resp {
                    emotes.extend(
                        resp.sets
                            .into_iter()
                            .flat_map(|(_, set)| set.emoticons)
                            .map(Into::into),
                    )
                }
                let emotes: Arc<[ChannelEmote]> = emotes.into();
                self.channels.insert(channel.into(), emotes.clone());
                Ok(emotes)
            }
        }
    }
}
