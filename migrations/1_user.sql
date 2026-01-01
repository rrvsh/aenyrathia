create table "user_data"
(
  id            uuid primary key default gen_random_uuid(),
  full_name     text unique not null,
  email         text unique not null,
  password_hash text        not null
);
