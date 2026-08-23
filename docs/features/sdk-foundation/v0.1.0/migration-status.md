---
module: sdk-foundation
version: v0.1.0
date: 2026-08-24
tags: [FrameworkX, migration, identity, storage]
---

# FrameworkX SDK 迁移状态

> 本文只记录迁移事实，不代表 FrameworkX 已发布或 ViewX 已切换依赖。

## 已建立独立基线

| 领域 | FrameworkX 目标 | 原始来源 | 当前状态 |
| --- | --- | --- | --- |
| Server Foundation | `server/crates/foundation/fx_core` | `ViewX/server/modules/fx_core` | 已迁移并移除 Axum、SQLx 依赖；FrameworkX 版本为新边界实现。 |
| Server Auth | `server/crates/identity/auth/*` | `ViewX/server/modules/auth/*` | Provider、验证码与测试已迁移。 |
| Server Storage | `server/crates/storage/*` | `ViewX/server/modules/storage/*` | core、service、local、OSS、image、STS 已迁移。 |
| Client Foundation | `client/packages/foundation/*` | `ViewX/client/docs/ref/fx/modules/core/*` | `fx_exception`、`fx_dio` 已迁移到 Pub Workspace。 |
| Client Identity Core | `client/packages/identity/fx_user_core` | `fullstack/client/modules/user_auth/fx_user_core` | 已迁移。 |
| Client Identity Session | `client/packages/identity/fx_user_session` | `fullstack/client/modules/user_auth/fx_user_session` | 已迁移；HTTP 实现已从会话包移除。 |
| Client Identity HTTP | `client/packages/identity/adapters/fx_user_http` | 原 `fx_user_session` HTTP 实现 | 已拆为独立 adapter，并移除 `fx_core` 与跨仓库 override。 |
| Client Identity UI | `client/packages/identity/ui/*` | FullStack `fx_user_ui`、ViewX `fx_account` | 已迁移；删除引用旧状态模型的失效 Scope。 |
| Client Resource | `client/packages/storage/fx_resource` | `ViewX/client/packages/fx_resource` | 已迁移到 storage 领域。 |

## 尚未迁移

| 能力 | 原始位置 | 下一步 |
| --- | --- | --- |
| 用户服务 core/service | `ViewX/server/modules/fx_user` | 拆分领域模型、端口和应用用例。 |
| 用户 PostgreSQL 与 migrations | `ViewX/server/modules/fx_user`、`ViewX/server/migrations` | 建立 `fx_user_postgres`，迁移自有表和密码记录查询。 |
| 用户 Axum adapter | `ViewX/server/modules/fx_user/routes.rs` | 建立传输层错误映射后迁移。 |
| 存储 PostgreSQL adapter | `ViewX/server/src/storage/pg.rs` | 迁移文件账本和配额，不携带 ViewX 引用查询。 |
| 存储 Axum adapter | `ViewX/server/src/storage/routes.rs` | 拆出通用文件 API，ViewX 关联接口留在宿主。 |
| 客户端云存储 core/http/queue | `ViewX/client/modules/business/fn_cloud_storage` | 去掉 `viewId/nodeId/itemId` 和 `fn_*` 依赖后迁移。 |
| ViewX 依赖切换 | `ViewX/client/pubspec.yaml`、`ViewX/server/Cargo.toml` | 全部目标包验证完成后原子切换。 |

## 权威来源规则

- 在 ViewX 尚未切换依赖前，ViewX 当前运行路径仍是生产行为权威来源。
- FrameworkX 中已经迁移的代码用于独立边界重构和验证，不接受在两个仓库分别修复同一问题。
- 新修复先判断是否属于公共能力：公共修复进入 FrameworkX，并在 ViewX 切换任务中回接；ViewX 产品修复继续留在 ViewX。
- 完成原子切换并验证后，原目录删除或改为版本依赖，不保留第二份可编辑副本。

## 已验证

- `cargo check --workspace --offline`：通过。
- `cargo test --workspace --offline`：通过，认证验证码、存储 service、本地后端和图像测试全部成功。
- `flutter pub get --offline`：通过，无跨仓库 path override。
- `flutter analyze`：通过，无问题。
- 各个含测试的 Flutter package：全部通过。

