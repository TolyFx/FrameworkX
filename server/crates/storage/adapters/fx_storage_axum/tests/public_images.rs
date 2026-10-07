//! 真实编码器与 HTTP 路由测试，不依赖数据库、上传服务或磁盘发布目录。
use std::{io::Cursor, sync::Arc};

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
    response::Response,
};
use fx_storage_axum::{PublicImageState, public_image_routes};
use fx_storage_image::ImageExtractor;
use image::GenericImageView;
use tempfile::TempDir;
use tower::ServiceExt;

fn fixture() -> (TempDir, Router, Vec<u8>) {
    let directory = TempDir::new().unwrap();
    let mut output = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(image::RgbaImage::new(60, 40))
        .write_to(&mut output, image::ImageFormat::Png)
        .unwrap();
    let bytes = output.into_inner();
    std::fs::create_dir(directory.path().join("travel")).unwrap();
    std::fs::write(directory.path().join("travel/atlas.png"), &bytes).unwrap();
    let router = Router::new().nest(
        "/media",
        public_image_routes(PublicImageState::new(
            directory.path(),
            Arc::new(ImageExtractor::new(400, 80)),
        )),
    );
    (directory, router, bytes)
}

async fn fetch(router: &Router, method: &str, uri: &str, headers: &[(&str, &str)]) -> Response {
    let mut request = Request::builder().method(method).uri(uri);
    for (key, value) in headers {
        request = request.header(*key, *value);
    }
    router
        .clone()
        .oneshot(request.body(Body::empty()).unwrap())
        .await
        .unwrap()
}

#[tokio::test]
async fn original_preserves_bytes_head_range_and_conditional_cache() {
    let (_directory, router, original) = fixture();
    let response = fetch(&router, "GET", "/media/travel/atlas.png", &[]).await;
    assert_eq!(response.status(), StatusCode::OK);
    let modified = response.headers()[header::LAST_MODIFIED]
        .to_str()
        .unwrap()
        .to_owned();
    assert_eq!(
        to_bytes(response.into_body(), usize::MAX).await.unwrap(),
        original
    );
    let response = fetch(&router, "HEAD", "/media/travel/atlas.png", &[]).await;
    assert_eq!(
        response.headers()[header::CONTENT_LENGTH],
        original.len().to_string()
    );
    assert!(
        to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap()
            .is_empty()
    );
    let response = fetch(
        &router,
        "GET",
        "/media/travel/atlas.png",
        &[("range", "bytes=0-7")],
    )
    .await;
    assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(
        to_bytes(response.into_body(), usize::MAX).await.unwrap(),
        &original[..8]
    );
    let response = fetch(
        &router,
        "GET",
        "/media/travel/atlas.png",
        &[("if-modified-since", &modified)],
    )
    .await;
    assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
}

#[tokio::test]
async fn variants_support_formats_fits_alpha_and_do_not_modify_original() {
    let (directory, router, original) = fixture();
    for (query, expected, mime) in [
        ("w=30", (30, 20), "image/webp"),
        ("w=31", (31, 20), "image/webp"),
        ("h=10&format=png", (15, 10), "image/png"),
        ("w=20&h=20&fit=contain&format=png", (20, 13), "image/png"),
        (
            "w=20&h=20&fit=cover&format=jpeg&q=70",
            (20, 20),
            "image/jpeg",
        ),
        ("w=20&h=20&fit=fill&format=webp", (20, 20), "image/webp"),
    ] {
        let response = fetch(
            &router,
            "GET",
            &format!("/media/travel/atlas.png?{query}"),
            &[],
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK, "{query}");
        assert_eq!(response.headers()[header::CONTENT_TYPE], mime);
        let output = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let image = image::load_from_memory(&output).unwrap();
        assert_eq!(image.dimensions(), expected);
        if mime != "image/jpeg" {
            assert_eq!(image.to_rgba8().get_pixel(0, 0).0[3], 0);
        }
    }
    assert_eq!(
        std::fs::read(directory.path().join("travel/atlas.png")).unwrap(),
        original
    );
}

