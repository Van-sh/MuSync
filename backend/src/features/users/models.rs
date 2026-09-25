use anyhow::{Result, anyhow};
use chrono::NaiveDateTime;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    env::Env,
    error::{ApiError, ApiResult},
};

#[derive(Deserialize)]
pub struct RegisterBody {
    pub username: String,
    pub password: String,
    #[serde(rename = "camelCase")]
    pub confirm_password: String,
}

#[derive(Deserialize)]
pub struct LoginBody {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct AccessTokenClaims {
    pub sub: Uuid,
    pub username: String,
    #[serde(with = "jwt_numeric_date")]
    pub exp: NaiveDateTime,
}

impl AccessTokenClaims {
    pub fn encode_jwt(&self) -> Result<String> {
        let env = Env::get();

        jsonwebtoken::encode(
            &Header::default(),
            self,
            &EncodingKey::from_secret(env.jwt_secret.as_bytes()),
        )
        .map_err(|_| anyhow!("Failed to create access token"))
    }
}

#[derive(Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    pub sub: Uuid,
    pub jti: Uuid,
    #[serde(with = "jwt_numeric_date")]
    pub exp: NaiveDateTime,
}

impl RefreshTokenClaims {
    pub fn encode_jwt(&self) -> Result<String> {
        let env = Env::get();

        jsonwebtoken::encode(
            &Header::default(),
            self,
            &EncodingKey::from_secret(env.jwt_secret.as_bytes()),
        )
        .map_err(|_| anyhow!("Failed to create refresh token"))
    }

    pub fn decode_jwt(token: &[u8]) -> ApiResult<Self> {
        let env = Env::get();
        let decode = jsonwebtoken::decode::<Self>(
            token,
            &DecodingKey::from_secret(env.jwt_secret.as_bytes()),
            &Validation::default(),
        );
        match decode {
            Ok(data) => Ok(data.claims),
            Err(error) => {
                if error.kind() == &jsonwebtoken::errors::ErrorKind::ExpiredSignature {
                    Err(ApiError::Unauthorized)
                } else {
                    Err(anyhow!(error).into())
                }
            }
        }
    }
}

impl From<entity::refresh_tokens::Model> for RefreshTokenClaims {
    fn from(value: entity::refresh_tokens::Model) -> Self {
        Self {
            sub: value.user_id,
            jti: value.id,
            exp: value.expires_at,
        }
    }
}

mod jwt_numeric_date {
    use chrono::{DateTime, NaiveDateTime};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(date: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let timestamp = date.and_utc().timestamp();
        serializer.serialize_i64(timestamp)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let time = DateTime::from_timestamp(i64::deserialize(deserializer)?, 0)
            .ok_or_else(|| serde::de::Error::custom("invalid Unix timestamp value"));

        time.map(|t| t.naive_utc())
    }
}
