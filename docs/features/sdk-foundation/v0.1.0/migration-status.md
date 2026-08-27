---
module: sdk-foundation
version: v0.1.0
date: 2026-08-24
tags: [FrameworkX, migration, identity, storage]
---

# FrameworkX SDK 迁移状态

> 本文只记录迁移事实，不代表 FrameworkX 已发布；ViewX 已开始按领域切换稳定 SDK。

## 已建立独立基线

| 领域 | FrameworkX 目标 | 原始来源 | 当前状态 |
| --- | --- | --- | --- |
| Server Foundation | `server/crates/foundation/fx_core` | `ViewX/server/modules/fx_core` | 已迁移并移除 Axum、SQLx 依赖；FrameworkX 版本为新边界实现。 |
| Server Auth | `server/crates/identity/auth/*` | `ViewX/server/modules/auth/*` | Provider、验证码与测试已迁移；新增 Google ID Token Provider，ViewX 本地副本已删除。 |
| Server User | `server/crates/identity/user/*` | `ViewX/server/modules/fx_user` | 已拆分 core、service、PostgreSQL、Axum 与聚合入口；ViewX 本地副本已删除。 |
| Server Web | `server/crates/foundation/fx_server_web` | `ViewX/server/modules/fx_server_web` | 已建立纯 `fx_core` 到 Axum/SQLx 的技术适配；ViewX 本地副本已删除。 |
| Server Storage | `server/crates/storage/*` | `ViewX/server/modules/storage/*`、`ViewX/server/src/storage/*` | core、service、local、OSS、router、image、STS、PostgreSQL 与 Axum adapter 已迁移。 |
| PostgreSQL Migrations | `server/migrations/postgres` | ViewX 基础迁移 | 用户、扫描与存储基础迁移已集中到 SDK 稳定入口。 |
| Client Foundation | `client/packages/foundation/*` | `ViewX/client/docs/ref/fx/modules/core/*` | `fx_exception`、`fx_dio` 已迁移到 Pub Workspace。 |
| Client Identity Core | `client/packages/identity/fx_user_core` | `fullstack/client/modules/user_auth/fx_user_core` | 已迁移。 |
| Client Identity Session | `client/packages/identity/fx_user_session` | `fullstack/client/modules/user_auth/fx_user_session` | 已迁移；HTTP 实现已从会话包移除。 |
| Client Identity HTTP | `client/packages/identity/adapters/fx_user_http` | 原 `fx_user_session` HTTP 实现 | 已拆为独立 adapter，并移除 `fx_core` 与跨仓库 override。 |
| Client Identity UI | `client/packages/identity/ui/*` | FullStack `fx_user_ui`、ViewX `fx_account` | 已迁移；删除引用旧状态模型的失效 Scope。 |
| Client Resource | `client/packages/storage/fx_resource` | `ViewX/client/packages/fx_resource` | 已迁移到 storage 领域。 |

## 尚未迁移

| 能力 | 原始位置 | 下一步 |
| --- | --- | --- |
| 客户端云存储 core/http/queue | `ViewX/client/modules/business/fn_cloud_storage` | 去掉 `viewId/nodeId/itemId` 和 `fn_*` 依赖后迁移。 |
| ViewX 客户端依赖切换 | `ViewX/client/pubspec.yaml` | 服务端已完成；客户端仍需逐包切换。 |

## 已接入消费方

- ViewX 服务端已将通用存储 core、service、本地后端、OSS、STS 和图片扩展切换到 FrameworkX。
- ViewX 服务端已将 foundation、认证 Provider、用户 core/service/PostgreSQL/Axum 全部切换到 FrameworkX。
- 用户与存储基础迁移统一归属 `server/migrations/postgres`，ViewX 开发与部署脚本负责聚合 SDK 和产品迁移。
- ViewX 仅通过闭包提供 `token → Scope` 映射，并持有画板资产目录与引用策略；通用 HTTP、Bearer 解析、文件账本和配额实现由 FrameworkX 持有。
- ViewX 中原有的通用存储源码副本已移除，FrameworkX 成为该能力的唯一权威来源。

## 权威来源规则

- 尚未切换的认证、用户与客户端能力仍以 ViewX 当前运行路径为生产行为权威来源。
- FrameworkX 中已经迁移的代码用于独立边界重构和验证，不接受在两个仓库分别修复同一问题。
- 新修复先判断是否属于公共能力：公共修复进入 FrameworkX，并在 ViewX 切换任务中回接；ViewX 产品修复继续留在 ViewX。
- 完成原子切换并验证后，原目录删除或改为版本依赖，不保留第二份可编辑副本。

## 已验证

- `cargo check --workspace --offline`：通过。
- `cargo test --workspace --offline`：通过，认证验证码、存储 service、本地后端和图像测试全部成功。
- `flutter pub get --offline`：通过，无跨仓库 path override。
- `flutter analyze`：通过，无问题。
- 各个含测试的 Flutter package：全部通过。
