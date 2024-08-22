use core::time;
use std::{fmt::Debug, str::FromStr, sync::LazyLock, time::UNIX_EPOCH};

use axum::{async_trait, extract::{FromRef, FromRequestParts}, http::request::Parts, RequestPartsExt};
use axum_extra::{headers::{authorization::Bearer, Authorization}, TypedHeader};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use redis::{aio::MultiplexedConnection, AsyncCommands};
use serde::{Deserialize, Serialize};
use tracing::{event, Level};
use uuid::Uuid;

use crate::{error::AppError, util::redis_store, AppState};
use crate::error::Result;

pub const TOKEN_EXPIRE_SECS: u64 = 7 * 24 * 3600;
struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

static KEYS: LazyLock<Keys> = LazyLock::new(|| {
    let secret = dotenv::var("JWT_SECRET").expect("JWT_SECRET must be set.");
    Keys::new(secret.as_bytes())
});


#[derive(Clone, Debug)]
pub struct AuthContext(pub Uuid);


impl AuthContext {
    pub fn generate_jwt(&self) -> String {
        let now= std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Duration should be convertible to epoch")
            .as_secs();

        let now_plus_week = now.checked_add(TOKEN_EXPIRE_SECS)
            .expect("System curr time should be addable with a week.");


        let claims = TokenClaims {
            sub: self.0.to_string(),
            iat: now as usize,
            exp: now_plus_week as usize
        };

        let token_type = "Bearer";
        let jwt = encode(&Header::default(), &claims, &KEYS.encoding).unwrap();

        format!("{token_type} {jwt}")
    }

    pub fn verify_jwt(token: &str) -> Result<TokenData<TokenClaims>> {
        decode::<TokenClaims>(token, &KEYS.decoding, &Validation::default())
            .map_err(|_| AppError::InvalidToken)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    iat: usize,
    exp: usize,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthContext 
where 
    AppState: FromRef<S>,
    S: Send + Sync
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S
    ) -> 
        ::core::result::Result<Self, Self::Rejection> {
        
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AppError::MissingToken)?;

        let mut state = parts
            .extract_with_state::<AppState, _>(state)
            .await?;

        let uuid_cache = redis_store::get_token(&mut state.redis, bearer.token()).await;

        Ok(AuthContext(
            match uuid_cache {
                Some(uuid) => uuid, 
                None => {
                    let jwt = Self::verify_jwt(bearer.token())?;
                    Uuid::from_str(&jwt.claims.sub).expect("JWT sub should be convertible to Uuid")
                }
            }
        ))
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AppState
where
    Self: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self::from_ref(state)) 
    }
}

