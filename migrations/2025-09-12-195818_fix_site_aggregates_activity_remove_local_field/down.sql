-- Restore the original function with local field references
-- Note: This rollback will fail if the 'local' field doesn't exist in the person table

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
