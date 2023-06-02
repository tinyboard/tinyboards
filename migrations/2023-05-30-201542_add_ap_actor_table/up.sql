create or replace function generate_unique_changeme()
returns text language sql
as $$
    select 'changeme_' || string_agg (substr('abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz0123456789', ceil (random() * 62)::integer, 1), '')
    from generate_series(1, 20)
$$;

-- add apub columns for user
alter table users add column actor_id text not null default 'http://fake.com'; -- this should be checked and updated in code, build from site url if user is local
alter table users add column local boolean not null default true;
alter table users add column private_key text; -- keys generated in code
alter table users add column public_key text;
alter table users add column inbox_url text not null default generate_unique_changeme();
alter table users add column shared_inbox_url text;
alter table users add column bot_account boolean not null default false;
alter table users add column last_refreshed_date timestamp not null default now();
alter table users add constraint idx_person_inbox_url unique (inbox_url);

-- add apub columns for boards
alter table boards add column actor_id text not null default 'http://fake.com'; --needs to be checked and updated in code (built from site url if local)
alter table boards add column local boolean not null default true;
alter table boards add column private_key text; -- keys managed in code
alter table boards add column public_key text;
alter table boards add column subscribers_url text not null default generate_unique_changeme();
alter table boards add column inbox_url text not null default generate_unique_changeme();
alter table boards add column shared_inbox_url text;
alter table boards add column last_refreshed_date timestamp not null default now();
alter table boards add constraint idx_board_subscriptions_url unique (subscribers_url);
alter table boards add constraint idx_board_inbox_url unique (inbox_url);

-- rename users to person
alter table users rename to person;

-- create a new local_user table
create table local_user(
    id serial primary key,
    name text not null default '',
    person_id int references person on update cascade on delete cascade not null,
    passhash text not null,
    email text unique,
    is_admin boolean not null default false,
    is_banned boolean not null default false,
    is_deleted boolean not null default false,
    unban_date timestamp,
    show_nsfw boolean not null default false,
    show_bots boolean not null default true,
    theme text default '' not null,
    default_sort_type smallint not null default 0,
    default_listing_type smallint not null default 1,
    lang character varying(20) not null default 'browser'::character varying,
    email_notifications_enabled boolean not null default false,
    accepted_application boolean not null default false,
    is_application_accepted boolean not null default false,
    email_verified boolean not null default false,
    updated timestamp,
    creation_date timestamp not null default now(),
    unique (person_id)
);

-- copy any local users into new table
insert into local_user
(
    name,
    person_id,
    passhash,
    email,
    is_admin,
    show_nsfw,
    theme,
    default_sort_type,
    default_listing_type,
    email_notifications_enabled,
    accepted_application,
    is_application_accepted,
    email_verified
)
select
    name,
    id,
    passhash,
    email,
    is_admin,
    show_nsfw,
    theme,
    default_sort_type,
    default_listing_type,
    email_notifications_enabled,
    accepted_application,
    is_application_accepted,
    email_verified
from person
where local = true;

alter table person
    drop column theme,
    drop column passhash,
    drop column email cascade,
    drop column is_admin,
    drop column show_nsfw,
    drop column default_sort_type,
    drop column default_listing_type,
    drop column email_notifications_enabled,
    drop column accepted_application,
    drop column is_application_accepted,
    drop column email_verified,
    drop column chat_id;

alter table person rename column preferred_name to display_name;

-- rename triggers
alter trigger site_aggregates_user_delete on person rename to site_aggregates_person_delete;
alter trigger site_aggregates_user_insert on person rename to site_aggregates_person_insert;

-- rename trigger functions
alter function site_aggregates_user_delete() rename to site_aggregates_person_delete;
alter function site_aggregates_user_insert() rename to site_aggregates_person_insert;

-- remake user_aggregates into person_aggregates
alter table user_aggregates rename to person_aggregates;
alter sequence user_aggregates_id_seq rename to person_aggregates_id_seq;
alter table person_aggregates rename column user_id to person_id;

-- drop old triggers and functions
drop trigger user_aggregates_user on person;
drop trigger user_aggregates_post_count on posts;
drop trigger user_aggregates_post_score on post_votes;
drop trigger user_aggregates_comment_count on comments;
drop trigger user_aggregates_comment_score on comment_votes;
drop function
    user_aggregates_user,
    user_aggregates_post_count,
    user_aggregates_post_score,
    user_aggregates_comment_count,
    user_aggregates_comment_score;

-- initial user add
create function person_aggregates_person()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    insert into person_aggregates (person_id) values (NEW.id);
  ELSIF (TG_OP = 'DELETE') THEN
    delete from person_aggregates where person_id = OLD.id;
  END IF;
  return null;
end $$;

create trigger person_aggregates_person
after insert or delete on person
for each row
execute procedure person_aggregates_person();

-- post count
create function person_aggregates_post_count()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update person_aggregates
    set post_count = post_count + 1 where person_id = NEW.creator_id;

  ELSIF (TG_OP = 'DELETE') THEN
    update person_aggregates
    set post_count = post_count - 1 where person_id = OLD.creator_id;

    -- If the post gets deleted, the score calculation trigger won't fire,
    -- so you need to re-calculate
    update person_aggregates ua
    set post_score = pd.score
    from (
      select u.id,
      coalesce(0, sum(pv.score)) as score
      -- User join because posts could be empty
      from person u
      left join posts p on u.id = p.creator_id
      left join post_votes pv on p.id = pv.post_id
      group by u.id
    ) pd
    where ua.person_id = OLD.creator_id;

  END IF;
  return null;
