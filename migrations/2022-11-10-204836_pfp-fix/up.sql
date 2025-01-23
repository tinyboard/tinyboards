alter table user_ alter column avatar set default '/assets/default_pfp.png';
update user_ set avatar='/assets/default_pfp.png';
alter table user_ alter column avatar drop not null;
