-- Revert SQL functions back to use 'person' table references
-- This down migration reverts the functions to their previous state

-- Revert site_aggregates_activity function to use person table
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
    union
    select p.creator_id from posts p
    inner join person pu on p.creator_id = pu.id
    where p.creation_date > ('now'::timestamp - i::interval)
  ) a;
  return count_;
end;
$$;

-- Revert board_aggregates_activity function to use person table
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