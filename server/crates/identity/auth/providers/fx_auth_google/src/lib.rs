//! 验证 Google OpenID Connect ID Token，并输出统一外部身份。

mod model;

use std::time::Duration;

use async_trait::async_trait;
use fx_auth_core::{AuthContext, AuthIdentity, AuthInput, AuthProvider};
use fx_core::{AppError, AppResult};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use reqwest::Client;

use model::{GoogleClaims, GoogleJwks};

const GOOGLE_JWKS_URL: &str = "https://www.googleapis.com/oauth2/v3/certs";

pub struct GoogleAuthProvider {
    client: Client,
    audiences: Vec<String>,
}

impl GoogleAuthProvider {
    pub fn new(audiences: Vec<String>) -> Self {
        Self::with_client(
            audiences,
            Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("failed to create google auth client"),
        )
    }

    pub fn with_client(audiences: Vec<String>, client: Client) -> Self {
        Self { client, audiences }
    }
}

#[async_trait]
impl AuthProvider for GoogleAuthProvider {
    fn kind(&self) -> &'static str {
        "google"
    }

    async fn authenticate(&self, input: AuthInput, _ctx: AuthContext) -> AppResult<AuthIdentity> {
        if self.audiences.is_empty() {
            return Err(AppError::internal(
                "Google audience is empty",
                "google auth config",
            ));
        }
        let header =
            jsonwebtoken::decode_header(&input.credential).map_err(|_| invalid_google_token())?;
        let kid = header.kid.ok_or_else(invalid_google_token)?;
        let keys = self
            .client
            .get(GOOGLE_JWKS_URL)
            .send()
            .await
            .map_err(|error| AppError::internal(error, "google keys"))?
            .error_for_status()
            .map_err(|error| AppError::internal(error, "google keys"))?
            .json::<GoogleJwks>()
            .await
            .map_err(|error| AppError::internal(error, "google keys"))?;
        let key = keys
            .keys
            .iter()
            .find(|key| key.kid == kid)
            .ok_or_else(invalid_google_token)?;
        let decoding_key = DecodingKey::from_rsa_components(&key.n, &key.e)
            .map_err(|error| AppError::internal(error, "google key"))?;
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&["accounts.google.com", "https://accounts.google.com"]);
        validation.set_audience(&self.audiences);
        let claims = decode::<GoogleClaims>(&input.credential, &decoding_key, &validation)
            .map_err(|_| invalid_google_token())?
            .claims;
        let display_name = claims.display_name();
        Ok(AuthIdentity {
            auth_type: self.kind().to_owned(),
            identifier: claims.sub,
            display_name: Some(display_name),
            avatar: claims.picture,
        })
    }
}

fn invalid_google_token() -> AppError {
    AppError::unauthorized_code("AUTH_GOOGLE_TOKEN_INVALID", "Google 授权验证失败")
}
