-- Fix SQL functions to reference 'users' table instead of 'person' table
-- This fixes automated task errors when updating site and board statistics

-- Update site_aggregates_activity function to use users table
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
    inner join users u on c.creator_id = u.id
    where c.creation_date > ('now'::timestamp - i::interval)
    union
    select p.creator_id from posts p
    inner join users u on p.creator_id = u.id
    where p.creation_date > ('now'::timestamp - i::interval)
  ) a;
  return count_;
end;
$$;

-- Update board_aggregates_activity function to use users table
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
    inner join users u on c.creator_id = u.id
    where c.creation_date > ('now'::timestamp - i::interval)
    union
    select p.creator_id, p.board_id from posts p
    inner join users u on p.creator_id = u.id
    where p.creation_date > ('now'::timestamp - i::interval)
  ) a
  group by board_id;
end;
$$;