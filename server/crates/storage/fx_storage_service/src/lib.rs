//! 可复用文件存储编排层。
//!
//! 职责:在核心协议之上编排上传/去重/引用计数/删除,定义宿主注入的四个端口。
//! 边界:依赖 `fx_storage_core`;不依赖 `flash-core` 或任何宿主/IM 模块。
//! 约束:核心类型 `StorageObject` 不含 owner;owner/配额/引用经端口与 `Scope` 透传。

pub mod config;
pub mod direct;
pub mod ports;
pub mod service;

pub use config::StorageServiceConfig;
pub use direct::{
    ConfirmRequest, ConfirmResult, DirectUploadIssuer, DirectUploadService, IssueGrantRequest,
    StsCredentials, UploadGrant,
};
pub use ports::{
    FileObjectStore, ImageFit, ImageMetadata, ImageOutputFormat, ImageTransform, MetadataExtractor,
    NewStorageObject, QuotaPolicy, TransformedImage, VideoMeta,
};
pub use service::{
    DeleteResult, FileContent, FileInfo, FileListResult, StorageService, UploadResult,
};

pub use fx_storage_core::{QuotaSnapshot, Scope, StorageBackend, StorageError, StorageObject};
