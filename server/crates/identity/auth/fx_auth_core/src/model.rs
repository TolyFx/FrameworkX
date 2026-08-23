//! 定义认证过程跨模块传递的基础数据模型。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct AuthInput {
    pub kind: String,
    pub identifier: String,
    pub credential: String,
}

#[derive(Debug, Clone, Default)]
pub struct AuthContext {
    pub ip: Option<String>,
    pub platform: Option<String>,
    pub device_name: Option<String>,
    pub device_id: Option<String>,
    pub app_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthIdentity {
    pub auth_type: String,
    pub identifier: String,
    pub display_name: Option<String>,
    pub avatar: Option<String>,
}
