//! 用户能力的服务端便利入口。
//!
//! 领域、应用服务和技术适配仍由独立 crate 维护；本 crate 只聚合公开 API，
//! 便于宿主在不破坏依赖方向的前提下完成装配。

pub use fx_auth_core::{
    AuthContext, AuthIdentity, AuthInput, AuthProvider, VerificationCodeProvider,
    VerificationCodeProviders,
};
pub use fx_user_axum::user_routes;
pub use fx_user_core::{
    Account, AccountIdentifierCheck, AccountStatus, AuthCredential, EmptyUserHooks, LoginResponse,
    ScanSession, TokenProvider, UserContext, UserHooks, UserProfile, UserRepository, UserResponse,
};
pub use fx_user_postgres::PgUserRepository;
pub use fx_user_service::FxUserService;
