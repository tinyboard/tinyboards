-- TinyBoards Essential Migration 2/3: Content and Aggregation Systems
-- This migration creates content tables (posts, comments), voting systems,
-- aggregation tables, and moderation infrastructure

-- Additional trigger functions needed for content
CREATE FUNCTION public.trigger_set_timestamp() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.updated = NOW();
    RETURN NEW;
END;
$$;

CREATE FUNCTION public.comment_aggregates_comment() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
begin
  IF (TG_OP = 'INSERT') THEN
    insert into comment_aggregates (comment_id) values (NEW.id);
  ELSIF (TG_OP = 'DELETE') THEN
    delete from comment_aggregates where comment_id = OLD.id;
  END IF;
  return null;
end $$;

CREATE FUNCTION public.post_aggregates_post() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
begin
  IF (TG_OP = 'INSERT') THEN
    insert into post_aggregates (post_id, board_id, creator_id) values (NEW.id, NEW.board_id, NEW.creator_id);
  ELSIF (TG_OP = 'DELETE') THEN
    delete from post_aggregates where post_id = OLD.id;
  END IF;
  return null;
end $$;

CREATE FUNCTION public.site_aggregates_post_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
begin
  update site_aggregates
  set posts = posts + 1;
  return null;
end $$;

CREATE FUNCTION public.site_aggregates_post_delete() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
begin
  update site_aggregates
  set posts = posts - 1;
  return null;
end $$;

CREATE FUNCTION public.user_aggregates_post_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
begin
  IF (TG_OP = 'INSERT') THEN
    update user_aggregates
    set post_count = post_count + 1 where user_id = NEW.creator_id;
  ELSIF (TG_OP = 'DELETE') THEN
    update user_aggregates
    set post_count = post_count - 1 where user_id = OLD.creator_id;
  END IF;
  return null;
end $$;

-- Posts table with ALL 27 fields from current database
CREATE TABLE public.posts (
    id integer NOT NULL,
    title character varying(200) NOT NULL,
    type_ character varying(10) DEFAULT 'text'::character varying NOT NULL,
    url text,
    thumbnail_url text,
    permalink text,
    body text NOT NULL,
    body_html text NOT NULL,
    creator_id integer NOT NULL,
    board_id integer NOT NULL,
    is_removed boolean DEFAULT false NOT NULL,
    is_locked boolean DEFAULT false NOT NULL,
    creation_date timestamp without time zone DEFAULT now() NOT NULL,
    is_deleted boolean DEFAULT false NOT NULL,
    is_nsfw boolean DEFAULT false NOT NULL,
    updated timestamp without time zone,
    image text,
    language_id integer,
    featured_board boolean DEFAULT false NOT NULL,
    featured_local boolean DEFAULT false NOT NULL,
    alt_text text,
    embed_title text,
    embed_description text,
    embed_video_url text,
    source_url text,
    last_crawl_date timestamp without time zone,
    title_chunk character varying(255) DEFAULT ''::character varying NOT NULL
);

CREATE SEQUENCE public.posts_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.posts_id_seq OWNED BY public.posts.id;
ALTER TABLE ONLY public.posts ALTER COLUMN id SET DEFAULT nextval('public.posts_id_seq'::regclass);

-- Comments table
CREATE TABLE public.comments (
    id integer NOT NULL,
    creator_id integer NOT NULL,
    post_id integer NOT NULL,
    parent_id integer,
    body text NOT NULL,
    body_html text NOT NULL,
    is_removed boolean DEFAULT false NOT NULL,
    read boolean DEFAULT false NOT NULL,
    creation_date timestamp without time zone DEFAULT now() NOT NULL,
    level integer DEFAULT 0 NOT NULL,
    is_deleted boolean DEFAULT false NOT NULL,
    updated timestamp without time zone,
    is_locked boolean DEFAULT false NOT NULL,
    board_id integer NOT NULL,
    language_id integer,
    is_pinned boolean
);

CREATE SEQUENCE public.comments_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.comments_id_seq OWNED BY public.comments.id;
ALTER TABLE ONLY public.comments ALTER COLUMN id SET DEFAULT nextval('public.comments_id_seq'::regclass);