end $$;

create trigger person_aggregates_post_count
after insert or delete on posts
for each row
execute procedure person_aggregates_post_count();

-- post score
create function person_aggregates_post_score()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    -- Need to get the post creator, not the voter
    update person_aggregates ua
    set post_score = post_score + NEW.score
    from posts p
    where ua.person_id = p.creator_id and p.id = NEW.post_id;

  ELSIF (TG_OP = 'DELETE') THEN
    update person_aggregates ua
    set post_score = post_score - OLD.score
    from posts p
    where ua.person_id = p.creator_id and p.id = OLD.post_id;
  END IF;
  return null;
end $$;

create trigger person_aggregates_post_score
after insert or delete on post_votes
for each row
execute procedure person_aggregates_post_score();

-- comment count
create function person_aggregates_comment_count()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update person_aggregates
    set comment_count = comment_count + 1 where person_id = NEW.creator_id;
  ELSIF (TG_OP = 'DELETE') THEN
    update person_aggregates
    set comment_count = comment_count - 1 where person_id = OLD.creator_id;

    -- If the comment gets deleted, the score calculation trigger won't fire,
    -- so you need to re-calculate
    update person_aggregates ua
    set comment_score = cd.score
    from (
      select u.id,
      coalesce(0, sum(cv.score)) as score
      -- User join because comments could be empty
      from person u
      left join comments c on u.id = c.creator_id
      left join comment_votes cv on c.id = cv.comment_id
      group by u.id
    ) cd
    where ua.person_id = OLD.creator_id;
  END IF;
  return null;
end $$;

create trigger person_aggregates_comment_count
after insert or delete on comments
for each row
execute procedure person_aggregates_comment_count();

-- comment score
create function person_aggregates_comment_score()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    -- Need to get the post creator, not the voter
    update person_aggregates ua
    set comment_score = comment_score + NEW.score
    from comments c
    where ua.person_id = c.creator_id and c.id = NEW.comment_id;
  ELSIF (TG_OP = 'DELETE') THEN
    update person_aggregates ua
    set comment_score = comment_score - OLD.score
    from comments c
    where ua.person_id = c.creator_id and c.id = OLD.comment_id;
  END IF;
  return null;
end $$;

create trigger person_aggregates_comment_score
after insert or delete on comment_votes
for each row
execute procedure person_aggregates_comment_score();

-- person_mentions
alter table user_mentions rename to person_mentions;
alter sequence user_mention_id_seq rename to person_mention_id_seq;
alter index user_mention_pkey rename to person_mention_pkey;
alter index user_mention_recipient_id_comment_id_key rename to person_mention_recipient_id_comment_id_key;
alter table person_mentions rename constraint user_mention_comment_id_fkey to person_mention_comment_id_fkey;
alter table person_mentions rename constraint user_mention_recipient_id_fkey to person_mention_recipient_id_fkey;

-- user_ban
alter table user_ban rename to person_ban;
alter sequence user_ban_id_seq rename to person_ban_id_seq;
alter index user_ban_pkey rename to person_ban_pkey;
alter index user_ban_user_id_key rename to person_ban_person_id_key;
alter table person_ban rename column user_id to person_id;
alter table person_ban rename constraint user_ban_user_id_fkey to person_ban_person_id_fkey;

-- comment_votes
alter table comment_votes rename column user_id to person_id;

-- user_comment_save
alter table user_comment_save rename to comment_saved;
alter table comment_saved rename column user_id to person_id;

-- board_subscriptions
alter table board_subscriptions rename column user_id to person_id;

-- board_mods
alter table board_mods rename column user_id to person_id;

-- board_user_bans
alter table board_user_bans rename to board_person_bans;
alter table board_person_bans rename column user_id to person_id;

-- mod_add_board_mod
alter table mod_add_board_mod rename column mod_user_id to mod_person_id;
alter table mod_add_board_mod rename column other_user_id to other_person_id;

-- mod_add_board
alter table mod_add_board rename column mod_user_id to mod_person_id;
alter table mod_add_board rename column other_user_id to other_person_id;

-- mod_ban
alter table mod_ban rename column mod_user_id to mod_person_id;
alter table mod_ban rename column other_user_id to other_person_id;

-- mod_ban_board
alter table mod_ban_from_board rename column mod_user_id to mod_person_id;
alter table mod_ban_from_board rename column other_user_id to other_person_id;

-- mod_lock_post
alter table mod_lock_post rename column mod_user_id to mod_person_id;

-- mod_remove_comment
alter table mod_remove_comment rename column mod_user_id to mod_person_id;

-- mod_remove_board
alter table mod_remove_board rename column mod_user_id to mod_person_id;

-- mod_remove_post
alter table mod_remove_post rename column mod_user_id to mod_person_id;

-- mod_sticky_post
alter table mod_sticky_post rename column mod_user_id to mod_person_id;

-- password_resets
delete from password_resets;
alter table password_resets drop column user_id;
alter table password_resets add column local_user_id integer not null references local_user(id) on update cascade on delete cascade;

-- post_votes
alter table post_votes rename column user_id to person_id;

-- user_post_read
alter table user_post_read rename to post_read;
alter table post_read rename column user_id to person_id;

-- user_post_save
alter table user_post_save rename to post_saved;
alter table post_saved rename column user_id to person_id;