use std::{
    sync::{Arc, LazyLock},
    time::Duration,
};

use dashmap::DashMap;
use http::{
    header::ACCEPT,
    HeaderName, HeaderValue,
};
use log::info;
use reqwest::StatusCode;
use serde::Deserialize;
use tinyvec::TinyVec;

use crate::{
    cache::Cache,
    emote::Emote,
    platforms::{cache::platform_cache_evictor, Platform, EMOTE_CACHE_MAX_AGE},
};

use super::{channel::ChannelEmote, EmotePlatform, PlatformError};

const ID_CACHE_MAX_AGE: Duration = Duration::from_secs(60 * 60 * 8);

#[derive(Clone)]
pub struct TwitchClient {
    client: reqwest::Client,
    client_id: Arc<str>,
    client_secret: Arc<str>,
    token: Arc<str>,
    user_id_cache: Arc<Cache<String, String>>,
    emote_cache: Arc<Cache<String, Emote>>,
}

impl std::fmt::Debug for TwitchClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TwitchClient")
            .field("client", &self.client)
            .field("client_id", &self.client_id)
            .field("client_secret", &"[debugger no debugging!]")
            .field("token", &"[debugger no debugging!]")
            .finish()
    }
}

impl TwitchClient {
    pub async fn new(
        client_id: impl Into<Arc<str>>,
        client_secret: impl Into<Arc<str>>,
    ) -> Result<Self, PlatformError> {
        #[allow(clippy::unwrap_used, reason = "if this breaks i'll kms")]
        static OAUTH_URL: LazyLock<url::Url> =
            LazyLock::new(|| url::Url::parse("https://id.twitch.tv/oauth2/token").unwrap());

        let client_id = client_id.into();
        let client_secret = client_secret.into();

        let client = reqwest::ClientBuilder::new()
            .gzip(true)
            .brotli(true)
            .default_headers(http::HeaderMap::from_iter([(
                HeaderName::from_static("client-id"),
                HeaderValue::from_str(&client_id).expect("client id must be an ASCII string"),
            )]))
            .build()?;

        let params = [
            ("client_id", &*client_id),
            ("client_secret", &*client_secret),
            ("grant_type", "client_credentials"),
        ];

        let resp = client.post(OAUTH_URL.clone()).form(&params).send().await?;

        let token = match resp.status() {
            x if x.is_success() => resp.json::<OauthResponse>().await?.access_token,
            StatusCode::UNAUTHORIZED => return Err(PlatformError::Unauthorized(Platform::Twitch)),
            _ => {
                return Err(PlatformError::RequestFailure(
                    resp.error_for_status()
                        .map_err(|e| e.without_url())
                        .expect_err("must never be Ok"),
                ))
            }
        };

        let user_cache = Arc::new(Cache::new(ID_CACHE_MAX_AGE));
        let emote_cache = Arc::new(Cache::new(EMOTE_CACHE_MAX_AGE));

        // task that clears out the cache every once in a while
        tokio::spawn(platform_cache_evictor(
            Arc::downgrade(&user_cache),
            Duration::from_secs(60 * 15),
            Arc::downgrade(&emote_cache),
            Duration::from_secs(60 * 15),
        ));

        Ok(Self {
            client,
            client_id,
            client_secret,
            token: token.into(),
            user_id_cache: user_cache,
            emote_cache,
        })
    }

    pub async fn get_channel_id<'a>(&'a self, channel: &str) -> Result<String, PlatformError> {
        #[allow(clippy::unwrap_used, reason = "if this breaks i'll kms")]
        static USERS_ENDPOINT: LazyLock<url::Url> =
            LazyLock::new(|| url::Url::parse("https://api.twitch.tv/helix/users").unwrap());

        if let Some(hit) = self.user_id_cache.get(channel) {
            info!("twitch id cache hit for {channel}");
            return Ok(hit.clone());
        }

        info!("requesting user id for {channel}");
        let mut url = USERS_ENDPOINT.clone();
        url.query_pairs_mut().append_pair("login", channel).finish();

        let resp = self.client.get(url).bearer_auth(&self.token).send().await?;

        match resp.status() {
            StatusCode::OK => {
                let id = resp
                    .json::<HelixResponse<Vec<UserResponse>>>()
                    .await?
                    .data
                    .into_iter()
                    .next()
                    .ok_or(PlatformError::ChannelNotFound)?
                    .id;
                self.user_id_cache.insert(channel.into(), id.clone());
                Ok(id)
            }
            StatusCode::UNAUTHORIZED => Err(PlatformError::Unauthorized(Platform::Twitch)),
            StatusCode::NOT_FOUND => Err(PlatformError::ChannelNotFound),
            _ => Err(PlatformError::PlatformError(Platform::Twitch)),
        }
    }
}

impl EmotePlatform for TwitchClient {
    type InternalEmoteType = Vec<TwitchEmote>;

    /// this is useless p much, emotes will only ever be requested by id
    async fn get_channel_emotes(
        &self,
        _twitch_id: &str,
    ) -> Result<impl std::ops::Deref<Target = Self::InternalEmoteType>, PlatformError>
    where
        for<'a> &'a Self::InternalEmoteType: IntoIterator<Item = super::channel::ChannelEmote>,
    {
        Err::<Arc<Self::InternalEmoteType>, PlatformError>(PlatformError::TwitchChannelEmotes)
    }

    async fn get_emote_by_id(&self, id: &str) -> Result<crate::emote::Emote, PlatformError> {
        if let Some(hit) = self.emote_cache.get(id) {
            return Ok(hit.clone());
        }

        let url = format!(
            "https://static-cdn.jtvnw.net/emoticons/v2/{}/default/dark/3.0",
            id
        );

        let resp = self
            .client
            .get(&url)
            .header(ACCEPT, "image/png, image/webp, image/gif")
            .send()
            .await
            .map_err(|e| e.without_url())?;

        let emote = Emote::try_from_response(resp, id).await?;

        self.emote_cache.insert(id.into(), emote.clone());

        Ok(emote)
    }

    async fn get_global_emotes(
        &self,
    ) -> Result<Arc<dashmap::DashMap<String, ChannelEmote>>, PlatformError> {
        let globals = self
            .client
            .get("https://api.twitch.tv/helix/chat/emotes/global")
            .bearer_auth(&self.token)
            .send()
            .await?
            .json::<HelixResponse<Vec<TwitchEmote>>>()
            .await?
            .data;

        let mut map = DashMap::new();
        map.extend(globals.into_iter().map(|e| (e.name.clone(), e.into())));

        Ok(Arc::new(map))
    }
}

#[derive(Debug, Deserialize)]
pub struct HelixResponse<T> {
    data: T,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TwitchEmote {
    pub id: String,
    pub name: String,
    pub format: TinyVec<[TwitchEmoteFormat; 2]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TwitchEmoteFormat {
    Animated,
    #[default]
    Static,
}

#[derive(Debug, Deserialize)]
pub struct UserResponse {
    id: String,
    login: String,
    display_name: String,
}

#[derive(Debug, Deserialize)]
pub struct OauthResponse {
    access_token: String,
}
