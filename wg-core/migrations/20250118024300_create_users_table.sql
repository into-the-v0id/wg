create table users
(
    id text not null primary key,
    name text not null,
    handle text not null,
    password_hash text not null,
    date_created timestamp not null default current_timestamp,
    date_deleted timestamp default null
);
