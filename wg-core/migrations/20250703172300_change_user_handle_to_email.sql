alter table users rename column handle to email;

-- rename index
drop index users_unique_handle_idx;
create unique index users_unique_email_idx on users(email);
