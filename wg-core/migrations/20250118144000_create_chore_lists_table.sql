create table chore_lists
(
    id text not null primary key,
    name text not null,
    date_created timestamp not null default current_timestamp,
    date_deleted timestamp default null
);

create table chores
(
    id text not null primary key,
    chore_list_id text not null references chore_lists(id),
    name text not null,
    points int not null,
    date_created timestamp not null default current_timestamp,
    date_deleted timestamp default null
);
