create type user_role as enum ('user', 'admin');

create table if not exists "user_groups" (
    id uuid primary key default gen_random_uuid(),
    role user_role not null,
    user_id uuid not null,
    group_id uuid not null,

    constraint fk_user foreign key(user_id) references users(id),
    constraint fk_group foreign key(group_id) references groups(id)
)
