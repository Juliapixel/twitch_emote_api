use serde::Serialize;

use super::{bttv::BttvEmote, ffz::FfzEmote, seventv::SevenTvEmote, Platform};

#[derive(Debug, Clone, Serialize)]
pub struct ChannelEmote {
    pub platform: Platform,
    pub id: String,
    pub name: String,
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
