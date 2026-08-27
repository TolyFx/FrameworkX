# Rust SDK Crates

此目录存放可被多个 Rust 服务复用的库，并按 `foundation`、`identity`、`storage` 领域归档。

建议依赖层次：

```text
fx_auth_* providers -> fx_auth_core
fx_user_axum        -> fx_user_service -> fx_user_core
fx_user_postgres    ------------------> fx_user_core
fx_storage_axum     -> fx_storage_service -> fx_storage_core
fx_storage_postgres -> fx_storage_service -> fx_storage_core
fx_storage_router   -> fx_storage_local + fx_storage_oss -> fx_storage_core
```

核心 crate 不依赖 Axum 或 SQLx；所有 PostgreSQL adapter 共享的基础迁移统一由
`server/migrations/postgres` 管理，消费方只聚合该稳定入口与自身产品迁移。
