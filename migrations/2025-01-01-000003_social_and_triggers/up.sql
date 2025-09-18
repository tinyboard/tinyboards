-- TinyBoards Essential Migration 3/3: Social Features and Complete Triggers
-- This migration creates all remaining tables and activates all triggers

-- All remaining social and admin tables
CREATE TABLE public.board_subscriber (
    id integer NOT NULL,
    board_id integer NOT NULL,
    user_id integer NOT NULL,
    creation_date timestamp without time zone DEFAULT now() NOT NULL,
    pending boolean DEFAULT false NOT NULL
);

CREATE SEQUENCE public.board_subscriber_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.board_subscriber_id_seq OWNED BY public.board_subscriber.id;
ALTER TABLE ONLY public.board_subscriber ALTER COLUMN id SET DEFAULT nextval('public.board_subscriber_id_seq'::regclass);

CREATE TABLE public.board_user_bans (
    id integer NOT NULL,
    board_id integer NOT NULL,
    user_id integer NOT NULL,
    creation_date timestamp without time zone DEFAULT now() NOT NULL,
    expires timestamp without time zone
);

CREATE SEQUENCE public.board_user_bans_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.board_user_bans_id_seq OWNED BY public.board_user_bans.id;
ALTER TABLE ONLY public.board_user_bans ALTER COLUMN id SET DEFAULT nextval('public.board_user_bans_id_seq'::regclass);

CREATE TABLE public.board_language (
    id integer NOT NULL,
    board_id integer NOT NULL,
    language_id integer NOT NULL
);

CREATE SEQUENCE public.board_language_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.board_language_id_seq OWNED BY public.board_language.id;
ALTER TABLE ONLY public.board_language ALTER COLUMN id SET DEFAULT nextval('public.board_language_id_seq'::regclass);

