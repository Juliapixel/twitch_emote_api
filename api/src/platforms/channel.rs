use serde::Serialize;

use super::{bttv::BttvEmote, ffz::FfzEmote, seventv::SevenTvEmote, Platform};

#[derive(Debug, Clone, Serialize)]
pub struct ChannelEmote {
    pub platform: Platform,
    pub id: String,
    pub name: String,
    pub animated: bool,
}

impl From<SevenTvEmote> for ChannelEmote {
    fn from(value: SevenTvEmote) -> Self {
        Self {
            platform: Platform::SevenTv,
            id: value.id,
            name: value.name,
            animated: value.data.animated,
        }
    }
}

impl From<&SevenTvEmote> for ChannelEmote {
    fn from(value: &SevenTvEmote) -> Self {
        Self {
            platform: Platform::SevenTv,
            id: value.id.clone(),
            name: value.name.clone(),
            animated: value.data.animated,
        }
    }
}

impl From<BttvEmote> for ChannelEmote {
    fn from(value: BttvEmote) -> Self {
        Self {
            platform: Platform::BetterTtv,
            id: value.id,
            name: value.code,
            animated: value.animated,
        }
    }
}

impl From<&BttvEmote> for ChannelEmote {
    fn from(value: &BttvEmote) -> Self {
        Self {
            platform: Platform::BetterTtv,
            id: value.id.clone(),
            name: value.code.clone(),
            animated: value.animated,
        }
    }
}

impl From<FfzEmote> for ChannelEmote {
    fn from(value: FfzEmote) -> Self {
        Self {
            platform: Platform::FrancerFaceZ,
            id: value.id.map_left(|id| id.to_string()).into_inner(),
            name: value.name,
            animated: value.animated.is_some(),
        }
    }
}

impl From<&FfzEmote> for ChannelEmote {
    fn from(value: &FfzEmote) -> Self {
        Self {
            platform: Platform::FrancerFaceZ,
            id: value.id.clone().map_left(|id| id.to_string()).into_inner(),
            name: value.name.clone(),
            animated: value.animated.is_some(),
        }
    }
}
