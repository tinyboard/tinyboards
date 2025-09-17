-- These columns don't need to be updated with a trigger, so they're saved daily via queries
alter table site_aggregates add column users_active_day bigint not null default 0;
alter table site_aggregates add column users_active_week bigint not null default 0;
alter table site_aggregates add column users_active_month bigint not null default 0;
alter table site_aggregates add column users_active_half_year bigint not null default 0;

alter table board_aggregates add column users_active_day bigint not null default 0;
alter table board_aggregates add column users_active_week bigint not null default 0;
alter table board_aggregates add column users_active_month bigint not null default 0;
alter table board_aggregates add column users_active_half_year bigint not null default 0;

create or replace function site_aggregates_activity(i text)
returns int
language plpgsql
as
$$
declare
   count_ integer;
begin
  select count(*) 
  into count_
  from (
    select c.creator_id from comments c
    inner join person p on c.creator_id = p.id
    where c.creation_date > ('now'::timestamp - i::interval) 
    and p.local = true
    union
    select p.creator_id from posts p
    inner join person pu on p.creator_id = pu.id
    where p.creation_date > ('now'::timestamp - i::interval)
    and p.local = true
  ) a;
  return count_;
end;
$$;

update site_aggregates 
set users_active_day = (select * from site_aggregates_activity('1 day'));

update site_aggregates 
set users_active_week = (select * from site_aggregates_activity('1 week'));

update site_aggregates 
set users_active_month = (select * from site_aggregates_activity('1 month'));

update site_aggregates 
set users_active_half_year = (select * from site_aggregates_activity('6 months'));

create or replace function board_aggregates_activity(i text)
returns table(count_ bigint, board_id_ integer)
language plpgsql
as
$$
begin
  return query 
  select count(*), board_id
  from (
    select c.creator_id, p.board_id from comments c
    inner join posts p on c.post_id = p.id
    where c.creation_date > ('now'::timestamp - i::interval)
    union
    select p.creator_id, p.board_id from posts p
    where p.creation_date > ('now'::timestamp - i::interval)  
  ) a
  group by board_id;
end;
$$;

update board_aggregates ba
set users_active_day = mv.count_
from board_aggregates_activity('1 day') mv
where ba.board_id = mv.board_id_;

update board_aggregates ba
set users_active_week = mv.count_
from board_aggregates_activity('1 week') mv
where ba.board_id = mv.board_id_;

update board_aggregates ba
set users_active_month = mv.count_
from board_aggregates_activity('1 month') mv
where ba.board_id = mv.board_id_;

update board_aggregates ba
set users_active_half_year = mv.count_
from board_aggregates_activity('6 months') mv
where ba.board_id = mv.board_id_;