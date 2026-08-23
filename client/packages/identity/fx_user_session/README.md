# fx_user_session

`fx_user_session` 是应用认证与会话模块：负责 HTTP 仓储实现、凭据恢复、
认证编排和会话状态发布。跨模块可安全使用的用户信息由 `FxIdentity` 提供：
ID、昵称与头像。

目录按职责划分为：`domain/`（会话状态与身份摘要）、`contract/`（身份投影、
凭据存储和只读访问协议）、`logic/`（认证编排）及 `infrastructure/`（HTTP 仓储）。

运行时会话只有三种状态：`FxAuthing`、`FxGuest` 和 `FxAuthed`。
`FxUserSessionCubit` 是唯一的应用会话状态源。

宿主通过一次 `FxIdentityCodec` 实现，将认证领域用户投影为 `FxIdentity`。
同时由宿主实现 `AuthCredentialStore`；功能模块只依赖本包，读取
`FxUserSession` 或注入只读的 `FxUserSessionSource`。
