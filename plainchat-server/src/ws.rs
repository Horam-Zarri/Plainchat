use std::str::FromStr;

use anyhow::Context;
use axum::{http::header::AUTHORIZATION, RequestPartsExt};
use axum_extra::{headers::{authorization::Bearer, Authorization}, TypedHeader};
use redis::AsyncCommands;
use serde::Serialize;
use socketioxide::{extract::{Data, Extension, SocketRef, State}, handler::ConnectHandler, layer::SocketIoLayer, SocketIo, SocketIoBuilder};
use tracing::{error, event, info, Level};
use uuid::Uuid;
use crate::{auth_extractor::AuthContext, error::AppError, models::{MessageType, UserModel}, routes::group::Message, util::{redis_store, sqlx_ext::SqlxConstraints}, AppState};
const BEARER_PREFIX: &'static str = "Bearer";


pub fn layer(state: AppState) -> SocketIoLayer {
    let (layer, io)= SocketIo::builder().with_state(state).build_layer();

    io.ns("/", on_connection.with(auth_mw));
    layer 
}

async fn auth_mw(s: SocketRef, State(mut state): State<AppState>) -> crate::error::Result<()> {

    event!(Level::TRACE, "SOCKET PASSING THROUGH MW");
    let auth_header =  s.req_parts()
        .headers
        .get(AUTHORIZATION)
        .ok_or(AppError::MissingToken)?;

    let auth_token= auth_header.to_str().
        context("Auth header could not be converted to str")?;

    if !auth_token.starts_with(BEARER_PREFIX) {
        event!(Level::TRACE, "Unable to decode JWT");
        Err(AppError::InvalidToken)
    } else {
        let token = &auth_token[(BEARER_PREFIX.len() + 1)..];
        let user_id= AuthContext::verify_jwt(token)?
            .claims.sub.parse::<Uuid>().expect("User id should be convertible to Uuid");
    
        let user_context = sqlx::query_as!(UserContext,
            "SELECT id, username FROM users WHERE id = $1",
             user_id)
            .fetch_one(&state.db)
            .await?;

        event!(Level::TRACE, "Decoded JWT {user_context:?}");
        s.extensions.insert(user_context.clone());

        let _ = redis_store::set_online(
            &mut state.redis , 
            &user_context.username)
            .await;

        Ok(())
    }
}

