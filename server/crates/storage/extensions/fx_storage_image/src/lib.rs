//! 图像元数据与缩略图提取。
//!
//! 职责:实现 `fx_storage::MetadataExtractor`,解码取宽高并生成 WebP 缩略图。
//! 边界:只依赖 `fx_storage`(端口)与 `image` crate;无 IO/网络/宿主依赖。
//! 约束:缩略图按最长边等比缩放(Lanczos3),输出 WebP;解码失败映射为 `StorageError::Image`。

use std::io::Cursor;

use async_trait::async_trait;
use fx_storage_service::{
    ImageFit, ImageMetadata, ImageOutputFormat, ImageTransform, MetadataExtractor, StorageError,
    TransformedImage,
};
use image::{DynamicImage, ImageEncoder, ImageFormat, ImageReader, imageops::FilterType};

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

    async fn transform_image(
        &self,
        data: &[u8],
        transform: ImageTransform,
    ) -> Result<TransformedImage, StorageError> {
        let image = ImageReader::new(Cursor::new(data))
            .with_guessed_format()
            .map_err(image_error)?
            .decode()
            .map_err(image_error)?;
        let image = transform_dimensions(image, transform)?;
        let mut bytes = Vec::new();
        let mime_type = match transform.format {
            ImageOutputFormat::WebP => {
                image
                    .write_to(&mut Cursor::new(&mut bytes), ImageFormat::WebP)
                    .map_err(image_error)?;
                "image/webp"
            }
            ImageOutputFormat::Png => {
                image
                    .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
                    .map_err(image_error)?;
                "image/png"
            }
            ImageOutputFormat::Jpeg => {
                let rgb = image.to_rgb8();
                image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, transform.quality)
                    .write_image(
                        rgb.as_raw(),
                        rgb.width(),
                        rgb.height(),
                        image::ExtendedColorType::Rgb8,
                    )
                    .map_err(image_error)?;
                "image/jpeg"
            }
        };
        Ok(TransformedImage { bytes, mime_type })
    }
}

fn transform_dimensions(
    image: DynamicImage,
    transform: ImageTransform,
) -> Result<DynamicImage, StorageError> {
    let (source_width, source_height) = (image.width(), image.height());
    let (width, height) = match (transform.width, transform.height) {
        (Some(width), Some(height)) => (width, height),
        (Some(width), None) => (
            width,
            ((source_height as u64 * width as u64) / source_width as u64).max(1) as u32,
        ),
        (None, Some(height)) => (
            ((source_width as u64 * height as u64) / source_height as u64).max(1) as u32,
            height,
        ),
        (None, None) => return Err(StorageError::Image("缺少目标宽度或高度".to_owned())),
    };
    Ok(match transform.fit {
        ImageFit::Contain => image.resize(width, height, FilterType::Lanczos3),
        ImageFit::Cover => image.resize_to_fill(width, height, FilterType::Lanczos3),
        ImageFit::Fill => image.resize_exact(width, height, FilterType::Lanczos3),
    })
}

fn image_error(error: impl std::fmt::Display) -> StorageError {
    StorageError::Image(error.to_string())
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
