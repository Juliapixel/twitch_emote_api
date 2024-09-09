use api::{
    cli::ARGS,
    emote::EmoteInfo,
    platforms::{EmoteManager, PlatformError},
};
use axum::{
    body::Body,
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::get,
    Json,
};
use http::StatusCode;

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn core::error::Error>> {
    env_logger::init();

    let app = axum::Router::new()
        .route(
            "/emote/user/:channel/:name/:frame",
            get(channel_emote_frame),
        )
        .route("/emote/user/:channel/:name", get(channel_emote_info))
        .route("/user/:username", get(emotes_by_username))
        .with_state(
            EmoteManager::new(ARGS.client_id.as_str(), ARGS.client_secret.as_str())
                .await
                .unwrap(),
        );

    let socket = tokio::net::TcpListener::bind((std::net::Ipv6Addr::UNSPECIFIED, 8080)).await?;

    axum::serve(socket, app).await?;

    Ok(())
}

async fn emotes_by_username(
    Path(username): Path<String>,
    State(manager): State<EmoteManager>,
) -> Response {
    manager
        .get_channel_emotes(&username)
        .await
        .map(Json::from)
        .into_response()
}

async fn channel_emote_frame(
    Path((channel, name, frame)): Path<(String, String, u32)>,
    State(manager): State<EmoteManager>,
) -> Result<Response<Body>, PlatformError> {
    let emotes = manager.get_channel_emotes(&channel).await?;
    let info = emotes.get(&name).ok_or(PlatformError::EmoteNotFound)?;
    let emote = manager.get_emote(info.platform, &info.id).await?;

    match emote.frames.get(frame as usize) {
        Some(frame) => Ok(frame.clone().into_response()),
        None => Ok((StatusCode::NOT_FOUND, ()).into_response()),
    }
}

async fn channel_emote_info(
    Path((channel, name)): Path<(String, String)>,
    State(manager): State<EmoteManager>,
) -> Result<Response<Body>, PlatformError> {
    let emotes = manager.get_channel_emotes(&channel).await?;
    let info = emotes.get(&name).ok_or(PlatformError::EmoteNotFound)?;
    let emote = manager.get_emote(info.platform, &info.id).await?;

    Ok(Json::from(EmoteInfo::new(info.value(), &emote)).into_response())
}
