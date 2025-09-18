-- TinyBoards Essential Migration 1/3: Core Schema and Extensions
-- This migration creates the foundational database structure including extensions,
-- core tables, sequences, and basic constraints.

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS pgcrypto WITH SCHEMA public;

-- Core PostgreSQL functions for aggregation and triggers
CREATE FUNCTION public.board_aggregates_activity(i text) RETURNS TABLE(count_ bigint, board_id_ integer)
    LANGUAGE plpgsql
    AS $$
begin
  return query
  select count(*), board_id
  from (
    select c.creator_id, p.board_id from comments c
    inner join posts p on c.post_id = p.id
    inner join users u on c.creator_id = u.id
    where c.creation_date > ('now'::timestamp - i::interval)
    union
    select p.creator_id, p.board_id from posts p
    inner join users u on p.creator_id = u.id
    where p.creation_date > ('now'::timestamp - i::interval)
  ) a
  group by board_id;
end;
$$;

-- Trigger functions for board aggregates
CREATE FUNCTION public.board_aggregates_board() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
begin
  IF (TG_OP = 'INSERT') THEN
    insert into board_aggregates (board_id) values (NEW.id);
  ELSIF (TG_OP = 'DELETE') THEN
    delete from board_aggregates where board_id = OLD.id;
  END IF;
  return null;
end $$;

CREATE FUNCTION public.board_aggregates_comment_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
begin
  IF (TG_OP = 'INSERT') THEN
    update board_aggregates ba
    set comments = comments + 1 from comments c, posts p
    where p.id = c.post_id
    and p.id = NEW.post_id
    and ba.board_id = p.board_id;
  ELSIF (TG_OP = 'DELETE') THEN
    update board_aggregates ba
    set comments = comments - 1 from comments c, posts p
    where p.id = c.post_id
    and p.id = OLD.post_id
    and ba.board_id = p.board_id;
  END IF;
  return null;
end $$;

CREATE FUNCTION public.board_aggregates_post_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
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
      from boards b
      left join posts p on b.id = p.board_id
      left join comments ct on p.id = ct.post_id
      group by b.id
    ) bd
    where ba.board_id = OLD.board_id;
  END IF;
  return null;
end $$;

CREATE FUNCTION public.board_aggregates_subscriber_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
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

-- Board moderation trigger functions
CREATE FUNCTION public.board_mods_delete_update_ranks() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
begin
	update board_mods
		set rank = rank - 1
		where board_id = OLD.board_id
		and rank >= OLD.rank;
	return OLD;
end;
$$;

CREATE FUNCTION public.board_mods_insert_set_rank() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
declare
	lowest_rank int;
begin
	select rank into lowest_rank
		from board_mods
		where board_id = NEW.board_id
		order by rank desc
		limit 1;

	if lowest_rank is null then
		NEW.rank := 1;
	else
		NEW.rank := lowest_rank + 1;
	end if;

	return NEW;
end;
$$;

-- Utility function for timestamps
CREATE FUNCTION public.set_updated_at() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.updated = NOW();
    RETURN NEW;
END;
$$;

CREATE FUNCTION public.set_invite_accepted_date() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF OLD.invite_accepted = false AND NEW.invite_accepted = true THEN
        NEW.invite_accepted_date = NOW();
    END IF;
    RETURN NEW;
END;
$$;

-- Languages table (required for content)
CREATE TABLE public.language (
    id integer NOT NULL,
    code character varying(3) NOT NULL,
    name text NOT NULL
);

CREATE SEQUENCE public.language_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.language_id_seq OWNED BY public.language.id;
ALTER TABLE ONLY public.language ALTER COLUMN id SET DEFAULT nextval('public.language_id_seq'::regclass);

