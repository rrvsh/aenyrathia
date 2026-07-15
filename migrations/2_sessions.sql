create table user_session
(
  token      text primary key not null,
  user_id    integer not null references user_data (id) on delete cascade,
  csrf_token text not null,
  created_at text not null default current_timestamp
);

create index user_session_user_id_idx on user_session (user_id);
