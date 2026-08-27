//! Google OpenID Connect 公钥与 ID Token 内部模型。

use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub(super) struct GoogleJwks {
    pub(super) keys: Vec<GoogleJwk>,
}

#[derive(Clone, Deserialize)]
pub(super) struct GoogleJwk {
    pub(super) kid: String,
    pub(super) n: String,
    pub(super) e: String,
}

#[derive(Clone, Deserialize)]
pub(super) struct GoogleClaims {
    pub(super) sub: String,
    pub(super) email: Option<String>,
    #[serde(default)]
    pub(super) email_verified: bool,
    pub(super) name: Option<String>,
    pub(super) picture: Option<String>,
}

impl GoogleClaims {
    pub(super) fn display_name(&self) -> String {
        self.name
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .map(str::to_owned)
            .or_else(|| {
                self.email_verified
                    .then(|| self.email.as_deref())
                    .flatten()
                    .and_then(|email| email.split('@').next())
                    .map(str::to_owned)
            })
            .unwrap_or_else(|| "Google 用户".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::GoogleClaims;

    #[test]
    fn verified_email_can_supply_display_name() {
        let claims: GoogleClaims = serde_json::from_value(serde_json::json!({
            "sub": "google-subject",
            "email": "toly@example.com",
            "email_verified": true
        }))
        .unwrap();

        assert_eq!(claims.display_name(), "toly");
    }

    #[test]
    fn unverified_email_is_not_used_as_profile_name() {
        let claims: GoogleClaims = serde_json::from_value(serde_json::json!({
            "sub": "google-subject",
            "email": "unknown@example.com",
            "email_verified": false
        }))
        .unwrap();

        assert_eq!(claims.display_name(), "Google 用户");
    }
}
