//! ImageExtractor 契约测试。
//!
//! 职责:验证图片宽高提取与 WebP 缩略图生成。
//! 边界:用 image crate 现造小图(无需外部 fixture 文件),不依赖对象存储/数据库。
//! 约束:仅测 MetadataExtractor::extract_image 语义;缩略图最长边裁剪逻辑由生成器保证。

use std::io::Cursor;

use fx_storage_image::ImageExtractor;
use fx_storage_service::{
    ImageFit, ImageMetadata, ImageOutputFormat, ImageTransform, MetadataExtractor,
};

/// 用 image crate 造一张 w×h 的纯色 PNG,返回字节
fn png_bytes(w: u32, h: u32) -> Vec<u8> {
    let img = image::RgbaImage::new(w, h);
    let mut buf = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(img)
        .write_to(&mut buf, image::ImageFormat::Png)
        .unwrap();
    buf.into_inner()
}

#[tokio::test]
async fn extract_image_returns_dims_and_webp_thumb() {
    let extractor = ImageExtractor::new(400, 80);
    let ImageMetadata {
        width,
        height,
        thumb,
        thumb_ext,
    } = extractor.extract_image(&png_bytes(8, 6)).await.unwrap();

    assert_eq!((width, height), (8, 6));
    assert!(!thumb.is_empty());
    assert_eq!(thumb_ext, "webp");
}

#[tokio::test]
async fn extract_image_preserves_dims_of_large_image() {
    // 大图(1000×500)仍应读出原始宽高,缩略图非空
    let extractor = ImageExtractor::new(400, 80);
    let meta = extractor
        .extract_image(&png_bytes(1000, 500))
        .await
        .unwrap();

    assert_eq!((meta.width, meta.height), (1000, 500));
    assert!(!meta.thumb.is_empty());
}

#[tokio::test]
async fn extract_image_invalid_bytes_errors() {
    let extractor = ImageExtractor::new(400, 80);
    let err = extractor.extract_image(b"not-an-image").await;
    assert!(err.is_err());
}

#[tokio::test]
async fn transform_image_supports_dynamic_cover_dimensions() {
    let extractor = ImageExtractor::new(400, 80);
    let output = extractor
        .transform_image(
            &png_bytes(1000, 500),
            ImageTransform {
                width: Some(240),
                height: Some(135),
                fit: ImageFit::Cover,
                quality: 80,
                format: ImageOutputFormat::WebP,
            },
        )
        .await
        .unwrap();
    let image = image::load_from_memory(&output.bytes).unwrap();
    assert_eq!((image.width(), image.height()), (240, 135));
    assert_eq!(output.mime_type, "image/webp");
}
