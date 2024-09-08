use std::{io::Cursor, sync::Arc};

use axum::http::{HeaderValue, Response};
use bytes::Bytes;
use image::{AnimationDecoder, DynamicImage};
use reqwest::header::CONTENT_TYPE;

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
pub struct ChannelEmote {
    pub name: String,
    pub emote: Emote,
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
                let decoder = image::codecs::webp::WebPDecoder::new(Cursor::new(data))?;
                if decoder.has_animation() {
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
    pub timestamp: f64,
    data: Bytes,
}

impl std::fmt::Debug for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Frame")
            .field("timestamp", &self.timestamp)
            .field("data", &"just a bunch of bytes...")
            .finish()
    }
}

impl Frame {
    pub fn into_response(self) -> Response<axum::body::Body> {
        let mut resp = axum::response::Response::new(self.data.into());
        resp.headers_mut().insert(
            CONTENT_TYPE,
            DEFAULT_IMAGE_FORMAT.to_mime_type().parse().unwrap(),
        );
        resp
    }

    fn try_from_iter(
        iter: impl IntoIterator<Item = Result<image::Frame, image::ImageError>>,
    ) -> Result<Vec<Self>, image::ImageError> {
        let mut timestamp: f64 = 0.0;
        let mut frames = Vec::new();
        for i in iter {
            let frame = i?;
            let mut buf = Cursor::new(Vec::new());
            frame.buffer().write_to(&mut buf, DEFAULT_IMAGE_FORMAT)?;
            let buf = buf.into_inner();

            frames.push(Frame {
                timestamp,
                data: buf.into(),
            });

            // why.
            timestamp += std::time::Duration::from(frame.delay()).as_secs_f64()
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
            timestamp: 0.0,
            data: buf.into(),
        })
    }
}
