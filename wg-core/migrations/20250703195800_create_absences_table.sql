create table absences
(
    id text not null primary key,
    user_id text not null references users(id),
    date_start timestamp not null,
    date_end timestamp null,
    comment text null default null,
    date_created timestamp not null default current_timestamp,
    date_deleted timestamp default null
);

create index absences_user_id_idx on absences(user_id);
