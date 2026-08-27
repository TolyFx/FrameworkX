//! 可嵌入宿主服务的管理领域 SDK。
//!
//! 当前以单 crate 的目录边界维护 core、service 与 adapters；公开接口稳定后再按需拆包。

mod model;
mod postgres;
mod repository;
mod routes;
mod service;

pub use model::{
    AdminLoginRecord, AdminUserAccount, AdminUserCredential, AdminUserDetail, AdminUserPage,
    AdminUserProfile, AdminUserQuery, AdminUserSummary,
};
pub use postgres::PgAdminUserRepository;
pub use repository::AdminUserRepository;
pub use routes::{AdminAccess, AdminApiState, admin_routes};
pub use service::AdminUserService;
