-- remerge local_user and person
alter table person rename to users;

alter table users
    add column theme text default '' not null,
    add column passhash text not null default '',
    add column email text unique,
    add column is_admin boolean not null default false,
    add column show_nsfw boolean not null default false,
    add column default_sort_type smallint not null default 0,
    add column default_listing_type smallint not null default 1,
    add column email_notifications_enabled boolean not null default false,
    add column accepted_application boolean not null default false,
    add column is_application_accepted boolean not null default false,
    add column email_verified boolean not null default false,
    add column chat_id text not null default '';

alter table users rename column display_name to preferred_name;

update users u
    set 
        theme = lu.theme,
        passhash = lu.passhash,
        email = lu.email,
        is_admin = lu.is_admin,
        show_nsfw = lu.show_nsfw,
        default_sort_type = lu.default_sort_type,
        default_listing_type = lu.default_listing_type,
        email_notifications_enabled = lu.email_notifications_enabled,
        accepted_application = lu.accepted_application,
        is_application_accepted = lu.is_application_accepted,
        email_verified = lu.email_verified
from (
    select
        person_id,
        theme,
        passhash,
        email,
        is_admin,
        show_nsfw,
        default_sort_type,
        default_listing_type,
        email_notifications_enabled,
        accepted_application,
        is_application_accepted,
        email_verified
    from local_user
) lu
where u.id = lu.person_id;

drop table local_user cascade;

alter table users
    drop column actor_id,
    drop column local,
    drop column private_key,
    drop column public_key,
    drop column inbox_url,
    drop column shared_inbox_url,
    drop column bot_account,
    drop column last_refreshed_date;

alter table users drop constraint if exists idx_person_inbox_url;

alter table boards
    drop column actor_id,
    drop column local,
    drop column private_key,
    drop column public_key,
    drop column subscribers_url,
    drop column inbox_url,
    drop column shared_inbox_url,
    drop column last_refreshed_date;

alter table boards
    drop constraint if exists idx_board_subscriptions_url,
    drop constraint if exists idx_board_inbox_url;

drop function generate_unique_changeme cascade;

-- rename triggers
alter trigger site_aggregates_person_delete on users rename to site_aggregates_user_delete;
alter trigger site_aggregates_person_insert on users rename to site_aggregates_user_insert;

-- rename trigger functions
alter function site_aggregates_person_delete() rename to site_aggregates_user_delete;
alter function site_aggregates_person_insert() rename to site_aggregates_user_insert;

-- remake user_aggregates into person_aggregates
alter table person_aggregates rename to user_aggregates;
alter sequence person_aggregates_id_seq rename to user_aggregates_id_seq;


-- drop old triggers and functions
drop trigger person_aggregates_person on users;
drop trigger person_aggregates_post_count on posts;
drop trigger person_aggregates_post_score on post_votes;
drop trigger person_aggregates_comment_count on comments;
drop trigger person_aggregates_comment_score on comment_votes;
drop function
    person_aggregates_person,
    person_aggregates_post_count,
    person_aggregates_post_score,
    person_aggregates_comment_count,
    person_aggregates_comment_score;

-- person_mentions
alter table person_mentions rename to user_mentions;
alter sequence person_mention_id_seq rename to user_mention_id_seq;
alter index person_mention_pkey rename to user_mention_pkey;
alter index person_mention_recipient_id_comment_id_key rename to user_mention_recipient_id_comment_id_key;
alter table user_mentions rename constraint person_mention_comment_id_fkey to user_mention_comment_id_fkey;
alter table user_mentions rename constraint person_mention_recipient_id_fkey to user_mention_recipient_id_fkey;

-- user_ban
alter table person_ban rename to user_ban;
alter sequence person_ban_id_seq rename to user_ban_id_seq;
alter index person_ban_pkey rename to user_ban_pkey;
alter index if exists person_ban_user_id_key rename to user_ban_person_id_key;
alter table user_ban rename column person_id to user_id;

-- comment_votes
alter table comment_votes rename column person_id to user_id;

-- user_comment_save
alter table comment_saved rename to user_comment_save;
alter table user_comment_save rename column person_id to user_id;

-- board_subscriptions
alter table board_subscriptions rename column person_id to user_id;

-- board_mods
alter table board_mods rename column person_id to user_id;

-- board_user_bans
alter table board_person_bans rename to board_user_bans;
alter table boauploadfrd_user_bans rename column person_id to user_id;

-- mod_add_board_mod
alter table mod_add_board_mod rename column mod_person_id to mod_user_id;
alter table mod_add_board_mod rename column other_person_id to other_user_id;

-- mod_add_board
alter table mod_add_board rename column mod_person_id to mod_user_id;
alter table mod_add_board rename column other_person_id to other_user_id;

