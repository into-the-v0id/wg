create table users
(
    id text not null primary key,
    first_name text not null,
    date_created timestamp not null default current_timestamp,
    date_deleted timestamp default null
);
