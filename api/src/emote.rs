use std::{io::Cursor, sync::{Arc, LazyLock}};

use axum::{
    body::Body,
    http::{HeaderValue, Response},
    response::IntoResponse,
};
use bytes::Bytes;
use http::header::CACHE_CONTROL;
use image::{AnimationDecoder, DynamicImage};
use reqwest::header::CONTENT_TYPE;
use serde::Serialize;

use crate::platforms::{channel::ChannelEmote, Platform};

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
    #[error("request did not containt a Content-Type header")]
    MissingContentTypeHeader,
}

#[derive(Debug, Clone)]
pub struct Emote {
    pub id: Arc<str>,
    pub frames: Arc<[Frame]>,
}

impl Emote {
    pub fn try_new(
        data: &[u8],
        format: image::ImageFormat,
        id: impl Into<Arc<str>>,
    ) -> Result<Self, EmoteError> {
        use image::ImageFormat as Format;
        let frames: Arc<[Frame]> = match format {
            Format::Gif => {
                let decoder = image::codecs::gif::GifDecoder::new(Cursor::new(data))?;
                Frame::try_from_iter(decoder.into_frames())?.into()
            }
            Format::WebP => {
                let mut decoder = image::codecs::webp::WebPDecoder::new(Cursor::new(data))?;
                if decoder.has_animation() {
                    decoder.set_background_color(image::Rgba([0; 4]))?;
                    Frame::try_from_iter(decoder.into_frames())?.into()
                } else {
                    [Frame::try_from(image::load_from_memory_with_format(
                        data,
                        Format::WebP,
                    )?)?]
                    .into()
                }
            }
            f => [Frame::try_from(image::load_from_memory_with_format(
                data, f,
            )?)?]
            .into(),
        };

        Ok(Self {
            id: id.into(),
            frames,
        })
    }

    pub async fn try_from_response(
        resp: reqwest::Response,
        id: impl Into<Arc<str>>,
    ) -> Result<Self, EmoteError> {
        if let Some(content_type) = resp.headers().get(reqwest::header::CONTENT_TYPE) {
            let format = image::ImageFormat::from_mime_type(String::from_utf8_lossy(
                content_type.as_bytes(),
            ));

            if let Some(format) = format {
                let bytes = resp.bytes().await?;
                // wow that looks awful
                let id = Into::<Arc<str>>::into(id);

                let emote = tokio::task::spawn_blocking(move || Emote::try_new(&bytes, format, id))
                    .await
                    .expect("what.")?;

                Ok(emote)
            } else {
                Err(EmoteError::WrongMimeType(content_type.to_owned()))
            }
        } else {
            Err(EmoteError::MissingContentTypeHeader)
        }
    }
}

#[derive(Clone)]
pub struct Frame {
    pub delay: f64,
    data: Bytes,
}

impl IntoResponse for Frame {
    fn into_response(self) -> axum::response::Response {
        static CACHE_HEADER: LazyLock<HeaderValue> = LazyLock::new(|| {
            format!("max-age={}, public", {60 * 60 * 15}).try_into().expect("oh no")
        });

        let mut resp = Response::new(Body::from(self.data));
        resp.headers_mut().insert(
            CONTENT_TYPE,
            DEFAULT_IMAGE_FORMAT
                .to_mime_type()
                .try_into()
                .expect("this should never fail erm"),
        );
        resp.headers_mut().insert(
            CACHE_CONTROL,
            CACHE_HEADER.clone()
        );
        resp
    }
}

impl std::fmt::Debug for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Frame")
            .field("delay", &self.delay)
            .field("data", &"just a bunch of bytes...")
            .finish()
    }
}

impl Frame {
    fn try_from_iter(
        iter: impl IntoIterator<Item = Result<image::Frame, image::ImageError>>,
    ) -> Result<Vec<Self>, image::ImageError> {
        let mut frames = Vec::new();
        for i in iter {
            let frame = i?;
            let mut buf = Cursor::new(Vec::new());
            frame.buffer().write_to(&mut buf, DEFAULT_IMAGE_FORMAT)?;
            let buf = buf.into_inner();

            // i love coding
            let delay = std::time::Duration::from(frame.delay()).as_secs_f64();

            frames.push(Frame {
                delay,
                data: buf.into(),
            });
        }
        Ok(frames)
    }
}

impl TryFrom<DynamicImage> for Frame {
    type Error = image::ImageError;

    fn try_from(value: DynamicImage) -> Result<Self, Self::Error> {
        let mut buf = Cursor::new(Vec::new());
        value.write_to(&mut buf, DEFAULT_IMAGE_FORMAT)?;
        let buf = buf.into_inner();

        Ok(Self {
            delay: f64::MAX,
            data: buf.into(),
        })
    }
}

#[derive(Debug, Serialize)]
pub struct EmoteInfo<'a> {
    name: &'a str,
    id: &'a str,
    frame_count: usize,
    platform: Platform,
    frame_delays: Vec<f64>,
}

impl<'a> EmoteInfo<'a> {
    pub fn new(channel_info: &'a ChannelEmote, emote: &'a Emote) -> Self {
        Self {
            name: &channel_info.name,
            id: &emote.id,
            platform: channel_info.platform,
            frame_count: emote.frames.len(),
            frame_delays: emote.frames.iter().map(|f| f.delay).collect(),
        }
    }
}
