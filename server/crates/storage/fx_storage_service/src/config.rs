//! 存储服务配置与通用工具(路径生成/扩展名/文件名截断/MIME 推断)。
//!
//! 职责:提供上传大小上限配置与上传路径相关纯函数,供 `StorageService` 编排使用。
//! 边界:不含后端 URL 前缀(归属后端),不含用户/配额业务参数(归属端口实现)。
//! 约束:路径按 `YYYY/MM` + UUID 生成,避免单目录文件爆炸与命名冲突。

use std::path::Path;

/// 上传大小上限配置(与业务无关,宿主可从环境变量覆盖)
#[derive(Debug, Clone)]
pub struct StorageServiceConfig {
    pub max_image_size: u64,
    pub max_video_size: u64,
    pub max_file_size: u64,
}

impl Default for StorageServiceConfig {
    fn default() -> Self {
        Self {
            max_image_size: 20 * 1024 * 1024,
            max_video_size: 50 * 1024 * 1024,
            max_file_size: 50 * 1024 * 1024,
        }
    }
}

impl StorageServiceConfig {
    pub fn from_env() -> Self {
        let mut c = Self::default();
        if let Some(n) = env_u64("UPLOAD_MAX_IMAGE_SIZE") {
            c.max_image_size = n;
        }
        if let Some(n) = env_u64("UPLOAD_MAX_VIDEO_SIZE") {
            c.max_video_size = n;
        }
        if let Some(n) = env_u64("UPLOAD_MAX_FILE_SIZE") {
            c.max_file_size = n;
        }
        c
    }
}

fn env_u64(key: &str) -> Option<u64> {
    std::env::var(key).ok().and_then(|v| v.parse().ok())
}

/// 日期分层路径 `YYYY/MM`
pub(crate) fn date_path() -> String {
    chrono::Utc::now().format("%Y/%m").to_string()
}

/// 取文件名扩展名(小写),缺省回退
pub(crate) fn ext_of(filename: &str, default: &str) -> String {
    Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or(default)
        .to_lowercase()
}

/// 截断文件名,保留尾部(含扩展名),按字符计避免 UTF-8 边界 panic
pub(crate) fn truncate_filename(name: &str, max: usize) -> String {
    let chars: Vec<char> = name.chars().collect();
    if chars.len() <= max {
        return name.to_string();
    }
    let tail: String = chars[chars.len() - max..].iter().collect();
    format!("…{}", tail)
}

/// 按扩展名推断 MIME type
pub(crate) fn mime_from_ext(ext: &str) -> String {
    match ext {
        "pdf" => "application/pdf",
        "doc" | "docx" => "application/msword",
        "xls" | "xlsx" => "application/vnd.ms-excel",
        "ppt" | "pptx" => "application/vnd.ms-powerpoint",
        "zip" => "application/zip",
        "rar" => "application/x-rar-compressed",
        "txt" => "text/plain",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "aac" => "audio/aac",
        "ogg" => "audio/ogg",
        "m4a" => "audio/mp4",
        _ => "application/octet-stream",
    }
    .to_string()
}
