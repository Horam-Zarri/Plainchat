use std::fmt::Display;

use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
#[derive(Debug, Serialize)]
pub struct UserModel {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
}

#[derive(Debug, Clone, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "message_type", rename_all = "lowercase")]
pub enum MessageType {
    Normal,
    Event
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageModel {
    pub id: Uuid,
    pub sender_id: Option<i32>,
    pub receiver_group_id: i32,
    pub content: String,
    pub msg_type: MessageType,
    pub created_at: chrono::NaiveDateTime
}

#[derive(Debug,Serialize)]
pub struct GroupModel {
    pub id: Uuid,
    pub name: String
}

#[derive(Debug, Clone, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    User,
    Admin,
}
#[derive(Debug, Serialize)]
pub struct UserGroupModel {
    pub user_id: Uuid,
    pub group_id: Uuid,
    pub role: UserRole
}


impl Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::User => write!(f, "user"),
            UserRole::Admin => write!(f, "admin"),
        }
    }
}

impl Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageType::Normal => write!(f, "normal"),
            MessageType::Event => write!(f, "event"),
        }
    }
}