-- Voting tables
CREATE TABLE public.post_votes (
    id integer NOT NULL,
    user_id integer NOT NULL,
    post_id integer NOT NULL,
    score integer NOT NULL,
    creation_date timestamp without time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.post_votes_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.post_votes_id_seq OWNED BY public.post_votes.id;
ALTER TABLE ONLY public.post_votes ALTER COLUMN id SET DEFAULT nextval('public.post_votes_id_seq'::regclass);

CREATE TABLE public.comment_votes (
    id integer NOT NULL,
    user_id integer NOT NULL,
    comment_id integer NOT NULL,
    post_id integer NOT NULL,
    score integer NOT NULL,
    creation_date timestamp without time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.comment_votes_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.comment_votes_id_seq OWNED BY public.comment_votes.id;
ALTER TABLE ONLY public.comment_votes ALTER COLUMN id SET DEFAULT nextval('public.comment_votes_id_seq'::regclass);

-- Aggregation tables with exact field structures
CREATE TABLE public.post_aggregates (
    id integer NOT NULL,
    post_id integer NOT NULL,
    comments bigint DEFAULT 0 NOT NULL,
    score bigint DEFAULT 0 NOT NULL,
    upvotes bigint DEFAULT 0 NOT NULL,
    downvotes bigint DEFAULT 0 NOT NULL,
    creation_date timestamp without time zone DEFAULT now() NOT NULL,
    newest_comment_time_necro timestamp without time zone,
    newest_comment_time timestamp without time zone DEFAULT now() NOT NULL,
    featured_board boolean DEFAULT false NOT NULL,
    featured_local boolean DEFAULT false NOT NULL,
    hot_rank integer DEFAULT 1728 NOT NULL,
    hot_rank_active integer DEFAULT 1728 NOT NULL,
    board_id integer DEFAULT 1 NOT NULL,
    creator_id integer DEFAULT 1 NOT NULL,
    controversy_rank double precision DEFAULT 0 NOT NULL
);

CREATE SEQUENCE public.post_aggregates_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.post_aggregates_id_seq OWNED BY public.post_aggregates.id;
ALTER TABLE ONLY public.post_aggregates ALTER COLUMN id SET DEFAULT nextval('public.post_aggregates_id_seq'::regclass);

CREATE TABLE public.comment_aggregates (
    id integer NOT NULL,
    comment_id integer NOT NULL,
    score bigint DEFAULT 0 NOT NULL,
    upvotes bigint DEFAULT 0 NOT NULL,
    downvotes bigint DEFAULT 0 NOT NULL,
    creation_date timestamp without time zone DEFAULT now() NOT NULL,
    child_count integer DEFAULT 0 NOT NULL,
    hot_rank integer DEFAULT 1728 NOT NULL,
    controversy_rank double precision DEFAULT 0 NOT NULL
);

CREATE SEQUENCE public.comment_aggregates_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.comment_aggregates_id_seq OWNED BY public.comment_aggregates.id;
ALTER TABLE ONLY public.comment_aggregates ALTER COLUMN id SET DEFAULT nextval('public.comment_aggregates_id_seq'::regclass);

CREATE TABLE public.board_aggregates (
    id integer NOT NULL,
    board_id integer NOT NULL,
    subscribers bigint DEFAULT 0 NOT NULL,
    posts bigint DEFAULT 0 NOT NULL,
    comments bigint DEFAULT 0 NOT NULL,
    creation_date timestamp without time zone DEFAULT now() NOT NULL,
    users_active_day bigint DEFAULT 0 NOT NULL,
    users_active_week bigint DEFAULT 0 NOT NULL,
    users_active_month bigint DEFAULT 0 NOT NULL,
    users_active_half_year bigint DEFAULT 0 NOT NULL
);

CREATE SEQUENCE public.board_aggregates_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.board_aggregates_id_seq OWNED BY public.board_aggregates.id;
ALTER TABLE ONLY public.board_aggregates ALTER COLUMN id SET DEFAULT nextval('public.board_aggregates_id_seq'::regclass);

CREATE TABLE public.user_aggregates (
    id integer NOT NULL,
    user_id integer NOT NULL,
    post_count bigint DEFAULT 0 NOT NULL,
    post_score bigint DEFAULT 0 NOT NULL,
    comment_count bigint DEFAULT 0 NOT NULL,
    comment_score bigint DEFAULT 0 NOT NULL
);

CREATE SEQUENCE public.user_aggregates_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.user_aggregates_id_seq OWNED BY public.user_aggregates.id;
ALTER TABLE ONLY public.user_aggregates ALTER COLUMN id SET DEFAULT nextval('public.user_aggregates_id_seq'::regclass);

CREATE TABLE public.site_aggregates (
    id integer NOT NULL,
    site_id integer NOT NULL,
    users bigint DEFAULT 0 NOT NULL,
    posts bigint DEFAULT 0 NOT NULL,
    comments bigint DEFAULT 0 NOT NULL,
    boards bigint DEFAULT 0 NOT NULL,
    users_active_day bigint DEFAULT 0 NOT NULL,
    users_active_week bigint DEFAULT 0 NOT NULL,
    users_active_month bigint DEFAULT 0 NOT NULL,
    users_active_half_year bigint DEFAULT 0 NOT NULL
);

CREATE SEQUENCE public.site_aggregates_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.site_aggregates_id_seq OWNED BY public.site_aggregates.id;
ALTER TABLE ONLY public.site_aggregates ALTER COLUMN id SET DEFAULT nextval('public.site_aggregates_id_seq'::regclass);

-- Core moderation tables
CREATE TABLE public.board_mods (
    id integer NOT NULL,
    board_id integer NOT NULL,
    user_id integer NOT NULL,
    creation_date timestamp without time zone DEFAULT now() NOT NULL,
    permissions integer DEFAULT 7 NOT NULL,
    rank integer NOT NULL,
    invite_accepted boolean DEFAULT false NOT NULL,
    invite_accepted_date timestamp without time zone
);

CREATE SEQUENCE public.board_mods_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.board_mods_id_seq OWNED BY public.board_mods.id;
ALTER TABLE ONLY public.board_mods ALTER COLUMN id SET DEFAULT nextval('public.board_mods_id_seq'::regclass);

-- Primary key constraints
ALTER TABLE ONLY public.posts ADD CONSTRAINT posts_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.comments ADD CONSTRAINT comments_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.post_votes ADD CONSTRAINT post_votes_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.comment_votes ADD CONSTRAINT comment_votes_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.post_aggregates ADD CONSTRAINT post_aggregates_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.comment_aggregates ADD CONSTRAINT comment_aggregates_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.board_aggregates ADD CONSTRAINT board_aggregates_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.user_aggregates ADD CONSTRAINT user_aggregates_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.site_aggregates ADD CONSTRAINT site_aggregates_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.board_mods ADD CONSTRAINT board_mods_pkey PRIMARY KEY (id);

-- Unique constraints
ALTER TABLE ONLY public.post_aggregates ADD CONSTRAINT post_aggregates_post_id_key UNIQUE (post_id);
ALTER TABLE ONLY public.comment_aggregates ADD CONSTRAINT comment_aggregates_comment_id_key UNIQUE (comment_id);
ALTER TABLE ONLY public.board_aggregates ADD CONSTRAINT board_aggregates_board_id_key UNIQUE (board_id);
ALTER TABLE ONLY public.user_aggregates ADD CONSTRAINT user_aggregates_user_id_key UNIQUE (user_id);

-- Foreign key constraints
ALTER TABLE ONLY public.posts ADD CONSTRAINT posts_board_id_fkey FOREIGN KEY (board_id) REFERENCES public.boards(id) ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE ONLY public.posts ADD CONSTRAINT posts_creator_id_fkey FOREIGN KEY (creator_id) REFERENCES public.users(id) ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE ONLY public.posts ADD CONSTRAINT posts_language_id_fkey FOREIGN KEY (language_id) REFERENCES public.language(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.comments ADD CONSTRAINT comments_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.posts(id) ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE ONLY public.comments ADD CONSTRAINT comments_creator_id_fkey FOREIGN KEY (creator_id) REFERENCES public.users(id) ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE ONLY public.comments ADD CONSTRAINT comments_language_id_fkey FOREIGN KEY (language_id) REFERENCES public.language(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.post_votes ADD CONSTRAINT post_votes_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.posts(id) ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE ONLY public.post_votes ADD CONSTRAINT post_votes_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.comment_votes ADD CONSTRAINT comment_votes_comment_id_fkey FOREIGN KEY (comment_id) REFERENCES public.comments(id) ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE ONLY public.comment_votes ADD CONSTRAINT comment_votes_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.posts(id) ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE ONLY public.comment_votes ADD CONSTRAINT comment_votes_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.post_aggregates ADD CONSTRAINT post_aggregates_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.posts(id) ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE ONLY public.comment_aggregates ADD CONSTRAINT comment_aggregates_comment_id_fkey FOREIGN KEY (comment_id) REFERENCES public.comments(id) ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE ONLY public.board_aggregates ADD CONSTRAINT board_aggregates_board_id_fkey FOREIGN KEY (board_id) REFERENCES public.boards(id) ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE ONLY public.user_aggregates ADD CONSTRAINT user_aggregates_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE ONLY public.site_aggregates ADD CONSTRAINT site_aggregates_site_id_fkey FOREIGN KEY (site_id) REFERENCES public.site(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.board_mods ADD CONSTRAINT board_mods_board_id_fkey FOREIGN KEY (board_id) REFERENCES public.boards(id) ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE ONLY public.board_mods ADD CONSTRAINT board_mods_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON UPDATE CASCADE ON DELETE CASCADE;

-- Create basic triggers (more will be added in migration 3)
CREATE TRIGGER post_aggregates_post AFTER INSERT OR DELETE ON public.posts FOR EACH ROW EXECUTE FUNCTION public.post_aggregates_post();
CREATE TRIGGER comment_aggregates_comment AFTER INSERT OR DELETE ON public.comments FOR EACH ROW EXECUTE FUNCTION public.comment_aggregates_comment();
CREATE TRIGGER board_aggregates_board AFTER INSERT OR DELETE ON public.boards FOR EACH ROW EXECUTE FUNCTION public.board_aggregates_board();

-- Insert default aggregates for site
INSERT INTO public.site_aggregates (site_id) VALUES (1) ON CONFLICT DO NOTHING;