#[tokio::test]
async fn variant_head_etag_and_weak_conditional_requests_work() {
    let (_directory, router, _) = fixture();
    let uri = "/media/travel/atlas.png?w=20&format=png";
    let response = fetch(&router, "GET", uri, &[]).await;
    let etag = response.headers()[header::ETAG]
        .to_str()
        .unwrap()
        .to_owned();
    let length = response.headers()[header::CONTENT_LENGTH].clone();
    let head = fetch(&router, "HEAD", uri, &[]).await;
    assert_eq!(head.headers()[header::ETAG], etag);
    assert_eq!(head.headers()[header::CONTENT_LENGTH], length);
    assert!(
        to_bytes(head.into_body(), usize::MAX)
            .await
            .unwrap()
            .is_empty()
    );
    let conditional = fetch(
        &router,
        "GET",
        uri,
        &[("if-none-match", &format!("\"unrelated\", W/{etag}"))],
    )
    .await;
    assert_eq!(conditional.status(), StatusCode::NOT_MODIFIED);
    assert!(
        to_bytes(conditional.into_body(), usize::MAX)
            .await
            .unwrap()
            .is_empty()
    );
    let other = fetch(
        &router,
        "GET",
        "/media/travel/atlas.png?w=10&format=png",
        &[],
    )
    .await;
    assert_ne!(other.headers()[header::ETAG], etag);
}

#[tokio::test]
async fn invalid_queries_missing_files_and_non_images_are_rejected() {
    let (directory, router, _) = fixture();
    for query in [
        "w=0",
        "w=4097",
        "w=4000&h=4096",
        "w=nope",
        "w=20&fit=bad",
        "w=20&q=30",
        "w=20&format=gif",
    ] {
        assert_eq!(
            fetch(
                &router,
                "GET",
                &format!("/media/travel/atlas.png?{query}"),
                &[]
            )
            .await
            .status(),
            StatusCode::BAD_REQUEST
        );
    }
    assert_eq!(
        fetch(&router, "GET", "/media/missing.png?w=20", &[])
            .await
            .status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        fetch(&router, "GET", "/media/travel", &[]).await.status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        fetch(&router, "POST", "/media/travel/atlas.png", &[])
            .await
            .status(),
        StatusCode::METHOD_NOT_ALLOWED
    );
    std::fs::write(directory.path().join("private.env"), "private").unwrap();
    assert_eq!(
        fetch(&router, "GET", "/media/private.env?w=20", &[])
            .await
            .status(),
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn oversized_sources_and_symlinks_outside_root_are_rejected() {
    let (directory, _, _) = fixture();
    let router = public_image_routes(
        PublicImageState::new(directory.path(), Arc::new(ImageExtractor::new(400, 80)))
            .with_max_source_size(1),
    );
    assert_eq!(
        fetch(&router, "GET", "/travel/atlas.png?w=20", &[])
            .await
            .status(),
        StatusCode::PAYLOAD_TOO_LARGE
    );
    // 原图仍沿用静态文件规则，不受变体读取上限影响。
    assert_eq!(
        fetch(&router, "GET", "/travel/atlas.png", &[])
            .await
            .status(),
        StatusCode::OK
    );
    #[cfg(unix)]
    {
        let outside = TempDir::new().unwrap();
        std::fs::write(outside.path().join("outside.png"), "private").unwrap();
        std::os::unix::fs::symlink(
            outside.path().join("outside.png"),
            directory.path().join("link.png"),
        )
        .unwrap();
        for uri in ["/link.png", "/link.png?w=20", "/%2e%2e/outside.png?w=20"] {
            assert_eq!(
                fetch(&router, "GET", uri, &[]).await.status(),
                StatusCode::NOT_FOUND
            );
        }
    }
}
