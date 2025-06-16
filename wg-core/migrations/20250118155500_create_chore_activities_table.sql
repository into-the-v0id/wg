create table chore_activities
(
    id text not null primary key,
    chore_id text not null references chores(id),
    user_id text not null references users(id),
    date timestamp not null,
    date_created timestamp not null default current_timestamp,
    date_deleted timestamp default null
);
