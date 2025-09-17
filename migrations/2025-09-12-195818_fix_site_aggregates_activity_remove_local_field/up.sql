-- Fix site_aggregates_activity function to remove references to removed 'local' field
-- This function was failing because it referenced p.local which was removed during defederation

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