async fn on_connection(
    socket: SocketRef,
) {

    event!(Level::TRACE, "Socket connected: {}", socket.id);

    socket.on_disconnect(
        |s: SocketRef, 
        Extension(user_ctx): Extension<UserContext>,
        State(mut state): State<AppState>| 
        async move {
            info!("SOCKET DC ON SERVER!!!");
            redis_store::set_offline(&mut state.redis, &user_ctx.username).await;
            let _ = s.broadcast()
                .emit("u_offline", user_ctx.username);
        }); 


    socket.on(
        "join",
        |s: SocketRef,
         Data(group_id): Data<String>, 
         Extension(user_ctx): Extension<UserContext>| 
         async move {
            event!(Level::TRACE, "JOINED ROOM!");
            let _ = s.leave_all();
            let _ = s.join(group_id.clone());
            let _ = s.within(group_id)
                .emit("u_online", user_ctx.username);
        },
    );

    socket.on(
        "message", 
        |s: SocketRef, Data(msg): Data<String>,
         Extension(user_ctx): Extension<UserContext>, State(mut state): State<AppState>|
        async move{
            event!(Level::TRACE, "SIGNALING MSG! {msg}");
            let room = current_room(&s);
            let room_id = Uuid::from_str(&room).unwrap();
            let msg_rec = sqlx::query!(r#"
                INSERT INTO messages(sender_id, receiver_group_id, content, msg_type)
                VALUES 
                    ($1, $2, $3, 'normal')
                RETURNING id, created_at AS date
            "#, user_ctx.id, room_id, msg)
                .fetch_one(&state.db).await.unwrap();
            
            let _ = s.within(room).emit("message", MessageBody {
                id: msg_rec.id,
                sender: user_ctx.username.clone(),
                content: msg.clone(),
                date: msg_rec.date
            });

            let _ = redis_store::append_message(&mut state.redis, room_id, Message {
                id: msg_rec.id,
                sender: Some(user_ctx.username),
                content: msg,
                msg_type: MessageType::Normal,
                date: msg_rec.date,
            }).await;
    });

    socket.on(
        "add_user",
        |s: SocketRef, Data(username): Data<String>,
        Extension(user_ctx) : Extension<UserContext>, State(mut state): State<AppState>|
        async move {
            event!(Level::TRACE, "WS: ADD USER {username}");
            let room = current_room(&s);
            let room_id = Uuid::from_str(&room).unwrap();

            let mut conn = state.db.acquire().await.unwrap();

            let add_res = sqlx::query_as!(UserModel, 
                "SELECT * FROM users WHERE username = $1", 
                    username)
                .fetch_one(conn.as_mut())
                .await
                .map_non_existence_err("User", &username);

            match add_res {
                Ok(add_user) => {
                    sqlx::query!(r#"
                        INSERT INTO user_groups(user_id, group_id, role)
                        VALUES
                            ($1, $2, 'user')
                    "#, add_user.id, room_id)
                        .execute(conn.as_mut())
                        .await
                        .unwrap();

                    let join_msg = format!("{} joined.", add_user.username);

                    let msg_rec = sqlx::query!(r#"
                        INSERT INTO messages(receiver_group_id, content, msg_type)
                        VALUES
                            ($1, $2, 'event')
                        RETURNING id, created_at AS date
                    "#, room_id, join_msg)
                        .fetch_one(conn.as_mut())
                        .await
                        .unwrap();


                    let _ = redis_store::append_message(
                        &mut state.redis, 
                        room_id, 
                        Message {
                            id: msg_rec.id,
                            sender: None,
                            content: join_msg,
                            msg_type: MessageType::Event,
                            date: msg_rec.date,
                    }).await;

                    let is_added_user_online = redis_store::is_online(&mut state.redis, &add_user.username).await;
                    let _ = s.within(room)
                        .emit("add_user", format!("{},{},{is_added_user_online}",add_user.username, user_ctx.username));
                }, 
                Err(e) => {
                    event!(Level::TRACE, "{e}");
                }
            }
        }
    );

    socket.on(
        "leave",
        |s: SocketRef,
        Extension(user_ctx): Extension<UserContext>, 
        State(mut state): State<AppState>|
        async move {

            let room = current_room(&s);
            let room_id = Uuid::from_str(&room).unwrap();

            let _ = s.within(room)
                .emit("leave", user_ctx.username.clone());
            let _ = s.leave_all();
            let _ = sqlx::query!(r#"
                DELETE FROM user_groups 
                WHERE 
                    user_id = $1 AND 
                    group_id = $2
            "#, user_ctx.id, room_id)
                .execute(&state.db)
                .await
                .unwrap();

            let leave_msg = format!("{} left.", user_ctx.username);

            let msg_rec = sqlx::query!(r#"
                INSERT INTO messages(receiver_group_id, content, msg_type)
                    VALUES
                        ($1, $2, 'event')
                    RETURNING id, created_at AS date
            "#, room_id, leave_msg)
                .fetch_one(&state.db)
                .await
                .unwrap();

            let _ = redis_store::append_message(
                &mut state.redis, 
                room_id, 
                Message {
                    id: msg_rec.id,
                    sender: None,
                    content: leave_msg,
                    msg_type: MessageType::Event,
                    date: msg_rec.date,
                })
                .await;
        }
    );

    socket.on(
        "type_start",
        |s: SocketRef, Extension(user_ctx): Extension<UserContext>|
        async move {
            event!(Level::TRACE, "SIGNALING TYPE_START!");
            let room = current_room(&s);
            let _ = s.within(room)
                .emit("type_start", user_ctx.username);
        }
    );

    socket.on(
        "type_stop", 
        |s: SocketRef, Extension(user_ctx): Extension<UserContext>| 
        async move {
            event!(Level::TRACE, "SIGNALING TYPE_STOP!");
            let room = current_room(&s);
            let _ = s.within(room)
                .emit("type_stop", user_ctx.username);
        }
    );

    socket.on(
        "kick", 
        |s: SocketRef, 
        Data(rem_user): Data<String>, 
        State(mut state): State<AppState>,
        Extension(user_ctx): Extension<UserContext>| 
        async move {
            event!(Level::TRACE, "SIGNALING USER kICK");
            let room = current_room(&s);
            let room_id = Uuid::parse_str(&room).expect("Room Id should be convertible to Uuid");
            let res = sqlx::query!(
            r#"
                DELETE FROM user_groups 
                USING users
                WHERE 
                    users.id = user_id AND 
                    users.username = $1 AND 
                    group_id = $2
            "#, rem_user, room_id)
                .execute(&state.db)
                .await;

            match res {
                Ok(_) => {
                    let kicker = user_ctx.username;
                    let _ = s.within(room)
                        .emit("kick", format!("{rem_user},{kicker}"));

                    let kick_msg = format!("{rem_user} was kicked out by {kicker}.");
                    let msg_rec = sqlx::query!(r#"
                        INSERT INTO 
                            messages(receiver_group_id, content, msg_type)
                        VALUES($1, $2, 'event')
                        RETURNING id, created_at AS date
                    "#, room_id, kick_msg)
                        .fetch_one(&state.db)
                        .await
                        .unwrap();

                    let _ = redis_store::append_message(
                        &mut state.redis, 
                        room_id,
                        Message {
                            id: msg_rec.id,
                            sender: None,
                            content: kick_msg,
                            msg_type: MessageType::Event,
                            date: msg_rec.date,
                        }
                    ).await;
                }, 
                Err(e) => error!("Unable to remove user from group: {e}"),
            }
        }
    )
}



fn current_room(s: &SocketRef) -> String {
    s.rooms()
        .expect("A socket should be connected to a group")
        .first()
        .expect("A socket should only be connected to one room")
        .to_string()
}

#[derive(Serialize)]
struct MessageBody {
    id: Uuid,
    sender: String,
    content: String,
    date: chrono::NaiveDateTime
}


#[derive(Debug, Clone)]
struct UserContext {
    id: Uuid,
    username: String,
}
