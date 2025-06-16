create table authentication_sessions
(
    id text not null primary key,
    token text not null,
    user_id text not null references users(id),
    date_expires timestamp not null,
    date_created timestamp not null default current_timestamp
);

create index authentication_sessions_token_idx on authentication_sessions(token);
