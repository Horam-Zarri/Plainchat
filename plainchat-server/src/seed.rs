use sqlx::PgPool;
use crate::util;
use crate::models::{UserModel, GroupModel};

pub async fn populate_db(db: &PgPool) {

    let p1 = util::pass_hash::hash_password("123456".to_string()).await.unwrap();
    let p2 = util::pass_hash::hash_password("qwerty".to_string()).await.unwrap();
    let p3 = util::pass_hash::hash_password("pazpaz33".to_string()).await.unwrap();
    let p4 = util::pass_hash::hash_password("66pazpaz".to_string()).await.unwrap();

    let users = sqlx::query_as!(UserModel, 
    r#"
        INSERT INTO users(username, password_hash)
        VALUES
            ('horam', $1),
            ('whatever', $2),
            ('andhere', $3),
            ('ten0g', $4)
        RETURNING id, username, password_hash
    "#, p1, p2, p3 ,p4)
        .fetch_all(db)
        .await
        .unwrap();

    let groups = sqlx::query_as!(GroupModel, r#"
        INSERT INTO groups(name)
        VALUES
            ('andersons farm'),
            ('the valhalla'),
            ('x84-64')
        RETURNING id, name
    "#)
        .fetch_all(db)
        .await
        .unwrap();

    sqlx::query!(r#"
        INSERT INTO messages(sender_id, receiver_group_id, content, msg_type)
        VALUES
            ($1, $5, 'psql16 really likes single quotes tho', 'normal'),
            (NULL, $5, 'someone joined.', 'event'),
            ($2, $5, 'you could shoot yourself in the foot', 'normal'),
            ($3, $6, 'if you do not adhere to its', 'normal'),
            ($1, $5, 'conventions and semantics.', 'normal'),
            ($1, $6, 'i am out of words', 'normal'),
            ($4, $5, 'or maybe not', 'normal'),
            ($2, $5, 'the well-lit room is awaiting me', 'normal')
    "#, 
        users[0].id, 
        users[1].id,
        users[2].id,
        users[3].id,
        groups[0].id,
        groups[1].id
    )
        .execute(db)
        .await
        .unwrap();

    sqlx::query!(r#"
        INSERT INTO user_groups(user_id, group_id, role)
        VALUES
            ($1, $5, 'admin'), 
            ($2, $5, 'user'),
            ($3, $5, 'user'),
            ($2, $6, 'admin'),
            ($1, $6, 'user'),
            ($4, $6, 'user'),
            ($4, $7, 'admin'),
            ($1, $7, 'user')
    "#, 
        users[0].id,
        users[1].id,
        users[2].id,
        users[3].id,
        groups[0].id,
        groups[1].id,
        groups[2].id,
    )
        .execute(db)
        .await
        .unwrap();

}
