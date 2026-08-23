---
module: sdk-foundation
version: v0.1.0
date: 2026-08-23
tags: [FrameworkX, SDK, identity, authentication, storage]
---

# FrameworkX 首批 SDK 域分析

## 1. 目标

将已经存在第二个消费场景的用户认证与云存储能力，从 ViewX/FullStack 业务仓库收敛到 FrameworkX，形成唯一权威源码与版本化 SDK。

首批包含两个独立域：

- `identity`：认证 Provider、用户账号、会话、账号操作 UI；
- `storage`：文件对象、存储后端、配额、上传下载、可恢复上传。

这两个域可以协作，但不得互相持有对方的具体实现。存储只消费稳定的“已认证主体”契约。

## 2. 当前来源

### 服务端身份域

- `ViewX/server/modules/auth/fx_auth_*`：认证核心与 Apple、GitHub、邮箱、短信、密码 Provider；
- `ViewX/server/modules/fx_user`：用户模型、服务、Repository、PostgreSQL 和 Axum 路由混合包；
- `ViewX/server/src/auth`：JWT、验证码发送配置及宿主组装；
- `ViewX/server/src/default_view.rs`：ViewX 注册/登录后的默认画板业务。

### 客户端身份域

- `fullstack/client/modules/user_auth/fx_user_core`；
- `fullstack/client/modules/user_auth/fx_user_session`；
- `fullstack/client/modules/user_auth/fx_user_ui`；
- `ViewX/client/docs/ref/fx/modules/kit/app/fx_account`；
- `ViewX/client/lib/user_auth`：ViewX 宿主组装和适配器。

### 服务端存储域

- `ViewX/server/modules/storage/fx_storage_core`；
- `fx_storage`、`fx_storage_local`、`fx_storage_oss`、`fx_storage_image`、`fx_storage_sts`；
- `ViewX/server/src/storage/pg.rs`：PostgreSQL 文件账本和配额实现；
- `ViewX/server/src/storage/routes.rs`：Axum API，同时包含 ViewX 资产关联查询；
- `ViewX/server/migrations`：用户、文件对象、配额和 ViewX 引用关系迁移。

### 客户端存储域

- `ViewX/client/packages/fx_resource`：本地文件引用、选择与存储；
- `ViewX/client/modules/business/fn_cloud_storage`：领域对象、HTTP、上传队列、SQLite 和 ViewX 云资产 UI 混合模块。

## 3. 边界判断

| 能力 | FrameworkX | ViewX | 理由 |
| --- | --- | --- | --- |
| 认证 Provider 与验证码场景 | 是 | 组装配置 | 与产品业务无关，可多服务复用。 |
| 用户模型、账号用例、会话 | 是 | 消费 | 已存在服务端和客户端共同契约。 |
| JWT 具体密钥和部署配置 | 否 | 是 | 属于宿主安全与部署责任。SDK 只定义 Token 端口和可选实现。 |
| 登录/账号通用 UI | 是 | 主题与提交回调 | UI 不依赖宿主路由和业务状态时可以复用。 |
| 登录后创建默认画板 | 否 | 是 | 是 ViewX 产品初始化，不属于身份域。 |
| 文件存储后端、文件对象、配额 | 是 | 消费 | 是通用基础设施。 |
| 本地资源引用和文件选择抽象 | 是 | 注入平台能力 | 与 ViewX 节点模型无关。 |
| 可恢复上传队列 | 是，需去业务定位字段 | 回调文档更新 | 队列机制通用，但当前任务含 `viewId/nodeId/itemId`。 |
| 云资产列表/详情通用模型 | 是 | 可组合展示 | 文件列表、分类、大小、状态属于云盘域。 |
| “被哪些 ViewX 画板引用” | 否 | 是 | 直接依赖 `views` 和 `view_asset_refs`。 |
| ViewX 云资产页面 | 否 | 是 | 页面、文案、导航和画板跳转属于产品体验。 |

## 4. 主要耦合点

1. `fx_user` 同时依赖 Axum 与 SQLx，核心服务无法独立消费。
2. ViewX 宿主中的 `PgPasswordStore` 直接知道公共认证表结构。
3. `UserHooks` 同步执行默认画板创建，画板异常可能阻断登录。
4. `fx_storage` 核心较干净，但 PostgreSQL 适配器仍留在 ViewX 宿主。
5. 存储 Axum 路由既做通用文件 API，又直接查询 ViewX 画板引用。
6. 客户端 `fn_cloud_storage` 同时包含通用上传队列和 ViewX 专属界面。
7. `AssetUploadTask` 使用 ViewX 的 `viewId/nodeId/itemId` 定位，无法成为通用 SDK 模型。
8. Flutter 身份包跨 ViewX 与 FullStack 两个仓库路径引用，没有唯一发布源。

## 5. 结论

身份域和存储域都适合进入 FrameworkX，但不能整目录复制。迁移必须按“核心契约 → 应用服务 → 技术适配器 → 宿主组装”重新归位。

FrameworkX 是唯一权威源码；迁移完成后 ViewX 只能通过固定版本依赖，不保留可被同时修改的镜像副本。

