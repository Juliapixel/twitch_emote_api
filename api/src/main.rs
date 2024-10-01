use std::{os::unix::ffi::OsStrExt, sync::LazyLock, time::Duration};

use twitch_emote_api::{
    cli::ARGS,
    emote::EmoteInfo,
    platforms::{EmoteManager, Platform, PlatformError},
};
use axum::{
    body::Body, extract::{Path, Extension}, response::{IntoResponse, Response}, routing::get, Extension as ExtensionLayer, Json
};
use config::builder::DefaultState;
use futures::FutureExt;
use http::{header::CACHE_CONTROL, HeaderValue, StatusCode};
use tokio::signal::unix::SignalKind;

#[global_allocator]
#[cfg(target_os = "linux")]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn core::error::Error>> {
    tracing_subscriber::FmtSubscriber::builder()
        .with_level(true)
        .with_max_level(tracing::Level::INFO)
        .init();

    // SOON
    /*
    let config_path = std::env::current_dir()
        .unwrap()
        .with_file_name("config.yml");

    let config = config::ConfigBuilder::<DefaultState>::default()
        .add_source(config::File::new(
            &config_path.to_string_lossy(),
            config::FileFormat::Yaml,
        ))
        .build()?;

    let db_path = config.get::<std::path::PathBuf>("db_path")?;

    // touch.....
    if !db_path.exists() {
        std::fs::OpenOptions::new().create(true).truncate(false).write(true).open(&db_path)?;
    }

    let pool = sqlx::SqlitePool::connect(
        format!(
            "sqlite://{}",
            String::from_utf8_lossy(
                db_path
                    .canonicalize()
                    .expect("failed to canonicalize db path")
                    .as_os_str()
                    .as_bytes()
            )
        )
        .as_str(),
    )
    .await
    .expect("failed to open sqlite pool");
    */

    let app = axum::Router::new()
        .route("/user/:username", get(emotes_by_username))
        .route("/emote/:channel/:name/:frame", get(channel_emote_frame))
        .route("/emote/:channel/:name", get(channel_emote_info))
        .route("/emote/:channel/:name/atlas.webp", get(channel_emote_atlas))
        .route("/emote/twitch/:id", get(twitch_emote_info))
        .route("/emote/twitch/:id/atlas.webp", get(twitch_emote_atlas))
        .route("/emote/twitch/:id/:frame", get(twitch_emote_frame))
        .route("/emote/globals/:platform", get(platform_global_emotes))
        .route(
            "/emote/globals/:platform/:name",
            get(platform_global_emote_info),
        )
        .route(
            "/emote/globals/:platform/:name/:frame",
            get(platform_global_emote_frame),
        )
        .route(
            "/emote/globals/:platform/:name/atlas.webp",
            get(platform_global_emote_atlas),
        )
        .layer(tower_http::cors::CorsLayer::permissive())
        .layer(tower_http::compression::CompressionLayer::new().no_zstd())
        .layer(
            tower_http::trace::TraceLayer::new_for_http()
                .make_span_with(|req: &http::Request<Body>| {
                    tracing::info_span!(
                        "http-request",
                        path = req.uri().path(),
                        latency_millis = tracing::field::Empty,
                        status_code = tracing::field::Empty,
                    )
                })
                .on_response(|response: &Response<Body>, latency: Duration, span: &tracing::Span| {
                    span.record("latency_millis", latency.as_secs_f32() * 1000.0);
                    span.record("status_code", response.status().as_u16());
                    tracing::info!("request finished")
                })
        )
        // .layer(ExtensionLayer(pool))
        .layer(
            ExtensionLayer(
                EmoteManager::new(ARGS.client_id.as_str(), ARGS.client_secret.as_str())
                    .await
                    .unwrap()
            )
        );

    let socket =
        tokio::net::TcpListener::bind((std::net::Ipv6Addr::UNSPECIFIED, ARGS.port)).await?;

    #[cfg(unix)]
    axum::serve(socket, app)
        .with_graceful_shutdown(async {
            let mut sigterm = tokio::signal::unix::signal(SignalKind::terminate()).unwrap();
            let mut sigint = tokio::signal::unix::signal(SignalKind::interrupt()).unwrap();

            futures::select! {
                _ = Box::pin(sigterm.recv().fuse()) => (),
                _ = Box::pin(sigint.recv().fuse()) => (),
            }
        })
        .await?;

    #[cfg(not(unix))]
    axum::serve(socket, app).await?;

    Ok(())
}

