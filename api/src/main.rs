use api::platforms::{EmoteManager, Platform};
use axum::{
    body::Body,
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::get,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn core::error::Error>> {
    let app = axum::Router::new()
        .route("/emote/:platform/:id", get(emote_by_platform))
        .route(
            "/emote/user/:channel/:name/:frame",
            get(channel_emote_frame),
        )
        .route("/user/:username", get(emotes_by_username))
        .with_state(EmoteManager::new(todo!("token")));

    let socket = tokio::net::TcpListener::bind((std::net::Ipv6Addr::UNSPECIFIED, 8080)).await?;

    axum::serve(socket, app).await?;

    Ok(())
}

async fn emotes_by_username(Path(username): Path<String>) -> impl IntoResponse {
    format!("Hello, {}!", username)
}

async fn emote_by_platform(
    Path((platform, id)): Path<(Platform, String)>,
    State(manager): State<EmoteManager>,
) -> Response<Body> {
    match manager.get_emote(platform, id.as_str()).await {
        Ok(emote) => emote.frames.first().unwrap().clone().into_response(),
        Err(e) => Response::new(Body::from(e.to_string())),
    }
}

async fn channel_emote_frame(
    Path((channel, name, frame)): Path<(String, String, u32)>,
) -> impl IntoResponse {
}