-- Core site configuration (COMPLETE with ALL fields from current database)
CREATE TABLE public.site (
    id integer NOT NULL,
    site_setup boolean DEFAULT false NOT NULL,
    invite_only boolean DEFAULT false NOT NULL,
    enable_downvotes boolean DEFAULT true NOT NULL,
    open_registration boolean DEFAULT true NOT NULL,
    enable_nsfw boolean DEFAULT true NOT NULL,
    board_creation_admin_only boolean DEFAULT false NOT NULL,
    require_email_verification boolean DEFAULT false NOT NULL,
    require_application boolean DEFAULT true NOT NULL,
    application_question text DEFAULT 'To verify that you are a human, please explain why you want to create an account on this site'::text,
    private_instance boolean DEFAULT true NOT NULL,
    default_theme text DEFAULT 'browser'::text NOT NULL,
    default_post_listing_type text DEFAULT 'Local'::text NOT NULL,
    default_avatar text,
    legal_information text,
    hide_modlog_mod_names boolean DEFAULT true NOT NULL,
    application_email_admins boolean DEFAULT false NOT NULL,
    captcha_enabled boolean DEFAULT false NOT NULL,
    captcha_difficulty character varying(255) DEFAULT 'medium'::character varying NOT NULL,
    creation_date timestamp without time zone DEFAULT now() NOT NULL,
    updated timestamp without time zone,
    reports_email_admins boolean DEFAULT false NOT NULL,
    name character varying(50) NOT NULL,
    primary_color character varying(25),
    secondary_color character varying(25),
    hover_color character varying(25),
    description character varying(255),
    icon character varying(255),
    welcome_message character varying(255),
    boards_enabled boolean DEFAULT true NOT NULL,
    board_creation_mode character varying(20) DEFAULT 'admin'::character varying NOT NULL,
    trusted_user_min_reputation integer DEFAULT 100 NOT NULL,
    trusted_user_min_account_age_days integer DEFAULT 30 NOT NULL,
    trusted_user_manual_approval boolean DEFAULT false NOT NULL,
    trusted_user_min_posts integer DEFAULT 5 NOT NULL,
    allowed_post_types text,
    enable_nsfw_tagging boolean,
    word_filter_enabled boolean,
    filtered_words text,
    word_filter_applies_to_posts boolean,
    word_filter_applies_to_comments boolean,
    word_filter_applies_to_usernames boolean,
    link_filter_enabled boolean,
    banned_domains text,
    approved_image_hosts text,
    image_embed_hosts_only boolean,
    registration_mode character varying DEFAULT 'RequireApplication'::character varying NOT NULL,
    emoji_enabled boolean DEFAULT true NOT NULL,
    max_emojis_per_post integer,
    max_emojis_per_comment integer,
    emoji_max_file_size_mb integer DEFAULT 2 NOT NULL,
    board_emojis_enabled boolean DEFAULT true NOT NULL
);

CREATE SEQUENCE public.site_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.site_id_seq OWNED BY public.site.id;
ALTER TABLE ONLY public.site ALTER COLUMN id SET DEFAULT nextval('public.site_id_seq'::regclass);

-- Users table (COMPLETE with ALL fields from current database)
CREATE TABLE public.users (
    id integer NOT NULL,
    name character varying(30) NOT NULL,
    display_name character varying(30),
    email text,
    passhash text NOT NULL,
    email_verified boolean DEFAULT false NOT NULL,
    is_banned boolean DEFAULT false NOT NULL,
    is_deleted boolean DEFAULT false NOT NULL,
    is_admin boolean DEFAULT false NOT NULL,
    admin_level integer DEFAULT 0 NOT NULL,
    unban_date timestamp without time zone,
    bio text,
    bio_html text,
    signature text,
    avatar text,
    banner text,
    profile_background text,
    avatar_frame text,
    profile_music text,
    profile_music_youtube text,
    show_nsfw boolean DEFAULT false NOT NULL,
    show_bots boolean DEFAULT true NOT NULL,
    theme text DEFAULT 'browser'::text NOT NULL,
    default_sort_type smallint DEFAULT 0 NOT NULL,
    default_listing_type smallint DEFAULT 0 NOT NULL,
    interface_language text DEFAULT 'browser'::text NOT NULL,
    email_notifications_enabled boolean DEFAULT false NOT NULL,
    bot_account boolean DEFAULT false NOT NULL,
    board_creation_approved boolean DEFAULT false NOT NULL,
    accepted_application boolean DEFAULT false NOT NULL,
    is_application_accepted boolean DEFAULT false NOT NULL,
    creation_date timestamp without time zone DEFAULT now() NOT NULL,
    updated timestamp without time zone
);

