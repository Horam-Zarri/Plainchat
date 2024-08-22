use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thiserror::Error;
use tracing::{event, warn, Level};

pub type Result<T, E = AppError> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("error occurred with database")]
    Sqlx(#[from] sqlx::Error),

    #[error("error occurred with redis")]
    Redis(#[from] redis::RedisError), 

    #[error("wrong credentials provided")]
    WrongCredentials(Option<String>),

    #[error("{target_type} {data} already exists.")]
    AlreadyExists {
        target_type: String,
        data: String
    },

    #[error("{target_type} {data} does not exist.")]
    DoesNotExist {
        target_type: String,
        data: String
    },

    #[error("user is not eligible to access this method")]
    ForbiddenAction,

    #[error("missing jwt token")]
    MissingToken,

    #[error("invalid jwt token")]
    InvalidToken,

    #[error("anyhow error no explain")]
    Anyhow(#[from] anyhow::Error)
}


impl IntoResponse for AppError {
    fn into_response(self) -> Response {

        warn!("CUSTOM ERR EXECUTING...");

        let err: (StatusCode, Json<Value>) = match &self {
            AppError::WrongCredentials(desc) =>
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({
                        "error": {
                            "msg": desc.to_owned().unwrap_or(self.to_string())
                        }
                    }))
                ),
            AppError::DoesNotExist {
                target_type: _, 
                data: _
            } =>
                (
                    StatusCode::NOT_FOUND,
                    Json(json!({
                        "error": {
                            "msg": self.to_string()
                        }
                    }))
                ),
            AppError::AlreadyExists {
                target_type: _, 
                data: _
            } => (
                StatusCode::CONFLICT,
                Json(json!({
                    "error": {
                        "msg": self.to_string()
                    }
                }))
            ),
            AppError::ForbiddenAction =>
                (
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "error": {
                            "msg": self.to_string()
                        }
                    }))
                ),
            AppError::MissingToken =>
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({
                        "error": {
                            "msg": self.to_string(),
                            "token": "missing",
                        }
                    }))
                ),
            AppError::InvalidToken =>
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": {
                            "msg": self.to_string(),
                            "token": "invalid"
                        }
                    }))
                ),
            AppError::Sqlx(e) => {
                event!(Level::ERROR, "DB ERROR: {e:?}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": {
                            "msg": "Server encountered some error.",
                            "server": "db",
                        }
                    }))
                )
            },
            AppError::Redis(e) => {
                event!(Level::ERROR, "REDIS ERROR: {e:?}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": {
                            "msg": "Server encountered some error.",
                            "server": "redis",
                        }
            }))
                )
            }
            AppError::Anyhow(e) => {
                tracing::error!("{e:?}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": {
                            "msg": "Server encountered some error.",
                            "server": "unknown",
                        }
                    }))
                )
            }
        };

        err.into_response()
    }
}