-- mod_ban
alter table mod_ban rename column mod_person_id to mod_user_id;
alter table mod_ban rename column other_person_id to other_user_id;

-- mod_ban_board
alter table mod_ban_from_board rename column mod_person_id to mod_user_id;
alter table mod_ban_from_board rename column other_person_id to other_user_id;

-- mod_lock_post
alter table mod_lock_post rename column mod_person_id to mod_user_id;

-- mod_remove_comment
alter table mod_remove_comment rename column mod_person_id to mod_user_id;

-- mod_remove_board
alter table mod_remove_board rename column mod_person_id to mod_user_id;

-- mod_remove_post
alter table mod_remove_post rename column mod_person_id to mod_user_id;

-- mod_sticky_post
alter table mod_sticky_post rename column mod_person_id to mod_user_id;

-- password_resets
delete from password_resets;
alter table password_resets drop column local_user_id;
alter table password_resets add column user_id integer not null references users(id) on update cascade on delete cascade;

-- post_votes
alter table post_votes rename column person_id to user_id;

-- user_post_read
alter table post_read rename to user_post_read;
alter table user_post_read rename column person_id to user_id;

-- user_post_save
alter table post_saved rename to user_post_save;
alter table user_post_save rename column person_id to user_id;

-- initial user add
create function user_aggregates_user()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    insert into user_aggregates (user_id) values (NEW.id);
  ELSIF (TG_OP = 'DELETE') THEN
    delete from user_aggregates where user_id = OLD.id;
  END IF;
  return null;
end $$;

create trigger user_aggregates_user
after insert or delete on users
for each row
execute procedure user_aggregates_user();

-- post count
create function user_aggregates_post_count()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update user_aggregates
    set post_count = post_count + 1 where user_id = NEW.creator_id;

  ELSIF (TG_OP = 'DELETE') THEN
    update user_aggregates
    set post_count = post_count - 1 where user_id = OLD.creator_id;

    -- If the post gets deleted, the score calculation trigger won't fire,
    -- so you need to re-calculate
    update user_aggregates ua
    set post_score = pd.score
    from (
      select u.id,
      coalesce(0, sum(pv.score)) as score
      -- User join because posts could be empty
      from users u
      left join posts p on u.id = p.creator_id
      left join post_votes pv on p.id = pv.post_id
      group by u.id
    ) pd
    where ua.user_id = OLD.creator_id;

  END IF;
  return null;
end $$;

create trigger user_aggregates_post_count
after insert or delete on posts
for each row
execute procedure user_aggregates_post_count();

-- post score
create function user_aggregates_post_score()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    -- Need to get the post creator, not the voter
    update user_aggregates ua
    set post_score = post_score + NEW.score
    from posts p
    where ua.user_id = p.creator_id and p.id = NEW.post_id;

  ELSIF (TG_OP = 'DELETE') THEN
    update user_aggregates ua
    set post_score = post_score - OLD.score
    from posts p
    where ua.user_id = p.creator_id and p.id = OLD.post_id;
  END IF;
  return null;
end $$;

create trigger user_aggregates_post_score
after insert or delete on post_votes
for each row
execute procedure user_aggregates_post_score();

create function user_aggregates_comment_count()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update user_aggregates
    set comment_count = comment_count + 1 where user_id = NEW.creator_id;
  ELSIF (TG_OP = 'DELETE') THEN
    update user_aggregates
    set comment_count = comment_count - 1 where user_id = OLD.creator_id;

    -- If the comment gets deleted, the score calculation trigger won't fire,
    -- so you need to re-calculate
    update user_aggregates ua
    set comment_score = cd.score
    from (
      select u.id,
      coalesce(0, sum(cv.score)) as score
      -- User join because comments could be empty
      from user u
      left join comments c on u.id = c.creator_id
      left join comment_votes cv on c.id = cv.comment_id
      group by u.id
    ) cd
    where ua.user_id = OLD.creator_id;
  END IF;
  return null;
end $$;

create trigger user_aggregates_comment_count
after insert or delete on comments
for each row
execute procedure user_aggregates_comment_count();

-- comment score
create function user_aggregates_comment_score()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    -- Need to get the post creator, not the voter
    update user_aggregates ua
    set comment_score = comment_score + NEW.score
    from comments c
    where ua.user_id = c.creator_id and c.id = NEW.comment_id;
  ELSIF (TG_OP = 'DELETE') THEN
    update user_aggregates ua
    set comment_score = comment_score - OLD.score
    from comments c
    where ua.user_id = c.creator_id and c.id = OLD.comment_id;
  END IF;
  return null;
end $$;

create trigger user_aggregates_comment_score
after insert or delete on comment_votes
for each row
execute procedure user_aggregates_comment_score();
