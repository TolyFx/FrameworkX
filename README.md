# FrameworkX

FrameworkX 是 FullStackX 体系的公共 SDK 层，为 ViewX 及后续产品提供可复用、可版本化、与具体业务无关的基础能力。

## 定位

FrameworkX 负责：

- 稳定的领域模型与公开契约；
- 认证、用户、网络、存储等通用能力；
- Flutter/Dart 客户端 SDK；
- Rust 服务端库与框架适配器；
- 跨端协议、错误码和兼容性约定。

FrameworkX 不负责：

- ViewX 的画板、节点、资产等产品业务；
- 具体应用页面、路由和状态容器；
- 宿主环境变量、密钥和部署配置；
- 为单一业务定制的数据初始化流程。

## 目录

```text
FrameworkX/
├── client/packages/       # Flutter/Dart SDK 包
├── server/crates/         # Rust 核心库及服务端适配器
├── contracts/             # 跨端 API、事件、错误码契约
└── docs/architecture/     # SDK 边界和演进决策
```

当前处于首批 SDK 迁移阶段。认证 Provider、存储核心、Flutter 网络/异常、用户会话与账号 UI 已建立独立 Workspace 基线；用户服务端适配器和客户端云存储队列仍待迁移。具体权威来源以 [迁移状态](docs/features/sdk-foundation/v0.1.0/migration-status.md) 为准。
