//! 定义认证插件共享的输入、上下文、身份与 Provider 协议。
//!
//! 本包保持最小依赖，不包含具体认证方式或宿主用户业务。

mod model;
mod provider;

pub use model::{AuthContext, AuthIdentity, AuthInput};
pub use provider::{AuthProvider, VerificationCodeProvider, VerificationCodeProviders};
