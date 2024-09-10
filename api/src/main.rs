use std::sync::LazyLock;

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
use http::{header::CACHE_CONTROL, HeaderValue, StatusCode};
use regex::Regex;

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn core::error::Error>> {
    env_logger::init();

    let app = axum::Router::new()
        .route(
            "/emote/:channel/:name/:frame",
            get(channel_emote_frame),
        )
        .route(
            "/emote/:channel/:name",
            get(channel_emote_info),
        )
        .route(
            "/user/:username",
            get(emotes_by_username)
            .route_layer(
                tower_http::compression::CompressionLayer::new()
                    .gzip(true)
                    .br(true)
            )
        )
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(
            EmoteManager::new(ARGS.client_id.as_str(), ARGS.client_secret.as_str())
                .await
                .unwrap(),
        );

    let socket = tokio::net::TcpListener::bind((std::net::Ipv6Addr::UNSPECIFIED, ARGS.port)).await?;

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
    Path((channel, name, frame)): Path<(String, String, String)>,
    State(manager): State<EmoteManager>,
) -> Result<Response<Body>, PlatformError> {
    static WEBP_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"\d+\.").unwrap()
    });

    let frame_requested = frame.to_lowercase();
    if !frame.ends_with(".webp") {
        return Err(PlatformError::EmoteNotFound)
    }

    let number = frame_requested[0..frame_requested.len()-5].parse::<u32>();

    match number {
        Ok(frame) => {
            let emotes = manager.get_channel_emotes(&channel).await?;
            let info = emotes.get(&name).ok_or(PlatformError::EmoteNotFound)?;
            let emote = manager.get_emote(info.platform, &info.id).await?;

            match emote.frames.get(frame as usize) {
                Some(frame) => Ok(frame.clone().into_response()),
                None => Ok((StatusCode::NOT_FOUND, ()).into_response()),
            }
        },
        Err(e) => Err(PlatformError::EmoteNotFound),
    }


}

async fn channel_emote_info(
    Path((channel, name)): Path<(String, String)>,
    State(manager): State<EmoteManager>,
) -> Result<Response<Body>, PlatformError> {
    static CACHE_HEADER: LazyLock<HeaderValue> = LazyLock::new(|| {
        format!("max-age={}, public", {60 * 60 * 15}).try_into().expect("oh no")
    });

    let emotes = manager.get_channel_emotes(&channel).await?;
    let info = emotes.get(&name).ok_or(PlatformError::EmoteNotFound)?;
    let emote = manager.get_emote(info.platform, &info.id).await?;

    let mut resp = Json::from(EmoteInfo::new(info.value(), &emote)).into_response();

    resp
        .headers_mut()
        .insert(
            CACHE_CONTROL,
            CACHE_HEADER.clone()
        );
    Ok(resp)
}
