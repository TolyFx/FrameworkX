//! 图像元数据与缩略图提取。
//!
//! 职责:实现 `fx_storage::MetadataExtractor`,解码取宽高并生成 WebP 缩略图。
//! 边界:只依赖 `fx_storage`(端口)与 `image` crate;无 IO/网络/宿主依赖。
//! 约束:缩略图按最长边等比缩放(Lanczos3),输出 WebP;解码失败映射为 `StorageError::Image`。

use std::io::Cursor;

use async_trait::async_trait;
use fx_storage_service::{ImageMetadata, MetadataExtractor, StorageError};
use image::{DynamicImage, ImageFormat, ImageReader, imageops::FilterType};

pub struct ImageExtractor {
    max_size: u32,
    #[allow(dead_code)]
    quality: u8,
}

impl ImageExtractor {
    pub fn new(max_size: u32, quality: u8) -> Self {
        Self { max_size, quality }
    }
}

#[async_trait]
impl MetadataExtractor for ImageExtractor {
    async fn extract_image(&self, data: &[u8]) -> Result<ImageMetadata, StorageError> {
        let img = ImageReader::new(Cursor::new(data))
            .with_guessed_format()
            .map_err(|e| StorageError::Image(e.to_string()))?
            .decode()
            .map_err(|e| StorageError::Image(e.to_string()))?;

        let width = img.width();
        let height = img.height();
        let thumb = generate_thumbnail(&img, self.max_size);

        let mut buf = Cursor::new(Vec::new());
        thumb
            .write_to(&mut buf, ImageFormat::WebP)
            .map_err(|e| StorageError::Image(e.to_string()))?;

        Ok(ImageMetadata {
            width,
            height,
            thumb: buf.into_inner(),
            thumb_ext: "webp",
        })
    }
}

fn generate_thumbnail(img: &DynamicImage, max_size: u32) -> DynamicImage {
    let (w, h) = (img.width(), img.height());
    if w <= max_size && h <= max_size {
        return img.clone();
    }

    let ratio = if w > h {
        max_size as f32 / w as f32
    } else {
        max_size as f32 / h as f32
    };

    let new_w = (w as f32 * ratio) as u32;
    let new_h = (h as f32 * ratio) as u32;
    img.resize(new_w, new_h, FilterType::Lanczos3)
}
