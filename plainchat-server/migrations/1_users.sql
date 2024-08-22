create table if not exists "users" (
    id uuid primary key default gen_random_uuid(),
    username varchar(255) not null unique,
    password_hash varchar(255) not null
)
