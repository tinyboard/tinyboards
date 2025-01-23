create table board_aggregates (
  id serial primary key,
  board_id int references board on update cascade on delete cascade not null,
  subscribers bigint not null default 0,
  posts bigint not null default 0,
  comments bigint not null default 0,
  published timestamp not null default now(),
  unique (board_id)
);

insert into board_aggregates (board_id, subscribers, posts, comments, published)
  select 
    b.id,
    coalesce(bs.subs, 0) as subscribers,
    coalesce(cd.posts, 0) as posts,
    coalesce(cd.comments, 0) as comments,
    b.published
  from board b
  left join ( 
    select 
      p.board_id,
      count(distinct p.id) as posts,
      count(distinct ct.id) as comments
    from post p
    left join comment ct on p.id = ct.post_id
    group by p.board_id
  ) cd on cd.board_id = b.id
  left join ( 
    select 
      board_subscriber.board_id,
      count(*) as subs
    from board_subscriber
    group by board_subscriber.board_id
  ) bs on bs.board_id = b.id;

-- Add board aggregate triggers

-- initial board add
create function board_aggregates_board()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    insert into board_aggregates (board_id) values (NEW.id);
  ELSIF (TG_OP = 'DELETE') THEN
    delete from board_aggregates where board_id = OLD.id;
  END IF;
  return null;
end $$;

create trigger board_aggregates_board
after insert or delete on board
for each row
execute procedure board_aggregates_board();
-- post count
create function board_aggregates_post_count()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update board_aggregates 
    set posts = posts + 1 where board_id = NEW.board_id;
  ELSIF (TG_OP = 'DELETE') THEN
    update board_aggregates 
    set posts = posts - 1 where board_id = OLD.board_id;

    -- Update the counts if the post got deleted
    update board_aggregates ba
    set posts = coalesce(bd.posts, 0),
    comments = coalesce(bd.comments, 0)
    from ( 
      select 
      b.id,
      count(distinct p.id) as posts,
      count(distinct ct.id) as comments
      from board b
      left join post p on b.id = p.board_id
      left join comment ct on p.id = ct.post_id
      group by b.id
    ) bd 
    where ba.board_id = OLD.board_id;
  END IF;
  return null;
end $$;

create trigger board_aggregates_post_count
after insert or delete on post
for each row
execute procedure board_aggregates_post_count();

-- comment count
create function board_aggregates_comment_count()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update board_aggregates ba
    set comments = comments + 1 from comment c, post p
    where p.id = c.post_id 
    and p.id = NEW.post_id 
    and ba.board_id = p.board_id;
  ELSIF (TG_OP = 'DELETE') THEN
    update board_aggregates ba
    set comments = comments - 1 from comment c, post p
    where p.id = c.post_id 
    and p.id = OLD.post_id 
    and ba.board_id = p.board_id;

  END IF;
  return null;
end $$;

create trigger board_aggregates_comment_count
after insert or delete on comment
for each row
execute procedure board_aggregates_comment_count();

-- subscriber count
create function board_aggregates_subscriber_count()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update board_aggregates 
    set subscribers = subscribers + 1 where board_id = NEW.board_id;
  ELSIF (TG_OP = 'DELETE') THEN
    update board_aggregates 
    set subscribers = subscribers - 1 where board_id = OLD.board_id;
  END IF;
  return null;
end $$;

create trigger board_aggregates_subscriber_count
after insert or delete on board_subscriber
for each row
execute procedure board_aggregates_subscriber_count();

