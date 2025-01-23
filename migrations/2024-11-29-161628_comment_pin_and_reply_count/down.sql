alter table comments
drop column is_pinned;

drop trigger comment_aggregates_update_reply_count on comments cascade;

alter table comment_aggregates
drop column reply_count;
