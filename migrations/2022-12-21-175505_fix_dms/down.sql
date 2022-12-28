-- fix dms
-- drop dm triggers
drop trigger refresh_dm on dms;
drop function refresh_dm;

-- recreate dm triggers
create or replace function refresh_dm()
returns trigger language plpgsql
as $$
begin
  refresh materialized view concurrently private_message_mview;
  return null;
end $$;

create trigger refresh_dm
after insert or update or delete or truncate
on dms
for each statement
execute procedure refresh_dm();
