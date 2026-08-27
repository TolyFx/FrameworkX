# fx_auth_wechat

微信小程序认证 Provider。输入为小程序 `wx.login` 返回的临时 `code`，Provider 调用微信
`jscode2session`，验证成功后输出统一身份：

```text
auth_type = wechat
identifier = openid
```

`session_key` 和 `unionid` 不进入领域模型，也不会被持久化。账号创建、已有凭据查询、资料初始化和
业务 Token 签发仍由 `fx_user_service` 处理。

## 宿主接入

```rust
use std::sync::Arc;

use fx_auth_wechat::WeChatAuthProvider;

let provider = Arc::new(WeChatAuthProvider::new(
    std::env::var("WECHAT_APPID")?,
    std::env::var("WECHAT_SECRET")?,
));

let users = users.with_auth_provider(provider);
```

统一登录请求中：

```json
{
  "type": "wechat",
  "identifier": "可选的用户显示名",
  "credential": "wx.login 返回的 code"
}
```

配置是否启用由宿主决定；Provider 本身不读取环境变量，也不依赖 Axum、SQLx 或具体产品代码。
