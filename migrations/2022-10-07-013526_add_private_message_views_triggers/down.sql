-- Drop the triggers
drop trigger if exists refresh_private_message on private_message;
drop function if exists refresh_private_message();

-- Drop the view
drop view if exists private_message_view cascade;