CREATE SEQUENCE public.users_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.users_id_seq OWNED BY public.users.id;
ALTER TABLE ONLY public.users ALTER COLUMN id SET DEFAULT nextval('public.users_id_seq'::regclass);

-- Boards table
CREATE TABLE public.boards (
    id integer NOT NULL,
    name character varying(50) NOT NULL,
    title character varying(150) NOT NULL,
    description text,
    creation_date timestamp without time zone DEFAULT now() NOT NULL,
    updated timestamp without time zone,
    is_deleted boolean DEFAULT false NOT NULL,
    is_nsfw boolean DEFAULT false NOT NULL,
    is_hidden boolean DEFAULT false NOT NULL,
    last_refreshed_date timestamp without time zone DEFAULT now() NOT NULL,
    instance_id integer DEFAULT 1 NOT NULL,
    icon text,
    banner text,
    posting_restricted_to_mods boolean DEFAULT false NOT NULL,
    is_removed boolean DEFAULT false NOT NULL,
    ban_reason character varying(512),
    primary_color character varying(25) DEFAULT '60, 105, 145'::character varying NOT NULL,
    secondary_color character varying(25) DEFAULT '96, 128, 63'::character varying NOT NULL,
    hover_color character varying(25) DEFAULT '54, 94, 129'::character varying NOT NULL,
    sidebar character varying(10000),
    sidebar_html text,
    is_banned boolean DEFAULT false NOT NULL,
    public_ban_reason text,
    banned_by integer,
    banned_at timestamp without time zone,
    exclude_from_all boolean DEFAULT false NOT NULL,
    moderators_url text,
    featured_url text
);

CREATE SEQUENCE public.board_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.board_id_seq OWNED BY public.boards.id;
ALTER TABLE ONLY public.boards ALTER COLUMN id SET DEFAULT nextval('public.board_id_seq'::regclass);

-- Secret table for JWT tokens
CREATE TABLE public.secret (
    id integer NOT NULL,
    jwt_secret character varying NOT NULL
);

CREATE SEQUENCE public.secret_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.secret_id_seq OWNED BY public.secret.id;
ALTER TABLE ONLY public.secret ALTER COLUMN id SET DEFAULT nextval('public.secret_id_seq'::regclass);

-- Primary key constraints
ALTER TABLE ONLY public.language ADD CONSTRAINT language_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.site ADD CONSTRAINT site_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.users ADD CONSTRAINT users_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.boards ADD CONSTRAINT boards_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.secret ADD CONSTRAINT secret_pkey PRIMARY KEY (id);

-- Unique constraints
ALTER TABLE ONLY public.language ADD CONSTRAINT language_code_key UNIQUE (code);
ALTER TABLE ONLY public.users ADD CONSTRAINT users_name_key UNIQUE (name);
ALTER TABLE ONLY public.boards ADD CONSTRAINT boards_name_key UNIQUE (name);

-- Basic foreign key constraints will be added in later migrations

-- Insert default language
INSERT INTO public.language (id, code, name) VALUES (1, 'en', 'English') ON CONFLICT DO NOTHING;

-- Insert default site configuration
INSERT INTO public.site (id, name, description, site_setup) VALUES
(1, 'TinyBoards', 'A modern discussion platform', false) ON CONFLICT DO NOTHING;

-- Insert JWT secret (generated using pgcrypto)
INSERT INTO public.secret (id, jwt_secret) VALUES
(1, encode(gen_random_bytes(32), 'hex')) ON CONFLICT DO NOTHING;