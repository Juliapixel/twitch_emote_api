use std::{io::Cursor, sync::Arc};

use atlas::AtlasTexture;
use frame::Frame;
use http::HeaderValue;
use image::AnimationDecoder;
use serde::Serialize;

use crate::platforms::{channel::ChannelEmote, Platform};

pub mod atlas;
pub mod frame;

pub const DEFAULT_IMAGE_FORMAT: image::ImageFormat = image::ImageFormat::WebP;

#[derive(Debug, thiserror::Error)]
pub enum EmoteError {
    #[error(transparent)]
    ImageError(#[from] image::ImageError),
    #[error(transparent)]
    RequestError(#[from] reqwest::Error),
    #[error("the provided url was not valid")]
    BadUrl,
    #[error("request returned wrong MIME type: ")]
    WrongMimeType(HeaderValue),
    #[error("request did not contain proper headers or wasn't a valid image")]
    UnableToDetermineFormat,
}

// AWFUL code
// TODO: make it less awful
fn atlas_and_frames_from_iter(
    frames: image::Frames,
) -> Result<(AtlasTexture, Vec<Frame>, u32, u32), EmoteError> {
    let collected_iter = frames.into_iter().collect_frames()?;
    let (width, height) = {
        let first = collected_iter.first().unwrap().buffer();
        (first.width(), first.height())
    };

    let frames = Frame::try_from_iter(collected_iter.iter())?;
    let atlas = AtlasTexture::new(
        collected_iter.iter().map(|f| f.buffer()),
        width,
        height,
        collected_iter.len() as u32,
    )?;

    Ok((atlas, frames, width, height))
}

#[derive(Debug, Clone)]
pub struct Emote {
    pub id: Arc<str>,
    pub width: u32,
    pub height: u32,
    pub frames: Arc<[Frame]>,
    pub atlas: Option<AtlasTexture>,
}

impl Emote {
    pub fn try_new(
        data: &[u8],
        format: image::ImageFormat,
        id: impl Into<Arc<str>>,
    ) -> Result<Self, EmoteError> {
        use image::ImageFormat as Format;
        let (atlas, frames, width, height) = match format {
            Format::Gif => {
                let decoder = image::codecs::gif::GifDecoder::new(Cursor::new(data))?;
                let (atlas, frames, width, height) =
                    atlas_and_frames_from_iter(decoder.into_frames())?;
                (Some(atlas), frames, width, height)
            }
            Format::WebP => {
                let mut decoder = image::codecs::webp::WebPDecoder::new(Cursor::new(data))?;
                if decoder.has_animation() {
                    decoder.set_background_color(image::Rgba([0; 4]))?;
                    let (atlas, frames, width, height) =
                        atlas_and_frames_from_iter(decoder.into_frames())?;
                    (Some(atlas), frames, width, height)
                } else {
                    let decoded = image::load_from_memory_with_format(data, Format::WebP)?;
                    let frame = Frame::try_from(&decoded)?;
                    (None, vec![frame], decoded.width(), decoded.height())
                }
            }
            f => {
                let decoded = image::load_from_memory_with_format(data, f)?;
                let frame = Frame::try_from(&decoded)?;
                (None, vec![frame], decoded.width(), decoded.height())
            }
        };

        Ok(Self {
            id: id.into(),
            width,
            height,
            frames: frames.into(),
            atlas,
        })
    }

    pub async fn try_from_response(
        resp: reqwest::Response,
        id: impl Into<Arc<str>>,
    ) -> Result<Self, EmoteError> {
        let bytes;

        // either take from the headers or guess with magic bytes (because of
        // fucking OpieOP emote and other weird twitch emotes)
        let format = resp.headers().get(reqwest::header::CONTENT_TYPE)
            .and_then(|h| {
                image::ImageFormat::from_mime_type(String::from_utf8_lossy(
                    h.as_bytes(),
                ))
            })
            .or_else({
                bytes = resp.bytes().await?;
                || {
                    image::ImageReader::new(Cursor::new(&bytes))
                        .with_guessed_format()
                        .ok()?
                        .format()
                }
            });

        if let Some(format) = format {
            // wow that looks awful
            let id = Into::<Arc<str>>::into(id);

            let emote = tokio::task::spawn_blocking(move || Emote::try_new(&bytes, format, id))
                .await
                .expect("what.")?;

            Ok(emote)

        } else {
            Err(EmoteError::UnableToDetermineFormat)
        }
    }
}

#[derive(Debug, Serialize)]
pub struct EmoteInfo<'a> {
    name: &'a str,
    id: &'a str,
    width: u32,
    height: u32,
    animated: bool,
    platform: Platform,
    frame_count: usize,
    frame_delays: Vec<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    atlas_info: Option<AtlasInfo>,
}

impl<'a> EmoteInfo<'a> {
    pub fn new(channel_info: &'a ChannelEmote, emote: &'a Emote) -> Self {
        let atlas_info = emote.atlas.as_ref().map(AtlasInfo::new);
        Self {
            name: &channel_info.name,
            id: &emote.id,
            width: emote.width,
            height: emote.height,
            animated: channel_info.animated,
            platform: channel_info.platform,
            frame_count: emote.frames.len(),
            frame_delays: emote.frames.iter().map(|f| f.delay).collect(),
            atlas_info,
        }
    }

    pub fn new_twitch(emote: &'a Emote) -> Self {
        let atlas_info = emote.atlas.as_ref().map(AtlasInfo::new);
        Self {
            name: &emote.id,
            id: &emote.id,
            width: emote.width,
            height: emote.height,
            animated: emote.atlas.is_some(),
            platform: Platform::Twitch,
            frame_count: emote.frames.len(),
            frame_delays: emote.frames.iter().map(|f| f.delay).collect(),
            atlas_info,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AtlasInfo {
    x_size: u32,
    y_size: u32,
}

impl AtlasInfo {
    fn new(atlas: &AtlasTexture) -> Self {
        Self {
            x_size: atlas.x_size,
            y_size: atlas.y_size,
        }
    }
}
