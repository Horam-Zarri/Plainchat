use std::{borrow::{Borrow, BorrowMut}, str::FromStr};

use axum::{extract::{Request, State}, middleware::{self, Next}, response::Response, routing::{delete, post}, Json, Router};
use axum::extract::Path;
use axum::routing::get;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{PgPool, Row};
use tokio::task::futures;
use tower_http::follow_redirect::policy::PolicyExt;
use tracing::{event, info, trace, warn, Level};
use uuid::Uuid;

use crate::{auth_extractor::AuthContext, models::MessageType, util::{redis_store, sqlx_ext::SqlxConstraints}, AppState};
use crate::error::{AppError, Result};
use crate::models::{MessageModel, GroupModel, UserGroupModel, UserModel, UserRole};
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(user_groups).post(create_group))
        .route("/:group_id", delete(delete_group))
        .route("/:group_id/members", get(list_members))
        .route("/:group_id/messages", get(list_messages))
        //.route("/api/groups/:id/messages", get(list_group_messages))
}


async fn create_group(
    State(state): State<AppState>,
    AuthContext(user_id): AuthContext,
    Json(payload): Json<GroupPayload>
) -> Result<Json<Value>> {
    // multiple query better with explicit acquire i guess idk
    let mut conn = state.db.acquire().await?;

    let group_id = sqlx::query!(r#"
        INSERT INTO groups(name) 
        VALUES ($1) 
        RETURNING id
    "#, payload.name)
       .fetch_one(conn.as_mut()).await?;

    sqlx::query!(r#"
        INSERT INTO user_groups(user_id, group_id, role)
        VALUES ($1, $2, 'admin')
    "#, user_id, group_id.id).execute(conn.as_mut()).await?;
    Ok(Json(Value::String(payload.name)))
}

async fn user_groups(
    State(state): State<AppState>,
    AuthContext(user_id): AuthContext,
) -> Result<Json<Vec<Group>>> {

    if !user_exists(&state.db, user_id).await? {
        return Err(AppError::InvalidToken)
    }

    let groups = sqlx::query_as!(Group,
    r#"
        SELECT 
            groups.id, 
            name
        FROM (
            SELECT * FROM user_groups WHERE user_id = $1
        ) AS gs
        INNER JOIN groups
        ON gs.group_id = groups.id
    "#, user_id).fetch_all(&state.db).await?;

    Ok(Json(groups))
}
async fn delete_group(
    State(state): State<AppState>,
    AuthContext(user_id): AuthContext,
    Path(group_id): Path<String>
) -> Result<Json<Value>> {

    let group_id = Uuid::from_str(&group_id).expect("Id in path should be convertible to Uuid");

    if !user_in_group(
        &state.db, 
        user_id, 
        group_id, 
        Some(UserRole::Admin))
        .await?
    {
        return Err(AppError::ForbiddenAction);
    }

    let group = sqlx::query_as!(GroupModel, "SELECT * FROM groups WHERE id = $1", group_id)
            .fetch_one(&state.db)
            .await
            .map_non_existence_err("Group", "")?;

    sqlx::query!("DELETE FROM user_groups WHERE group_id = $1", group_id).execute(&state.db).await?;
    sqlx::query!("DELETE FROM messages WHERE receiver_group_id = $1", group_id).execute(&state.db).await?;
    sqlx::query!("DELETE FROM groups WHERE id = $1", group_id).execute(&state.db).await?;
 
    Ok(Json(Value::String(group.name)))
}

async fn list_messages(
    State(mut state): State<AppState>,
    AuthContext(user_id): AuthContext,
    Path(group_id): Path<String>
) -> Result<Json<Vec<Message>>> {
    event!(Level::TRACE, "LISTING MSGS!");

    let group_id = Uuid::parse_str(&group_id)
        .expect("Group ID in path should be convertible to Uuid");

    if !user_in_group(
        &state.db, 
        user_id, 
        group_id,
        None)
        .await?
    {
        return Err(AppError::ForbiddenAction);
    }

    let cached_msgs = redis_store::get_messages(
        &mut state.redis, 
        group_id)
        .await;

    if !cached_msgs.is_empty() {
        trace!("CACHED MSG ARE SENT!");
        Ok(Json(cached_msgs))
    } else {
        let messages = sqlx::query_as!(Message, 
        r#"
            SELECT 
                msgs.id, 
                COALESCE(username, '') AS sender,
                content, 
                msgs.msg_type AS "msg_type: MessageType",
                created_at AS date
            FROM(
                    SELECT *
                    FROM messages 
                    WHERE receiver_group_id = $1
            ) AS msgs
            LEFT JOIN users
            ON users.id = msgs.sender_id
            ORDER BY created_at
        "#, group_id)
            .fetch_all(&state.db)
            .await?;

        redis_store::cache_messages(
            &mut state.redis, 
            group_id, 
            &messages)
            .await?;
        
        Ok(Json(messages))
    }
}

async fn list_members(
    State(state): State<AppState>,
    AuthContext(user_id): AuthContext, 
    Path(group_id): Path<String>
) -> Result<Json<Vec<Member>>> {
    event!(Level::INFO, "LIST GROUP INFO!");

    if !user_in_group(
        &state.db, 
        user_id, 
        Uuid::parse_str(&group_id).unwrap(),
        None)
        .await?
    {
        return Err(AppError::ForbiddenAction);
    }

    let members = sqlx::query!(
    r#"
        SELECT
            users.username,
            gs.role AS "role: UserRole"
        FROM( 
            SELECT * 
            FROM user_groups 
            WHERE group_id = $1
        ) AS gs
        INNER JOIN users
        ON users.id = gs.user_id
        INNER JOIN groups
        ON groups.id = gs.group_id
    "#, group_id.parse::<Uuid>()
            .expect("Group ID should be convertible to Uuid"))
        .fetch_all(&state.db)
    .await?;



    let mut redis_conn = state.redis.clone();

    let mut mem_vec = vec![];
    mem_vec.reserve(members.len());

    for m in members.into_iter() {
        let presense = redis_store::is_online(&mut redis_conn, &m.username).await;
        mem_vec.push(Member {
            username: m.username,
            role: m.role,
            presence: presense
        })
    }

    info!("MEMBERS: {mem_vec:?}");

    Ok(Json(mem_vec))
}


async fn user_exists(
    conn: &PgPool,
    user_id: Uuid
) -> Result<bool> {
    Ok(sqlx::query!(
        r#"
            SELECT EXISTS (
                SELECT 1 
                FROM users 
                WHERE 
                    id = $1
            ) AS "exists!"
        "#, user_id)
            .fetch_one(conn)
            .await?
            .exists
    )
}
async fn user_in_group(
    conn: &PgPool, 
    user_id: Uuid,
    group_id: Uuid,
    role: Option<UserRole>
) -> Result<bool> {
    Ok(if let Some(r) = role {
        sqlx::query!(r#"
            SELECT EXISTS (
                SELECT 1
                FROM user_groups
                WHERE
                    group_id = $1 AND 
                    user_id = $2 AND 
                    role = $3
            ) AS "exists!"
        "#, group_id, user_id, r.to_string() as _)
            .fetch_one(conn)
            .await?
            .exists
    } else {
        sqlx::query!(
        r#"
            SELECT EXISTS (
                SELECT 1 FROM user_groups
                WHERE
                    group_id = $1 AND 
                    user_id = $2 
            ) AS "exists!"
        "#, group_id, user_id)
            .fetch_one(conn)
            .await?
            .exists
    })


}
//async fn mw_require_group_member(
//auth_ctx: AuthContext,
//) {

//}
// async fn list_group_messages(
//     State(state): State<AppState>,
//     auth_ctx: AuthContext,
//     Path(id): Path<String>
// ) -> Result<Json<Message>> {
//
//     // check if group not exist
//     let group = sqlx::query_as!(GroupModel, "SELECT * FROM groups WHERE id = $1",
//         id.parse::<i32>().expect("Id in path should be convertible to i32"))
//         .fetch_one(&state.db).await?;
//
//     //check if user doesnt belong to group
//     //let user_group = sqlx::query_as!(UserGroupModel, "SELECT * FROM user_groups WHERE ")
//
//     let x = sqlx::query!("SELECT EXISTS(SELECT 1 FROM user_groups WHERE user_id = $1 AND group_id = $2)",
//         auth_ctx.user_id,
//         id.parse::<i32>().expect("Id in path should be convertible to i32"))
//         .fetch_one(&state.db).await?;
//
//     //let messages = sqlx::query_as!(MessageModel, "SELECT *")
//     //Ok(Json(Message {
//
//     //}))
// }
#[derive(Deserialize)]
struct GroupPayload {
    name: String
}

#[derive(Deserialize)]
struct GroupUserPayload {
    username: String
}

#[derive(Serialize, sqlx::Type, Deserialize, Debug, Clone)]
pub struct Message {
    pub id: Uuid,
    pub sender: Option<String>,
    pub content: String,
    pub msg_type: MessageType,
    pub date: chrono::NaiveDateTime
}


#[derive(Serialize, Debug, sqlx::Type)]
struct Member {
    username: String,
    role: UserRole,
    presence: bool,
}
#[derive(Serialize)]
struct Group {
    id: Uuid,
    name: String,
}


