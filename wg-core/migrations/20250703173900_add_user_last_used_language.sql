alter table users add column last_used_language text default null;
alter table authentication_sessions add column last_used_language text default null;
