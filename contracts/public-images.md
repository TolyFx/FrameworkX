# 独立公开图片访问协议

状态：已实现；2026-10-06。HTTP 适配器为 `fx_storage_axum::public_image_routes`，默认转换器为 `fx_storage_image::ImageExtractor`。宿主只注入图片目录和转换器，不需要实例化 `StorageService`、用户、上传接口、配额或文件对象数据库。

## 接入

```rust
let images = fx_storage_axum::public_image_routes(
    fx_storage_axum::PublicImageState::new(
        "media",
        std::sync::Arc::new(fx_storage_image::ImageExtractor::new(400, 80)),
    ),
);
let app = axum::Router::new().nest("/media", images);
```

根目录由宿主指定，路径相对于该目录，不要求 `uploads/original` 布局。旧 `public_local_image_routes` 与 FlutterUnit 路由保持兼容；本入口为新增 API。

## 请求参数

示例：`/media/travel/example.png?w=400&h=300&fit=cover&format=webp`。

| 参数 | 默认值和约束 |
| --- | --- |
| w / h | 可选正整数；至少一个存在才转换；只传一项则推导另一项以保持比例 |
| fit | contain（等比放入）、cover（居中裁切填满）、fill（拉伸） |
| format | 默认 webp；支持 webp、png、jpeg/jpg |
| q | 默认 80；允许 40～95；当前编码器仅对 JPEG 应用质量参数，WebP 与 PNG 为无损输出 |

没有 w/h 时，即使带 format 或 q 也返回原图，保持既有 FlutterUnit 参数语义。未知参数被忽略。支持 GET/HEAD，POST 返回 405。

## 原图与变体

- 原图字节不变，使用 ServeDir 保留 Content-Type、Content-Length、HEAD、Last-Modified 条件请求和 Range/206。
- 变体在内存生成，不覆盖原文件，不写数据库。支持 HEAD、基于输出字节的 ETag 与 If-None-Match/304（强/弱、列表、通配符）。Cache-Control 为 `public, max-age=3600`。
- 变体不提供 Range 或 Last-Modified；Range 请求返回完整 200 响应。304 当前仍需要生成变体才能确认 ETag。没有服务端变体持久缓存。
- 变体的 MIME 与输出格式一致，URL 原始 .png 后缀不会强制输出 PNG。PNG/WebP 保留透明度，JPEG 不支持 alpha。
- 原图 SHA-256、尺寸与裁切区域不能用于变体校验。消费方应按完整 URL 分别缓存；图集裁切坐标仍采用原图像素，不能直接套到缩放后的图片上。

## 限制与错误

- 限定宿主图片根目录，canonicalize 后检查路径，拒绝越界符号链接；不列出目录，不开放非图片文件。接受 jpg/jpeg/png/gif/webp 源文件。
- 源文件默认上限 25 MiB，仅对转换请求生效；宿主可用 with_max_source_size 调整。
- 默认两个转换槽，可用 with_concurrency 调整。实际编码在 blocking 工作线程执行；取消 HTTP 请求不会提前释放仍在工作的槽位。原图不占转换槽。
- 默认 ImageExtractor 限制源图 3200 万像素，推导后的目标宽高均不超过 4096，目标总像素不超过 1600 万。
- 参数错误或不能解码的图片为 400；文件不存在/路径越界为 404；超出源文件字节上限为 413（IMAGE_SOURCE_TOO_LARGE）；转换槽已满为 503（IMAGE_TRANSFORM_BUSY），附 Retry-After: 1。
- 413/503 在图片适配器内明确写入 HTTP 与 JSON status，避免 CoreError 现有有限状态分类丢失这两个状态；未变更全局错误类型。

## 验证

在 FrameworkX/server 执行 `cargo test -p fx_storage_axum -p fx_storage_image --locked`。测试覆盖真实格式编码、比例与裁切、透明度、原图字节与 HEAD/Range/304、变体 ETag、非法参数、源大小、目录/符号链接、并发繁忙及推导尺寸预算。
