use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use dashmap::DashMap;
use hashbrown::HashMap;
use http::StatusCode;
use reqwest::header::ACCEPT;
use serde::{de::IgnoredAny, Deserialize};
use tokio::sync::OnceCell;

use crate::{cache::Cache, emote::Emote, platforms::channel::ChannelEmote};

use super::{
    cache::platform_cache_evictor, EmotePlatform, PlatformError, EMOTE_CACHE_MAX_AGE,
    USER_CACHE_MAX_AGE,
};

#[derive(Debug, Clone)]
pub struct FfzClient {
    client: reqwest::Client,
    emote_cache: Arc<Cache<String, Emote>>,
    user_cache: Arc<Cache<String, Arc<RoomEmotes>>>,
}

impl FfzClient {
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

impl EmotePlatform for FfzClient {
    type InternalEmoteType = RoomEmotes;

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

        let emotes = Arc::new(
            self.client
                .get(format!(
                    "https://api.frankerfacez.com/v1/room/id/{twitch_id}"
                ))
                .send()
                .await?
                .json::<RoomEmotes>()
                .await?,
        );

        self.user_cache.insert(twitch_id.into(), emotes.clone());
        Ok(emotes)
    }

    async fn get_emote_by_id(&self, id: &str) -> Result<Emote, PlatformError> {
        if let Some(hit) = self.emote_cache.get(id) {
            return Ok(hit.clone());
        }

        // TODO: get rid of this query, we already store whether an emote is
        // animated or not
        let emote_query = self
            .client
            .get(format!("https://api.frankerfacez.com/v1/emote/{id}"))
            .send()
            .await
            .map_err(|e| e.without_url())?;

        if emote_query.status() == StatusCode::NOT_FOUND {
            return Err(PlatformError::EmoteNotFound);
        }

        let url = if emote_query
            .json::<FfzEmoteQuery>()
            .await?
            .emote
            .animated
            .is_some()
        {
            format!("https://cdn.frankerfacez.com/emote/{id}/animated/4")
        } else {
            format!("https://cdn.frankerfacez.com/emote/{id}/4")
        };

        let resp = self
            .client
            .get(url)
            .header(ACCEPT, "image/webp, image/png, image/gif")
            .send()
            .await
            .map_err(|e| e.without_url())?;

        Ok(Emote::try_from_response(resp, id).await?)
    }

    async fn get_global_emotes(&self) -> Result<Arc<DashMap<String, ChannelEmote>>, PlatformError> {
        static FFZ_GLOBALS: OnceCell<(Arc<DashMap<String, ChannelEmote>>, Instant)> =
            OnceCell::const_new();

        let gotten = FFZ_GLOBALS
            .get_or_try_init(|| {
                async {
                    let resp = self
                        .client
                        .get("https://api.frankerfacez.com/v1/set/global/ids")
                        .send()
                        .await?
                        .json::<DefaultSets>()
                        .await?;

                    // HOLY MOLY
                    let emotes: DashMap<String, ChannelEmote> = resp
                        .sets
                        .into_values()
                        .filter(|set| resp.default_sets.contains(&set.id))
                        .flat_map(|s| s.emoticons)
                        .map(|e| (e.name.clone(), e.into()))
                        .collect();
                    Result::<(Arc<DashMap<String, ChannelEmote>>, Instant), PlatformError>::Ok((
                        Arc::new(emotes),
                        Instant::now(),
                    ))
                }
            })
            .await?;

        Ok(gotten.0.clone())
    }
}

impl Default for FfzClient {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
pub struct DefaultSets {
    default_sets: Vec<u64>,
    sets: HashMap<String, FfzSet>,
}

#[test]
fn ffz_test_deser() {
    const TEST: &str = "{\"room\":{\"_id\":1459859,\"twitch_id\":173685614,\"youtube_id\":null,\"id\":\"julialuxel\",\"is_group\":false,\"display_name\":\"Julialuxel\",\"set\":1459881,\"moderator_badge\":null,\"vip_badge\":null,\"mod_urls\":null,\"user_badges\":{},\"user_badge_ids\":{},\"css\":null},\"sets\":{\"1459881\":{\"id\":1459881,\"_type\":1,\"icon\":null,\"title\":\"Channel: Julialuxel\",\"css\":null,\"emoticons\":[{\"id\":725695,\"name\":\"Joel\",\"height\":32,\"width\":96,\"public\":true,\"hidden\":false,\"modifier\":false,\"modifier_flags\":0,\"offset\":null,\"margins\":null,\"css\":null,\"owner\":{\"_id\":1363844,\"name\":\"rodskaden\",\"display_name\":\"RodsKaden\"},\"artist\":null,\"urls\":{\"1\":\"https://cdn.frankerfacez.com/emote/725695/1\",\"2\":\"https://cdn.frankerfacez.com/emote/725695/2\",\"4\":\"https://cdn.frankerfacez.com/emote/725695/4\"},\"animated\":{\"1\":\"https://cdn.frankerfacez.com/emote/725695/animated/1\",\"2\":\"https://cdn.frankerfacez.com/emote/725695/animated/2\",\"4\":\"https://cdn.frankerfacez.com/emote/725695/animated/4\"},\"status\":1,\"usage_count\":76,\"created_at\":\"2023-04-15T14:00:09.398Z\",\"last_updated\":\"2023-04-15T14:35:57.027Z\"}]}}}";

    match serde_json::from_str::<RoomEmotes>(TEST) {
        Ok(_) => (),
        Err(e) => {
            eprintln!(
                "{}",
                &TEST[(e.column().saturating_sub(50))..(e.column() + 50).min(TEST.len())]
            );
            panic!("{e}")
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RoomEmotes {
    pub sets: HashMap<String, FfzSet>,
}

impl<'a> IntoIterator for &'a RoomEmotes {
    type Item = ChannelEmote;

    type IntoIter = impl Iterator<Item = ChannelEmote>;

    fn into_iter(self) -> Self::IntoIter {
        self.sets
            .iter()
            .flat_map(|s| &s.1.emoticons)
            .map(|e| e.into())
    }
}

#[derive(Debug, Deserialize)]
pub struct FfzSet {
    pub id: u64,
    pub emoticons: Vec<FfzEmote>,
}

#[derive(Debug, Deserialize)]
pub struct FfzEmote {
    #[serde(with = "either::serde_untagged")]
    pub id: either::Either<u64, String>,
    pub name: String,
    pub animated: Option<IgnoredAny>,
}

#[derive(Debug, Deserialize)]
struct FfzEmoteQuery {
    emote: FfzEmote,
}
