-- Create the view and materialized view which has the avatar and creator name
create view private_message_view as 
select        
pm.*,
u.name as creator_name,
u.avatar as creator_avatar,
u2.name as recipient_name,
u2.avatar as recipient_avatar
from private_message pm
inner join user_ u on u.id = pm.creator_id
inner join user_ u2 on u2.id = pm.recipient_id;

create materialized view private_message_mview as select * from private_message_view;

create unique index idx_private_message_mview_id on private_message_mview (id);

-- Create the triggers
create or replace function refresh_private_message()
returns trigger language plpgsql
as $$
begin
  refresh materialized view concurrently private_message_mview;
  return null;
end $$;

create trigger refresh_private_message
after insert or update or delete or truncate
on private_message
for each statement
execute procedure refresh_private_message();