//! 定义微信小程序 `jscode2session` 响应模型。

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct WeChatSessionResponse {
    pub openid: Option<String>,
    pub errcode: Option<i64>,
    pub errmsg: Option<String>,
}

impl WeChatSessionResponse {
    pub(crate) fn into_openid(self) -> Result<String, WeChatSessionError> {
        if let Some(openid) = self.openid.filter(|value| !value.trim().is_empty()) {
            return Ok(openid);
        }
        Err(WeChatSessionError {
            code: self.errcode,
            message: self.errmsg,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct WeChatSessionError {
    pub code: Option<i64>,
    pub message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::WeChatSessionResponse;

    #[test]
    fn extracts_openid_from_success_response() {
        let response: WeChatSessionResponse = serde_json::from_str(
            r#"{"openid":"openid-1","session_key":"secret","unionid":"union-1"}"#,
        )
        .unwrap();

        assert_eq!(response.into_openid().unwrap(), "openid-1");
    }

    #[test]
    fn preserves_wechat_error_for_classification() {
        let response: WeChatSessionResponse =
            serde_json::from_str(r#"{"errcode":40029,"errmsg":"invalid code"}"#).unwrap();

        let error = response.into_openid().unwrap_err();
        assert_eq!(error.code, Some(40029));
        assert_eq!(error.message.as_deref(), Some("invalid code"));
    }
}
