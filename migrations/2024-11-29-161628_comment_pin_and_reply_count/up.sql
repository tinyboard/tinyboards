alter table comments
add column is_pinned boolean default false;

alter table comment_aggregates
add column reply_count integer default 0;

update comment_aggregates
set
    reply_count = subquery.reply_count
from
    (
        select
            count(*) as "reply_count",
            parent_id
        from
            comments c
        group by c.parent_id
    ) as subquery
where comment_aggregates.comment_id = subquery.parent_id;

create or replace function update_reply_count()
    returns trigger
    language plpgsql
as
$$
begin
    update comment_aggregates
        set reply_count = reply_count + 1
        where comment_id = NEW.parent_id;

    return NEW;
end;
$$;

create trigger comment_aggregates_update_reply_count
    after insert
    on comments
    for each row
    execute function update_reply_count();
