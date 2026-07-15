create table user_data
(
  id            integer primary key autoincrement,
  full_name     text unique not null,
  email         text unique not null,
  password_hash text        not null
);
