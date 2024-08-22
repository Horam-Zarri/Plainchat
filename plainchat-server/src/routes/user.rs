use crate::auth_extractor::{AuthContext, TOKEN_EXPIRE_SECS};
use crate::util::redis_store;
use crate::AppState;
use axum::extract::State;
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use jsonwebtoken::{encode, Header};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{Acquire, Executor};
use tracing::{event, info, Level};

use crate::error::{AppError, Result};
use crate::models::UserModel;
use crate::util::{sqlx_ext::SqlxConstraints, pass_hash};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth", post(login_user))
        .route("/", post(create_user).put(update_user).delete(delete_user).get(curr_user))
    //.route("/api/user", get(curr_user).put(update_user))
}

async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<UserPayload>,
) -> Result<Json<Value>> {

    event!(Level::TRACE, "Creating User...");

    let password_hash = pass_hash::hash_password(payload.password).await?;

    sqlx::query!(
        "INSERT INTO users(username, password_hash) VALUES($1, $2)",
        payload.username,
        password_hash
    )
    .execute(&state.db)
    .await
    .map_unique_err("Username", &payload.username)?;

    Ok(Json(Value::Null))
}

async fn login_user(
    State(mut state): State<AppState>,
    Json(payload): Json<UserPayload>,
) -> Result<Json<User>> {
    let user = sqlx::query_as!(
        UserModel,
        "SELECT id, username, password_hash FROM users WHERE username = $1",
        payload.username
    )
    .fetch_one(&state.db)
    .await
    .map_non_existence_err("User", &payload.username)?;

    pass_hash::verify_password(payload.password, user.password_hash).await?;

    let token = AuthContext(user.id).generate_jwt();
    let key = format!("user-token:{}", token);

    // Expire redis key an hour before actual expire
    // just to be sure.
    state.redis.set_ex(
        key, 
        user.id.to_string(), 
        TOKEN_EXPIRE_SECS.checked_sub(3600).unwrap())
        .await?;

    redis_store::store_token(
        &mut state.redis, 
        &token, 
        user.id)
        .await?;

    Ok(Json(User {
        username: payload.username,
        token,
    }))
}

async fn update_user(
    State(state): State<AppState>,
    AuthContext(user_id): AuthContext,
    Json(payload): Json<UserUpdatePayload>,
) -> Result<Json<User>> {

    event!(Level::TRACE, "Updating user with {:?} and {:?}", 
        payload.username, payload.password);

    let password_hash = if let Some(pass) = payload.password {
        if pass.is_empty() {None} else {
            pass_hash::hash_password(pass).await.ok()
        }
    } else { None };

    let user = sqlx::query!(
        r#"
            UPDATE users
            SET username = COALESCE($1, users.username),
                password_hash = COALESCE($2, users.password_hash)
            WHERE id = $3
            RETURNING username
        "#, payload.username, password_hash, user_id
    )
        .fetch_one(&state.db)
        .await?;

    Ok(Json(User {
        username: user.username,
        token: AuthContext(user_id).generate_jwt(),
    }))
}

async fn delete_user(
    State(state): State<AppState>,
    AuthContext(user_id): AuthContext,
) -> Result<Json<Value>> {
    sqlx::query!("DELETE FROM user_groups WHERE user_id = $1", user_id)
        .execute(&state.db)
        .await?;

    let user = sqlx::query!("DELETE FROM users WHERE id = $1 RETURNING username", user_id)
        .fetch_one(&state.db)
        .await?
        .username;

    Ok(Json(Value::String(user)))
}

async fn curr_user(
    State(state): State<AppState>,
    AuthContext(user_id): AuthContext,
) -> Result<Json<Value>> {
    Ok(Json(Value::String(
        sqlx::query!(
            "SELECT (username) FROM users WHERE id = $1",
            user_id
        )
            .fetch_one(&state.db)
            .await
            .map_non_existence_err("User", "")?
            .username
    )))
}



#[derive(Deserialize)]
struct UserPayload {
    username: String,
    password: String,
}

#[derive(Deserialize, Default)]
struct UserUpdatePayload {
    username: Option<String>,
    password: Option<String>,
}

#[derive(Serialize)]
struct User {
    username: String,
    token: String,
}