async fn emotes_by_username(
    Path(username): Path<String>,
    Extension(manager): Extension<EmoteManager>,
) -> Response {
    manager
        .get_channel_emotes(&username)
        .await
        .map(Json::from)
        .into_response()
}

async fn channel_emote_frame(
    Path((channel, name, frame)): Path<(String, String, String)>,
    Extension(manager): Extension<EmoteManager>,
) -> Result<Response<Body>, PlatformError> {
    let frame_requested = frame.to_lowercase();
    if !frame.ends_with(".webp") {
        return Err(PlatformError::EmoteNotFound);
    }

    let number = frame_requested[0..frame_requested.len() - 5].parse::<u32>();

    match number {
        Ok(frame) => {
            let emotes = manager.get_channel_emotes(&channel).await?;
            let info = emotes.get(&name).ok_or(PlatformError::EmoteNotFound)?;
            let emote = manager.get_emote(info.platform, &info.id).await?;

            match emote.frames.get(frame as usize) {
                Some(frame) => Ok(frame.clone().into_response()),
                None => Ok((StatusCode::NOT_FOUND, ()).into_response()),
            }
        }
        Err(_) => Err(PlatformError::EmoteNotFound),
    }
}

async fn channel_emote_info(
    Path((channel, name)): Path<(String, String)>,
    Extension(manager): Extension<EmoteManager>,
) -> Result<Response<Body>, PlatformError> {
    static CACHE_HEADER: LazyLock<HeaderValue> = LazyLock::new(|| {
        format!("max-age={}, public", { 60 * 60 * 15 })
            .try_into()
            .expect("oh no")
    });

    let emotes = manager.get_channel_emotes(&channel).await?;
    let info = emotes.get(&name).ok_or(PlatformError::EmoteNotFound)?;
    let emote = manager.get_emote(info.platform, &info.id).await?;

    let mut resp = Json::from(EmoteInfo::new(info.value(), &emote)).into_response();

    resp.headers_mut()
        .insert(CACHE_CONTROL, CACHE_HEADER.clone());
    Ok(resp)
}

async fn channel_emote_atlas(
    Path((channel, name)): Path<(String, String)>,
    Extension(manager): Extension<EmoteManager>,
) -> Result<Response<Body>, PlatformError> {
    let emotes = manager.get_channel_emotes(&channel).await?;
    let info = emotes.get(&name).ok_or(PlatformError::EmoteNotFound)?;
    let emote = manager.get_emote(info.platform, &info.id).await?;

    if let Some(atlas) = emote.atlas {
        Ok(atlas.into_response())
    } else {
        Ok((StatusCode::NOT_FOUND, ()).into_response())
    }
}

async fn twitch_emote_info(
    Path(id): Path<String>,
    Extension(manager): Extension<EmoteManager>,
) -> Result<Response<Body>, PlatformError> {
    static CACHE_HEADER: LazyLock<HeaderValue> = LazyLock::new(|| {
        format!("max-age={}, public", { 60 * 60 * 15 })
            .try_into()
            .expect("oh no")
    });

    let emote = manager.get_emote(Platform::Twitch, &id).await?;

    let mut resp = Json::from(EmoteInfo::new_twitch(&emote)).into_response();

    resp.headers_mut()
        .insert(CACHE_CONTROL, CACHE_HEADER.clone());
    Ok(resp)
}

