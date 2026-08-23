# FrameworkX SDK 边界

## 依赖方向

```text
ViewX / Other Products
        |
        v
FrameworkX adapters
        |
        v
FrameworkX application services
        |
        v
FrameworkX core contracts
```

核心契约不知道宿主、HTTP 框架、数据库和 UI。适配器实现核心端口，宿主只负责组装配置和产品业务回调。

## 初始候选域

| 域 | 核心能力 | 可选适配器 |
| --- | --- | --- |
| auth | 认证输入、身份结果、Provider 接口 | Apple、GitHub、邮箱、短信、密码 |
| user | 用户模型、Repository、Session、账号用例 | HTTP、PostgreSQL、本地缓存 |
| network | Host、认证凭据、错误转换 | Dio |
| storage | 文件引用、上传下载端口 | HTTP、对象存储、本地文件 |

候选域不代表已经迁移或发布。每个域需要单独完成依赖审计后才能成为正式 SDK。

## 宿主保留职责

- 路由与页面导航；
- 产品 UI 与交互；
- 环境配置和密钥读取；
- JWT、邮件、短信等部署级配置；
- 注册后创建默认画板等产品初始化；
- 业务数据与业务事件消费。

## 第一阶段迁移顺序

1. 确定 FrameworkX 为公共代码唯一权威来源。
2. 定义 auth/user 的跨端契约和稳定错误码。
3. 迁移无宿主依赖的核心接口及测试。
4. 分离 Flutter HTTP/缓存适配器与 Rust Axum/PostgreSQL 适配器。
5. ViewX 改为版本化依赖并保留宿主组装层。
6. 使用第二个最小宿主验证 SDK 不包含 ViewX 隐式假设。

