use std::fmt::Display;
use std::ops::Mul;
use std::str::FromStr;

use anyhow::Context;
use redis::{aio::MultiplexedConnection, AsyncCommands};
use tracing::warn;
use uuid::Uuid;

use crate::auth_extractor::TOKEN_EXPIRE_SECS;
use crate::error::{AppError, Result};
use crate::routes::group::Message;

const REDIS_UTOKEN_KEY_BASE: &'static str = "user-token";
const REDIS_USTATUS_KEY_BASE: &'static str = "user-presence";
const REDIS_MSG_KEY_BASE:&'static str = "msgs";

// 5 Days for msg cache 
const REDIS_MSGS_EXPIRE: u64 = 3600 * 24 * 5;

fn redis_token_key(token: &str) -> String {
    format!("{REDIS_UTOKEN_KEY_BASE}:{token}")
}
fn redis_status_key(username: &str) -> String {
    format!("{REDIS_USTATUS_KEY_BASE}:{username}")
}
fn redis_msg_list_key(group_id: &str) -> String {
    format!("{REDIS_MSG_KEY_BASE}:{group_id}")
}


pub async fn store_token(
    conn: &mut MultiplexedConnection, 
    token: &str, 
    user_id: Uuid
) -> Result<()> {
    conn.set_ex(
        redis_token_key(token),
        user_id.to_string(),
        TOKEN_EXPIRE_SECS.checked_sub(3600).unwrap())
        .await?;
    Ok(())
}

pub async fn get_token(
    conn: &mut MultiplexedConnection,
    token: &str
) -> Option<Uuid> {
    conn.get::<'_, _, Option<String>>(redis_token_key(token))
        .await
        .ok()
        .flatten()
        .map(|ref id| Uuid::parse_str(id)
            .expect("Redis Token ID should be convertible back to Uuid"))
}

pub async fn set_online(
    conn: &mut MultiplexedConnection,
    username: &str,
) -> Result<()> {
    conn.set(redis_status_key(username), "ON")
        .await?;
    Ok(())
}

pub async fn set_offline(
    conn: &mut MultiplexedConnection,
    username: &str
) {
    let _ = conn.del::<'_, _, ()>
        (redis_status_key(username)).await;
}

pub async fn is_online(
    conn: &mut MultiplexedConnection, 
    username: &str
) -> bool {
    conn.get::<'_, _, String>(redis_status_key(username)).await
        .is_ok()
}

pub async fn cache_messages(
    conn: &mut MultiplexedConnection, 
    group_id: Uuid, 
    msgs: &Vec<Message>
) -> Result<()> {
    let list = redis_msg_list_key(&group_id.to_string());
    if conn.exists(&list).await? {
        return Ok(());
    }
    for msg in msgs {
        let json_str = serde_json::ser::to_string(msg).map_err(|e| anyhow::anyhow!(e))?;
        conn.lpush(&list, json_str)
            .await?;
    }

    conn.expire(list, REDIS_MSGS_EXPIRE as i64)
        .await?;

    Ok(())
}

pub async fn append_message(
    conn: &mut MultiplexedConnection,
    group_id: Uuid,
    msg: Message
) -> Result<()> {
    let list = redis_msg_list_key(&group_id.to_string());
    let msg_str = serde_json::ser::to_string(&msg).map_err(|e| anyhow::anyhow!(e))?;
    conn.lpush(list, msg_str)
        .await?;

    Ok(())
}
pub async fn get_messages(
    conn: &mut MultiplexedConnection,
    group_id: Uuid
) -> Vec<Message> {
    let list = redis_msg_list_key(&group_id.to_string());
    conn.lrange::<'_, _, Vec<String>>(list, 0, -1)
        .await
        //.map_err(|e| AppError::Redis(e))
        .map(|msgs_str| 
            msgs_str.iter()
                .rev()
                .map(String::as_str)
                .map(serde_json::de::from_str::<'_, Message>)
                .map(|res| res.map_err(|e| anyhow::anyhow!(e)).unwrap())
                .collect::<Vec<Message>>()
        ).unwrap_or(vec![])

}