async fn twitch_emote_frame(
    Path((id, frame)): Path<(String, String)>,
    Extension(manager): Extension<EmoteManager>,
) -> Result<Response<Body>, PlatformError> {
    let frame_requested = frame.to_lowercase();
    if !frame.ends_with(".webp") {
        return Err(PlatformError::EmoteNotFound);
    }

    let number = frame_requested[0..frame_requested.len() - 5].parse::<u32>();

    match number {
        Ok(frame) => {
            let emote = manager.get_emote(Platform::Twitch, &id).await?;

            match emote.frames.get(frame as usize) {
                Some(frame) => Ok(frame.clone().into_response()),
                None => Ok((StatusCode::NOT_FOUND, ()).into_response()),
            }
        }
        Err(_) => Err(PlatformError::EmoteNotFound),
    }
}

async fn twitch_emote_atlas(
    Path(id): Path<String>,
    Extension(manager): Extension<EmoteManager>,
) -> Result<Response<Body>, PlatformError> {
    let emote = manager.get_emote(Platform::Twitch, &id).await?;

    if let Some(atlas) = emote.atlas {
        Ok(atlas.into_response())
    } else {
        Ok((StatusCode::NOT_FOUND, ()).into_response())
    }
}

async fn platform_global_emotes(
    Path(platform): Path<Platform>,
    Extension(manager): Extension<EmoteManager>,
) -> Result<Response<Body>, PlatformError> {
    Ok(Json::from(manager.get_global_emotes(platform).await?).into_response())
}

async fn platform_global_emote_info(
    Path((platform, emote)): Path<(Platform, String)>,
    Extension(manager): Extension<EmoteManager>,
) -> Result<Response<Body>, PlatformError> {
    static CACHE_HEADER: LazyLock<HeaderValue> = LazyLock::new(|| {
        format!("max-age={}, public", { 60 * 60 * 24 })
            .try_into()
            .expect("oh no")
    });

    let emotes = manager.get_global_emotes(platform).await?;
    let info = emotes.get(&emote).ok_or(PlatformError::EmoteNotFound)?;
    let emote = manager.get_emote(info.platform, &info.id).await?;

    let mut resp = Json::from(EmoteInfo::new(info.value(), &emote)).into_response();

    resp.headers_mut()
        .insert(CACHE_CONTROL, CACHE_HEADER.clone());
    Ok(resp)
}

async fn platform_global_emote_frame(
    Path((platform, emote, frame)): Path<(Platform, String, String)>,
    Extension(manager): Extension<EmoteManager>,
) -> Result<Response<Body>, PlatformError> {
    let frame_requested = frame.to_lowercase();
    if !frame.ends_with(".webp") {
        return Err(PlatformError::EmoteNotFound);
    }

    let number = frame_requested[0..frame_requested.len() - 5].parse::<u32>();

    match number {
        Ok(frame) => match manager.get_global_emotes(platform).await?.get(&emote) {
            Some(info) => {
                let emote = manager.get_emote(info.platform, &info.id).await?;

                match emote.frames.get(frame as usize) {
                    Some(frame) => Ok(frame.clone().into_response()),
                    None => Ok((StatusCode::NOT_FOUND, ()).into_response()),
                }
            }
            None => Err(PlatformError::EmoteNotFound),
        },
        Err(_) => Err(PlatformError::EmoteNotFound),
    }
}

async fn platform_global_emote_atlas(
    Path((platform, emote)): Path<(Platform, String)>,
    Extension(manager): Extension<EmoteManager>,
) -> Result<Response<Body>, PlatformError> {
    match manager.get_global_emotes(platform).await?.get(&emote) {
        Some(info) => {
            let emote = manager.get_emote(info.platform, &info.id).await?;

            match emote.atlas {
                Some(atlas) => Ok(atlas.clone().into_response()),
                None => Ok((StatusCode::NOT_FOUND, ()).into_response()),
            }
        }
        None => Err(PlatformError::EmoteNotFound),
    }
}
