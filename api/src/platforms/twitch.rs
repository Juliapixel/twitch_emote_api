use std::sync::LazyLock;

use super::PlatformError;

#[derive(Clone)]
pub struct TwitchClient {
    client: reqwest::Client,
    token: String,
}

impl std::fmt::Debug for TwitchClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TwitchClient")
            .field("token", &"[debugger no debugging!]")
            .finish()
    }
}

impl TwitchClient {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            token: token.into(),
        }
    }

    pub async fn get_channel_id(&self, channel: &str) -> Result<String, PlatformError> {
        static USERS_ENDPOINT: LazyLock<url::Url> =
            LazyLock::new(|| url::Url::parse("https://api.twitch.tv/helix/users").unwrap());

        let mut url = USERS_ENDPOINT.clone();
        url.query_pairs_mut().append_pair("login", channel).finish();

        self.client
            .get(url)
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", &self.token),
            )
            .send()
            .await?;

        todo!()
        // match channel {
        //     Ok(None) => Err(PlatformError::ChannelNotFound),
        //     Ok(Some(resp)) => Ok(resp.broadcaster_id.to_owned()),
        //     Err(e) => Err(PlatformError::PlatformError(super::Platform::Twitch)),
        // }
    }
}
