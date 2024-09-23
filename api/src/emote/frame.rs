use std::{io::Cursor, sync::LazyLock};

use axum::{
    body::Body,
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use http::{
    header::{CACHE_CONTROL, CONTENT_TYPE},
    HeaderValue,
};
use image::DynamicImage;

use crate::emote::DEFAULT_IMAGE_FORMAT;

#[derive(Clone)]
pub struct Frame {
    pub delay: f64,
    data: Bytes,
}

impl IntoResponse for Frame {
    fn into_response(self) -> axum::response::Response {
        static CACHE_HEADER: LazyLock<HeaderValue> = LazyLock::new(|| {
            format!("max-age={}, public", { 60 * 60 * 15 })
                .try_into()
                .expect("oh no")
        });

        let mut resp = Response::new(Body::from(self.data));
        resp.headers_mut().insert(
            CONTENT_TYPE,
            DEFAULT_IMAGE_FORMAT
                .to_mime_type()
                .try_into()
                .expect("this should never fail erm"),
        );
        resp.headers_mut()
            .insert(CACHE_CONTROL, CACHE_HEADER.clone());
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
    pub fn try_from_iter<'a>(
        iter: impl IntoIterator<Item = &'a image::Frame>,
    ) -> Result<Vec<Self>, image::ImageError> {
        let mut frames = Vec::new();
        for frame in iter {
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

impl TryFrom<&DynamicImage> for Frame {
    type Error = image::ImageError;

    fn try_from(value: &DynamicImage) -> Result<Self, Self::Error> {
        let mut buf = Cursor::new(Vec::new());
        value.write_to(&mut buf, DEFAULT_IMAGE_FORMAT)?;
        let buf = buf.into_inner();

        Ok(Self {
            delay: f64::MAX,
            data: buf.into(),
        })
    }
}
