use std::{
    sync::{Arc, LazyLock},
    time::Duration,
};

use http::{HeaderName, HeaderValue};
use log::info;
use reqwest::StatusCode;
use serde::Deserialize;

use crate::{cache::Cache, platforms::Platform};

use super::PlatformError;

const ID_CACHE_MAX_AGE: Duration = Duration::from_secs(60 * 60 * 8);

#[derive(Clone)]
pub struct TwitchClient {
    client: reqwest::Client,
    client_id: Arc<str>,
    client_secret: Arc<str>,
    token: Arc<str>,
    user_id_cache: Arc<Cache<String, String>>,
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

        let cache = Arc::new(Cache::new(ID_CACHE_MAX_AGE));

        // task that clears out the cache every once in a while
        tokio::spawn({
            let cache = Arc::downgrade(&cache);
            async move {
                let mut interval = tokio::time::interval(Duration::from_secs(60 * 10));
                loop {
                    interval.tick().await;
                    if let Some(cache) = cache.upgrade() {
                        cache.evict_stale()
                    } else {
                        return;
                    }
                }
            }
        });

        Ok(Self {
            client,
            client_id,
            client_secret,
            token: token.into(),
            user_id_cache: cache,
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
#[derive(Debug, Deserialize)]
pub struct HelixResponse<T> {
    data: T,
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
