# Rust SDK Crates

此目录存放可被多个 Rust 服务复用的库，并按 `foundation`、`identity`、`storage` 领域归档。

建议依赖层次：

```text
fx_auth_* providers -> fx_auth_core
fx_user_axum        -> fx_user_service -> fx_user_core
fx_user_postgres    ------------------> fx_user_core
fx_storage_axum     -> fx_storage_service -> fx_storage_core
fx_storage_postgres ---------------------> fx_storage_core
```

核心 crate 不依赖 Axum 或 SQLx；数据库迁移跟随对应 PostgreSQL adapter 管理。

