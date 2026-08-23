# fx_user_core

`fx_user_core` 只提供认证与用户领域模型，以及远端仓储契约。

它不包含 HTTP 实现、认证状态管理、凭据持久化或 UI；这些运行时能力由
`fx_user_session` 负责，宿主应用负责注入环境、网络宿主和凭据存储实现。
