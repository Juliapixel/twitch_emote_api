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
use image::{GenericImage, ImageError, RgbaImage};

#[derive(Clone)]
pub struct AtlasTexture {
    /// WebP encoded atlas image
    pub data: Bytes,
    pub frame_count: u32,
    pub x_size: u32,
    pub y_size: u32,
}

impl std::fmt::Debug for AtlasTexture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AtlasTexture")
            .field(
                "atlas",
                &format!("bunch of bytes!!!, length: {}", self.data.len()),
            )
            .field("frame_count", &self.frame_count)
            .field("x_size", &self.x_size)
            .field("y_size", &self.y_size)
            .finish()
    }
}

impl AtlasTexture {
    // TODO: make the atlas texture size always a power of two
    pub fn new<'a>(
        iter: impl IntoIterator<Item = &'a RgbaImage>,
        width: u32,
        height: u32,
        frame_count: u32,
    ) -> Result<Self, ImageError> {
        let x_size = f64::from(frame_count).sqrt().ceil() as u32;
        let y_size = {
            let full_lines = frame_count / x_size;
            if frame_count % x_size == 0 {
                full_lines
            } else {
                full_lines + 1
            }
        };
        let mut atlas = RgbaImage::new(width * x_size, height * y_size);
        for (i, frame) in iter.into_iter().enumerate() {
            let i = i as u32;
            atlas.copy_from(frame, width * (i % x_size), height * (i / x_size))?
        }

        let mut out = Cursor::new(Vec::new());
        atlas.write_to(&mut out, image::ImageFormat::WebP)?;

        Ok(Self {
            data: Bytes::from(out.into_inner()),
            frame_count,
            x_size,
            y_size,
        })
    }
}

impl IntoResponse for AtlasTexture {
    fn into_response(self) -> axum::response::Response {
        static CACHE_HEADER: LazyLock<HeaderValue> = LazyLock::new(|| {
            format!("max-age={}, public", { 60 * 60 * 15 })
                .try_into()
                .expect("oh no")
        });

        let mut resp = Response::new(Body::from(self.data));

        resp.headers_mut().insert(
            CONTENT_TYPE,
            image::ImageFormat::WebP
                .to_mime_type()
                .try_into()
                .expect("this should never fail erm"),
        );

        resp.headers_mut()
            .insert(CACHE_CONTROL, CACHE_HEADER.clone());
        resp
    }
}
