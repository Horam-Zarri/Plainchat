create type message_type as enum ('normal', 'event');

create table if not exists "messages" (
    id uuid primary key default gen_random_uuid(),
    sender_id uuid references users(id) on delete set null,
    receiver_group_id uuid not null,
    content text not null,
    msg_type message_type not null,
    created_at timestamp default now() not null,

    constraint fk_group foreign key(receiver_group_id) references groups(id)
)
