-- Drop the triggers
drop trigger refresh_private_message on private_message;
drop function refresh_private_message();

-- Drop the view
drop view private_message_view cascade;