CREATE TABLE public.comment_saved (
    id integer NOT NULL,
    comment_id integer NOT NULL,
    user_id integer NOT NULL,
    creation_date timestamp without time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.comment_saved_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.comment_saved_id_seq OWNED BY public.comment_saved.id;
ALTER TABLE ONLY public.comment_saved ALTER COLUMN id SET DEFAULT nextval('public.comment_saved_id_seq'::regclass);

CREATE TABLE public.post_saved (
    id integer NOT NULL,
    post_id integer NOT NULL,
    user_id integer NOT NULL,
    creation_date timestamp without time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.post_saved_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.post_saved_id_seq OWNED BY public.post_saved.id;
ALTER TABLE ONLY public.post_saved ALTER COLUMN id SET DEFAULT nextval('public.post_saved_id_seq'::regclass);

CREATE TABLE public.post_read (
    id integer NOT NULL,
    post_id integer NOT NULL,
    user_id integer NOT NULL,
    creation_date timestamp without time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.post_read_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.post_read_id_seq OWNED BY public.post_read.id;
ALTER TABLE ONLY public.post_read ALTER COLUMN id SET DEFAULT nextval('public.post_read_id_seq'::regclass);

-- Admin and moderation tables
CREATE TABLE public.admin_ban_board (
    id integer NOT NULL,
    admin_id integer NOT NULL,
    board_id integer NOT NULL,
    internal_notes text,
    public_ban_reason text,
    action character varying(10) NOT NULL,
    when_ timestamp without time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.admin_ban_board_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.admin_ban_board_id_seq OWNED BY public.admin_ban_board.id;
ALTER TABLE ONLY public.admin_ban_board ALTER COLUMN id SET DEFAULT nextval('public.admin_ban_board_id_seq'::regclass);

CREATE TABLE public.admin_purge_board (
    id integer NOT NULL,
    admin_id integer NOT NULL,
    board_id integer NOT NULL,
    reason text,
    when_ timestamp without time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.admin_purge_board_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.admin_purge_board_id_seq OWNED BY public.admin_purge_board.id;
ALTER TABLE ONLY public.admin_purge_board ALTER COLUMN id SET DEFAULT nextval('public.admin_purge_board_id_seq'::regclass);

CREATE TABLE public.admin_purge_comment (
    id integer NOT NULL,
    admin_id integer NOT NULL,
    comment_id integer NOT NULL,
    reason text,
    when_ timestamp without time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.admin_purge_comment_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.admin_purge_comment_id_seq OWNED BY public.admin_purge_comment.id;
ALTER TABLE ONLY public.admin_purge_comment ALTER COLUMN id SET DEFAULT nextval('public.admin_purge_comment_id_seq'::regclass);

CREATE TABLE public.admin_purge_post (
    id integer NOT NULL,
    admin_id integer NOT NULL,
    post_id integer NOT NULL,
    reason text,
    when_ timestamp without time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.admin_purge_post_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.admin_purge_post_id_seq OWNED BY public.admin_purge_post.id;
ALTER TABLE ONLY public.admin_purge_post ALTER COLUMN id SET DEFAULT nextval('public.admin_purge_post_id_seq'::regclass);

CREATE TABLE public.admin_purge_user (
    id integer NOT NULL,
    admin_id integer NOT NULL,
    user_id integer NOT NULL,
    reason text,
    when_ timestamp without time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.admin_purge_user_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.admin_purge_user_id_seq OWNED BY public.admin_purge_user.id;
ALTER TABLE ONLY public.admin_purge_user ALTER COLUMN id SET DEFAULT nextval('public.admin_purge_user_id_seq'::regclass);

-- Mod action tables
CREATE TABLE public.mod_add_admin (
    id integer NOT NULL,
    mod_user_id integer NOT NULL,
    other_user_id integer NOT NULL,
    removed boolean,
    when_ timestamp without time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.mod_add_admin_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.mod_add_admin_id_seq OWNED BY public.mod_add_admin.id;
ALTER TABLE ONLY public.mod_add_admin ALTER COLUMN id SET DEFAULT nextval('public.mod_add_admin_id_seq'::regclass);

CREATE TABLE public.mod_add_board (
    id integer NOT NULL,
    mod_user_id integer NOT NULL,
    other_user_id integer NOT NULL,
    board_id integer NOT NULL,
    removed boolean,
    when_ timestamp without time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.mod_add_board_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.mod_add_board_id_seq OWNED BY public.mod_add_board.id;
ALTER TABLE ONLY public.mod_add_board ALTER COLUMN id SET DEFAULT nextval('public.mod_add_board_id_seq'::regclass);

CREATE TABLE public.mod_add_board_mod (
    id integer NOT NULL,
    mod_user_id integer NOT NULL,
    other_user_id integer NOT NULL,
    board_id integer NOT NULL,
    removed boolean,
    when_ timestamp without time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.mod_add_board_mod_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.mod_add_board_mod_id_seq OWNED BY public.mod_add_board_mod.id;
ALTER TABLE ONLY public.mod_add_board_mod ALTER COLUMN id SET DEFAULT nextval('public.mod_add_board_mod_id_seq'::regclass);

CREATE TABLE public.mod_ban (
    id integer NOT NULL,
    mod_user_id integer NOT NULL,
    other_user_id integer NOT NULL,
    reason text,
    banned boolean,
    expires timestamp without time zone,
    when_ timestamp without time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.mod_ban_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.mod_ban_id_seq OWNED BY public.mod_ban.id;
ALTER TABLE ONLY public.mod_ban ALTER COLUMN id SET DEFAULT nextval('public.mod_ban_id_seq'::regclass);

CREATE TABLE public.mod_ban_from_board (
    id integer NOT NULL,
    mod_user_id integer NOT NULL,
    other_user_id integer NOT NULL,
    board_id integer NOT NULL,
    reason text,
    banned boolean,
    expires timestamp without time zone,
    when_ timestamp without time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.mod_ban_from_board_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.mod_ban_from_board_id_seq OWNED BY public.mod_ban_from_board.id;
ALTER TABLE ONLY public.mod_ban_from_board ALTER COLUMN id SET DEFAULT nextval('public.mod_ban_from_board_id_seq'::regclass);

CREATE TABLE public.mod_feature_post (
    id integer NOT NULL,
    mod_user_id integer NOT NULL,
    post_id integer NOT NULL,
    featured boolean,
    when_ timestamp without time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.mod_feature_post_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.mod_feature_post_id_seq OWNED BY public.mod_feature_post.id;
ALTER TABLE ONLY public.mod_feature_post ALTER COLUMN id SET DEFAULT nextval('public.mod_feature_post_id_seq'::regclass);

CREATE TABLE public.mod_hide_board (
    id integer NOT NULL,
    board_id integer NOT NULL,
    mod_user_id integer NOT NULL,
    when_ timestamp without time zone DEFAULT now() NOT NULL,
    reason text,
    hidden boolean DEFAULT false NOT NULL
);

CREATE SEQUENCE public.mod_hide_board_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.mod_hide_board_id_seq OWNED BY public.mod_hide_board.id;
ALTER TABLE ONLY public.mod_hide_board ALTER COLUMN id SET DEFAULT nextval('public.mod_hide_board_id_seq'::regclass);

CREATE TABLE public.mod_lock_post (
    id integer NOT NULL,
    mod_user_id integer NOT NULL,
    post_id integer NOT NULL,
    locked boolean,
    when_ timestamp without time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.mod_lock_post_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.mod_lock_post_id_seq OWNED BY public.mod_lock_post.id;
ALTER TABLE ONLY public.mod_lock_post ALTER COLUMN id SET DEFAULT nextval('public.mod_lock_post_id_seq'::regclass);

CREATE TABLE public.mod_remove_board (
    id integer NOT NULL,
    mod_user_id integer NOT NULL,
    board_id integer NOT NULL,
    reason text,
    removed boolean,
    when_ timestamp without time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.mod_remove_board_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.mod_remove_board_id_seq OWNED BY public.mod_remove_board.id;
ALTER TABLE ONLY public.mod_remove_board ALTER COLUMN id SET DEFAULT nextval('public.mod_remove_board_id_seq'::regclass);

CREATE TABLE public.mod_remove_comment (
    id integer NOT NULL,
    mod_user_id integer NOT NULL,
    comment_id integer NOT NULL,
    reason text,
    removed boolean,
    when_ timestamp without time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.mod_remove_comment_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.mod_remove_comment_id_seq OWNED BY public.mod_remove_comment.id;
ALTER TABLE ONLY public.mod_remove_comment ALTER COLUMN id SET DEFAULT nextval('public.mod_remove_comment_id_seq'::regclass);

CREATE TABLE public.mod_remove_post (
    id integer NOT NULL,
    mod_user_id integer NOT NULL,
    post_id integer NOT NULL,
    reason text,
    removed boolean,
    when_ timestamp without time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.mod_remove_post_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.mod_remove_post_id_seq OWNED BY public.mod_remove_post.id;
ALTER TABLE ONLY public.mod_remove_post ALTER COLUMN id SET DEFAULT nextval('public.mod_remove_post_id_seq'::regclass);

-- Messaging and notification tables
CREATE TABLE public.messages (
    id integer NOT NULL,
    creator_id integer NOT NULL,
    recipient_id integer NOT NULL,
    subject character varying(200) NOT NULL,
    body text NOT NULL,
    body_html text NOT NULL,
    read boolean DEFAULT false NOT NULL,
    creation_date timestamp without time zone DEFAULT now() NOT NULL,
    updated timestamp without time zone,
    ap_id character varying(255) NOT NULL,
    local boolean DEFAULT true NOT NULL
);

CREATE SEQUENCE public.messages_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.messages_id_seq OWNED BY public.messages.id;
ALTER TABLE ONLY public.messages ALTER COLUMN id SET DEFAULT nextval('public.messages_id_seq'::regclass);

CREATE TABLE public.private_message (
    id integer NOT NULL,
    creator_id integer NOT NULL,
    recipient_user_id integer,
    recipient_board_id integer,
    body text NOT NULL,
    body_html text NOT NULL,
    published timestamp without time zone DEFAULT now() NOT NULL,
    updated timestamp without time zone,
    is_sender_hidden boolean DEFAULT false NOT NULL,
    title text NOT NULL
);

CREATE SEQUENCE public.private_message_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.private_message_id_seq OWNED BY public.private_message.id;
ALTER TABLE ONLY public.private_message ALTER COLUMN id SET DEFAULT nextval('public.private_message_id_seq'::regclass);

CREATE TABLE public.notifications (
    id integer NOT NULL,
    kind text NOT NULL,
    recipient_user_id integer NOT NULL,
    comment_id integer,
    post_id integer,
    message_id integer,
    created timestamp without time zone DEFAULT now() NOT NULL,
    is_read boolean DEFAULT false NOT NULL
);

CREATE SEQUENCE public.notifications_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.notifications_id_seq OWNED BY public.notifications.id;
ALTER TABLE ONLY public.notifications ALTER COLUMN id SET DEFAULT nextval('public.notifications_id_seq'::regclass);

CREATE TABLE public.pm_notif (
    id integer NOT NULL,
    recipient_id integer NOT NULL,
    pm_id integer NOT NULL,
    read boolean DEFAULT false NOT NULL,
    creation_date timestamp without time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.pm_notif_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.pm_notif_id_seq OWNED BY public.pm_notif.id;
ALTER TABLE ONLY public.pm_notif ALTER COLUMN id SET DEFAULT nextval('public.pm_notif_id_seq'::regclass);

-- Reports tables
CREATE TABLE public.comment_report (
    id integer NOT NULL,
    creator_id integer NOT NULL,
    comment_id integer NOT NULL,
    original_comment_text text NOT NULL,
    reason text NOT NULL,
    resolved boolean DEFAULT false NOT NULL,
    resolver_id integer,
    creation_date timestamp without time zone DEFAULT now() NOT NULL,
    updated timestamp without time zone
);

CREATE SEQUENCE public.comment_report_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.comment_report_id_seq OWNED BY public.comment_report.id;
ALTER TABLE ONLY public.comment_report ALTER COLUMN id SET DEFAULT nextval('public.comment_report_id_seq'::regclass);

CREATE TABLE public.comment_reports (
    id integer NOT NULL,
    creator_id integer NOT NULL,
    comment_id integer NOT NULL,
    original_comment_text text NOT NULL,
    reason text NOT NULL,
    resolved boolean DEFAULT false NOT NULL,
    resolver_id integer,
    creation_date timestamp without time zone DEFAULT now() NOT NULL,
    updated timestamp without time zone
);

CREATE SEQUENCE public.comment_reports_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.comment_reports_id_seq OWNED BY public.comment_reports.id;
ALTER TABLE ONLY public.comment_reports ALTER COLUMN id SET DEFAULT nextval('public.comment_reports_id_seq'::regclass);

CREATE TABLE public.post_report (
    id integer NOT NULL,
    creator_id integer NOT NULL,
    post_id integer NOT NULL,
    original_post_title text NOT NULL,
    original_post_url text,
    original_post_body text,
    reason text NOT NULL,
    resolved boolean DEFAULT false NOT NULL,
    resolver_id integer,
    creation_date timestamp without time zone DEFAULT now() NOT NULL,
    updated timestamp without time zone
);

CREATE SEQUENCE public.post_report_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.post_report_id_seq OWNED BY public.post_report.id;
ALTER TABLE ONLY public.post_report ALTER COLUMN id SET DEFAULT nextval('public.post_report_id_seq'::regclass);

CREATE TABLE public.post_reports (
    id integer NOT NULL,
    creator_id integer NOT NULL,
    post_id integer NOT NULL,
    original_post_title character varying(200) NOT NULL,
    original_post_url text,
    original_post_body text NOT NULL,
    reason text NOT NULL,
    resolved boolean DEFAULT false NOT NULL,
    resolver_id integer,
    creation_date timestamp without time zone DEFAULT now() NOT NULL,
    updated timestamp without time zone
);

CREATE SEQUENCE public.post_reports_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.post_reports_id_seq OWNED BY public.post_reports.id;
ALTER TABLE ONLY public.post_reports ALTER COLUMN id SET DEFAULT nextval('public.post_reports_id_seq'::regclass);

-- Additional utility tables
CREATE TABLE public.email_verification (
    id integer NOT NULL,
    user_id integer NOT NULL,
    email text NOT NULL,
    verification_code text NOT NULL,
    creation_date timestamp without time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.email_verification_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.email_verification_id_seq OWNED BY public.email_verification.id;
ALTER TABLE ONLY public.email_verification ALTER COLUMN id SET DEFAULT nextval('public.email_verification_id_seq'::regclass);

CREATE TABLE public.password_resets (
    id integer NOT NULL,
    reset_token text NOT NULL,
    creation_date timestamp without time zone DEFAULT now() NOT NULL,
    user_id integer NOT NULL
);

CREATE SEQUENCE public.password_resets_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.password_resets_id_seq OWNED BY public.password_resets.id;
ALTER TABLE ONLY public.password_resets ALTER COLUMN id SET DEFAULT nextval('public.password_resets_id_seq'::regclass);

CREATE TABLE public.registration_applications (
    id integer NOT NULL,
    user_id integer NOT NULL,
    answer text NOT NULL,
    admin_id integer,
    deny_reason text,
    creation_date timestamp without time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.registration_applications_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.registration_applications_id_seq OWNED BY public.registration_applications.id;
ALTER TABLE ONLY public.registration_applications ALTER COLUMN id SET DEFAULT nextval('public.registration_applications_id_seq'::regclass);

CREATE TABLE public.site_invite (
    id integer NOT NULL,
    verification_code text NOT NULL,
    created timestamp without time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.site_invite_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.site_invite_id_seq OWNED BY public.site_invite.id;
ALTER TABLE ONLY public.site_invite ALTER COLUMN id SET DEFAULT nextval('public.site_invite_id_seq'::regclass);

CREATE TABLE public.site_language (
    id integer NOT NULL,
    site_id integer NOT NULL,
    language_id integer NOT NULL
);

CREATE SEQUENCE public.site_language_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.site_language_id_seq OWNED BY public.site_language.id;
ALTER TABLE ONLY public.site_language ALTER COLUMN id SET DEFAULT nextval('public.site_language_id_seq'::regclass);

-- Media and upload tables
CREATE TABLE public.uploads (
    id integer NOT NULL,
    user_id integer NOT NULL,
    original_name text NOT NULL,
    file_name text NOT NULL,
    file_path text NOT NULL,
    upload_url text NOT NULL,
    creation_date timestamp without time zone DEFAULT now() NOT NULL,
    size bigint NOT NULL
);

CREATE SEQUENCE public.uploads_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.uploads_id_seq OWNED BY public.uploads.id;
ALTER TABLE ONLY public.uploads ALTER COLUMN id SET DEFAULT nextval('public.uploads_id_seq'::regclass);

CREATE TABLE public.stray_images (
    id integer NOT NULL,
    img_url text NOT NULL,
    creation_date timestamp without time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.stray_images_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.stray_images_id_seq OWNED BY public.stray_images.id;
ALTER TABLE ONLY public.stray_images ALTER COLUMN id SET DEFAULT nextval('public.stray_images_id_seq'::regclass);

-- Emoji tables
CREATE TABLE public.emoji (
    id integer NOT NULL,
    shortcode character varying(128) NOT NULL,
    image_url text NOT NULL,
    alt_text text NOT NULL,
    category text NOT NULL,
    creation_date timestamp without time zone DEFAULT now() NOT NULL,
    updated timestamp without time zone,
    board_id integer,
    created_by_user_id integer NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    usage_count integer DEFAULT 0 NOT NULL,
    emoji_scope character varying(10) NOT NULL
);

CREATE SEQUENCE public.emoji_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.emoji_id_seq OWNED BY public.emoji.id;
ALTER TABLE ONLY public.emoji ALTER COLUMN id SET DEFAULT nextval('public.emoji_id_seq'::regclass);

CREATE TABLE public.emoji_keyword (
    id integer NOT NULL,
    emoji_id integer NOT NULL,
    keyword character varying(128) NOT NULL
);

CREATE SEQUENCE public.emoji_keyword_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.emoji_keyword_id_seq OWNED BY public.emoji_keyword.id;
ALTER TABLE ONLY public.emoji_keyword ALTER COLUMN id SET DEFAULT nextval('public.emoji_keyword_id_seq'::regclass);

-- Rate limit and user management tables
CREATE TABLE public.local_site_rate_limit (
    id integer NOT NULL,
    local_site_id integer NOT NULL,
    message integer NOT NULL,
    message_per_second integer NOT NULL,
    post integer NOT NULL,
    post_per_second integer NOT NULL,
    register integer NOT NULL,
    register_per_second integer NOT NULL,
    image integer NOT NULL,
    image_per_second integer NOT NULL,
    comment integer NOT NULL,
    comment_per_second integer NOT NULL,
    search integer NOT NULL,
    search_per_second integer NOT NULL,
    creation_date timestamp without time zone DEFAULT now() NOT NULL,
    updated timestamp without time zone
);

CREATE SEQUENCE public.local_site_rate_limit_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.local_site_rate_limit_id_seq OWNED BY public.local_site_rate_limit.id;
ALTER TABLE ONLY public.local_site_rate_limit ALTER COLUMN id SET DEFAULT nextval('public.local_site_rate_limit_id_seq'::regclass);

CREATE TABLE public.user_ban (
    id integer NOT NULL,
    user_id integer NOT NULL,
    creation_date timestamp without time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.user_ban_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.user_ban_id_seq OWNED BY public.user_ban.id;
ALTER TABLE ONLY public.user_ban ALTER COLUMN id SET DEFAULT nextval('public.user_ban_id_seq'::regclass);

CREATE TABLE public.user_blocks (
    id integer NOT NULL,
    user_id integer NOT NULL,
    target_id integer NOT NULL,
    creation_date timestamp without time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.user_blocks_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.user_blocks_id_seq OWNED BY public.user_blocks.id;
ALTER TABLE ONLY public.user_blocks ALTER COLUMN id SET DEFAULT nextval('public.user_blocks_id_seq'::regclass);

CREATE TABLE public.user_board_blocks (
    id integer NOT NULL,
    user_id integer NOT NULL,
    board_id integer NOT NULL,
    creation_date timestamp without time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.user_board_blocks_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.user_board_blocks_id_seq OWNED BY public.user_board_blocks.id;
ALTER TABLE ONLY public.user_board_blocks ALTER COLUMN id SET DEFAULT nextval('public.user_board_blocks_id_seq'::regclass);

CREATE TABLE public.user_language (
    id integer NOT NULL,
    user_id integer NOT NULL,
    language_id integer NOT NULL
);

CREATE SEQUENCE public.user_language_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.user_language_id_seq OWNED BY public.user_language.id;
ALTER TABLE ONLY public.user_language ALTER COLUMN id SET DEFAULT nextval('public.user_language_id_seq'::regclass);

CREATE TABLE public.user_subscriber (
    id integer NOT NULL,
    user_id integer NOT NULL,
    subscriber_id integer NOT NULL,
    creation_date timestamp without time zone DEFAULT now() NOT NULL,
    pending boolean DEFAULT false NOT NULL
);

CREATE SEQUENCE public.user_subscriber_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.user_subscriber_id_seq OWNED BY public.user_subscriber.id;
ALTER TABLE ONLY public.user_subscriber ALTER COLUMN id SET DEFAULT nextval('public.user_subscriber_id_seq'::regclass);

-- There's a bug somewhere creating a 'relations' table - including to match exact database
CREATE TABLE public.relations (
    id integer NOT NULL
);

CREATE SEQUENCE public.relations_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.relations_id_seq OWNED BY public.relations.id;
ALTER TABLE ONLY public.relations ALTER COLUMN id SET DEFAULT nextval('public.relations_id_seq'::regclass);

-- Now add ALL primary key constraints
ALTER TABLE ONLY public.board_subscriber ADD CONSTRAINT board_subscriber_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.board_user_bans ADD CONSTRAINT board_user_bans_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.board_language ADD CONSTRAINT board_language_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.comment_saved ADD CONSTRAINT comment_saved_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.post_saved ADD CONSTRAINT post_saved_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.post_read ADD CONSTRAINT post_read_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.admin_ban_board ADD CONSTRAINT admin_ban_board_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.admin_purge_board ADD CONSTRAINT admin_purge_board_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.admin_purge_comment ADD CONSTRAINT admin_purge_comment_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.admin_purge_post ADD CONSTRAINT admin_purge_post_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.admin_purge_user ADD CONSTRAINT admin_purge_user_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.mod_add_admin ADD CONSTRAINT mod_add_admin_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.mod_add_board ADD CONSTRAINT mod_add_board_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.mod_add_board_mod ADD CONSTRAINT mod_add_board_mod_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.mod_ban ADD CONSTRAINT mod_ban_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.mod_ban_from_board ADD CONSTRAINT mod_ban_from_board_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.mod_feature_post ADD CONSTRAINT mod_feature_post_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.mod_hide_board ADD CONSTRAINT mod_hide_board_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.mod_lock_post ADD CONSTRAINT mod_lock_post_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.mod_remove_board ADD CONSTRAINT mod_remove_board_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.mod_remove_comment ADD CONSTRAINT mod_remove_comment_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.mod_remove_post ADD CONSTRAINT mod_remove_post_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.messages ADD CONSTRAINT messages_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.private_message ADD CONSTRAINT private_message_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.notifications ADD CONSTRAINT notifications_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.pm_notif ADD CONSTRAINT pm_notif_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.comment_report ADD CONSTRAINT comment_report_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.comment_reports ADD CONSTRAINT comment_reports_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.post_report ADD CONSTRAINT post_report_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.post_reports ADD CONSTRAINT post_reports_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.email_verification ADD CONSTRAINT email_verification_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.password_resets ADD CONSTRAINT password_resets_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.registration_applications ADD CONSTRAINT registration_applications_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.site_invite ADD CONSTRAINT site_invite_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.site_language ADD CONSTRAINT site_language_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.uploads ADD CONSTRAINT uploads_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.stray_images ADD CONSTRAINT stray_images_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.emoji ADD CONSTRAINT emoji_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.emoji_keyword ADD CONSTRAINT emoji_keyword_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.local_site_rate_limit ADD CONSTRAINT local_site_rate_limit_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.user_ban ADD CONSTRAINT user_ban_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.user_blocks ADD CONSTRAINT user_blocks_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.user_board_blocks ADD CONSTRAINT user_board_blocks_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.user_language ADD CONSTRAINT user_language_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.user_subscriber ADD CONSTRAINT user_subscriber_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.relations ADD CONSTRAINT relations_pkey PRIMARY KEY (id);

-- Add foreign key constraints
ALTER TABLE ONLY public.board_subscriber ADD CONSTRAINT board_subscriber_board_id_fkey FOREIGN KEY (board_id) REFERENCES public.boards(id) ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE ONLY public.board_subscriber ADD CONSTRAINT board_subscriber_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.board_user_bans ADD CONSTRAINT board_user_bans_board_id_fkey FOREIGN KEY (board_id) REFERENCES public.boards(id) ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE ONLY public.board_user_bans ADD CONSTRAINT board_user_bans_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.board_language ADD CONSTRAINT board_language_board_id_fkey FOREIGN KEY (board_id) REFERENCES public.boards(id) ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE ONLY public.board_language ADD CONSTRAINT board_language_language_id_fkey FOREIGN KEY (language_id) REFERENCES public.language(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.comment_saved ADD CONSTRAINT comment_saved_comment_id_fkey FOREIGN KEY (comment_id) REFERENCES public.comments(id) ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE ONLY public.comment_saved ADD CONSTRAINT comment_saved_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.post_saved ADD CONSTRAINT post_saved_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.posts(id) ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE ONLY public.post_saved ADD CONSTRAINT post_saved_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.post_read ADD CONSTRAINT post_read_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.posts(id) ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE ONLY public.post_read ADD CONSTRAINT post_read_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.admin_purge_comment ADD CONSTRAINT admin_purge_comment_admin_id_fkey FOREIGN KEY (admin_id) REFERENCES public.users(id) ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE ONLY public.admin_purge_comment ADD CONSTRAINT admin_purge_comment_comment_id_fkey FOREIGN KEY (comment_id) REFERENCES public.comments(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.admin_purge_post ADD CONSTRAINT admin_purge_post_admin_id_fkey FOREIGN KEY (admin_id) REFERENCES public.users(id) ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE ONLY public.admin_purge_post ADD CONSTRAINT admin_purge_post_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.posts(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.comment_reports ADD CONSTRAINT comment_reports_comment_id_fkey FOREIGN KEY (comment_id) REFERENCES public.comments(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.post_reports ADD CONSTRAINT post_reports_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.posts(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.email_verification ADD CONSTRAINT email_verification_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.password_resets ADD CONSTRAINT password_resets_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.uploads ADD CONSTRAINT uploads_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.emoji_keyword ADD CONSTRAINT emoji_keyword_emoji_id_fkey FOREIGN KEY (emoji_id) REFERENCES public.emoji(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.notifications ADD CONSTRAINT notifications_private_message_message_id_fkey FOREIGN KEY (message_id) REFERENCES public.private_message(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.pm_notif ADD CONSTRAINT pm_notif_private_message_pm_id_fkey FOREIGN KEY (pm_id) REFERENCES public.private_message(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.site_language ADD CONSTRAINT site_language_language_id_fkey FOREIGN KEY (language_id) REFERENCES public.language(id) ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE ONLY public.site_language ADD CONSTRAINT site_language_site_id_fkey FOREIGN KEY (site_id) REFERENCES public.site(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.user_board_blocks ADD CONSTRAINT user_board_blocks_board_id_fkey FOREIGN KEY (board_id) REFERENCES public.boards(id) ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE ONLY public.user_board_blocks ADD CONSTRAINT user_board_blocks_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.user_language ADD CONSTRAINT user_language_language_id_fkey FOREIGN KEY (language_id) REFERENCES public.language(id) ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE ONLY public.user_language ADD CONSTRAINT user_language_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON UPDATE CASCADE ON DELETE CASCADE;

-- Missing activity functions for scheduled tasks
CREATE OR REPLACE FUNCTION public.site_aggregates_activity(i text)
RETURNS int
LANGUAGE plpgsql
AS $$
DECLARE
   count_ integer;
BEGIN
  SELECT count(*)
  INTO count_
  FROM (
    SELECT c.creator_id FROM comments c
    INNER JOIN users u ON c.creator_id = u.id
    WHERE c.creation_date > ('now'::timestamp - i::interval)
    UNION
    SELECT p.creator_id FROM posts p
    INNER JOIN users u ON p.creator_id = u.id
    WHERE p.creation_date > ('now'::timestamp - i::interval)
  ) a;
  RETURN count_;
END;
$$;

CREATE OR REPLACE FUNCTION public.board_aggregates_activity(i text)
RETURNS table(count_ bigint, board_id_ integer)
LANGUAGE plpgsql
AS $$
BEGIN
  RETURN query
  SELECT count(*), board_id
  FROM (
    SELECT c.creator_id, p.board_id FROM comments c
    INNER JOIN posts p ON c.post_id = p.id
    WHERE c.creation_date > ('now'::timestamp - i::interval)
    UNION
    SELECT p.creator_id, p.board_id FROM posts p
    WHERE p.creation_date > ('now'::timestamp - i::interval)
  ) a
  GROUP BY board_id;
END;
$$;

-- Create only NEW triggers for tables created in this migration
CREATE TRIGGER board_aggregates_subscriber_count AFTER INSERT OR DELETE ON public.board_subscriber FOR EACH ROW EXECUTE FUNCTION public.board_aggregates_subscriber_count();