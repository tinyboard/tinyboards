-- Rollback for social and triggers migration
-- This removes all social features, admin tables, messaging, and complex triggers

-- Drop triggers created in this migration only
DROP TRIGGER IF EXISTS board_aggregates_subscriber_count ON public.board_subscriber;

-- Drop activity functions
DROP FUNCTION IF EXISTS public.site_aggregates_activity(text);
DROP FUNCTION IF EXISTS public.board_aggregates_activity(text);

-- Drop all tables created in the 3rd migration (in correct order to handle dependencies)
DROP TABLE IF EXISTS public.board_subscriber CASCADE;
DROP TABLE IF EXISTS public.board_user_bans CASCADE;
DROP TABLE IF EXISTS public.board_language CASCADE;
DROP TABLE IF EXISTS public.comment_saved CASCADE;
DROP TABLE IF EXISTS public.post_saved CASCADE;
DROP TABLE IF EXISTS public.post_read CASCADE;
DROP TABLE IF EXISTS public.admin_ban_board CASCADE;
DROP TABLE IF EXISTS public.admin_purge_board CASCADE;
DROP TABLE IF EXISTS public.admin_purge_comment CASCADE;
DROP TABLE IF EXISTS public.admin_purge_post CASCADE;
DROP TABLE IF EXISTS public.admin_purge_user CASCADE;
DROP TABLE IF EXISTS public.mod_add_admin CASCADE;
DROP TABLE IF EXISTS public.mod_add_board CASCADE;
DROP TABLE IF EXISTS public.mod_add_board_mod CASCADE;
DROP TABLE IF EXISTS public.mod_ban CASCADE;
DROP TABLE IF EXISTS public.mod_ban_from_board CASCADE;
DROP TABLE IF EXISTS public.mod_feature_post CASCADE;
DROP TABLE IF EXISTS public.mod_hide_board CASCADE;
DROP TABLE IF EXISTS public.mod_lock_post CASCADE;
DROP TABLE IF EXISTS public.mod_remove_board CASCADE;
DROP TABLE IF EXISTS public.mod_remove_comment CASCADE;
DROP TABLE IF EXISTS public.mod_remove_post CASCADE;
DROP TABLE IF EXISTS public.messages CASCADE;
DROP TABLE IF EXISTS public.private_message CASCADE;
DROP TABLE IF EXISTS public.notifications CASCADE;
DROP TABLE IF EXISTS public.pm_notif CASCADE;
DROP TABLE IF EXISTS public.comment_report CASCADE;
DROP TABLE IF EXISTS public.comment_reports CASCADE;
DROP TABLE IF EXISTS public.post_report CASCADE;
DROP TABLE IF EXISTS public.post_reports CASCADE;
DROP TABLE IF EXISTS public.email_verification CASCADE;
DROP TABLE IF EXISTS public.password_resets CASCADE;
DROP TABLE IF EXISTS public.registration_applications CASCADE;
DROP TABLE IF EXISTS public.site_invite CASCADE;
DROP TABLE IF EXISTS public.site_language CASCADE;
DROP TABLE IF EXISTS public.uploads CASCADE;
DROP TABLE IF EXISTS public.stray_images CASCADE;
DROP TABLE IF EXISTS public.emoji CASCADE;
DROP TABLE IF EXISTS public.emoji_keyword CASCADE;
DROP TABLE IF EXISTS public.local_site_rate_limit CASCADE;
DROP TABLE IF EXISTS public.user_ban CASCADE;
DROP TABLE IF EXISTS public.user_blocks CASCADE;
DROP TABLE IF EXISTS public.user_board_blocks CASCADE;
DROP TABLE IF EXISTS public.user_language CASCADE;
DROP TABLE IF EXISTS public.user_subscriber CASCADE;
DROP TABLE IF EXISTS public.relations CASCADE;

-- Drop all sequences created in the 3rd migration
DROP SEQUENCE IF EXISTS public.board_subscriber_id_seq;
DROP SEQUENCE IF EXISTS public.board_user_bans_id_seq;
DROP SEQUENCE IF EXISTS public.board_language_id_seq;
DROP SEQUENCE IF EXISTS public.comment_saved_id_seq;
DROP SEQUENCE IF EXISTS public.post_saved_id_seq;
DROP SEQUENCE IF EXISTS public.post_read_id_seq;
DROP SEQUENCE IF EXISTS public.admin_ban_board_id_seq;
DROP SEQUENCE IF EXISTS public.admin_purge_board_id_seq;
DROP SEQUENCE IF EXISTS public.admin_purge_comment_id_seq;
DROP SEQUENCE IF EXISTS public.admin_purge_post_id_seq;
DROP SEQUENCE IF EXISTS public.admin_purge_user_id_seq;
DROP SEQUENCE IF EXISTS public.mod_add_admin_id_seq;
DROP SEQUENCE IF EXISTS public.mod_add_board_id_seq;
DROP SEQUENCE IF EXISTS public.mod_add_board_mod_id_seq;
DROP SEQUENCE IF EXISTS public.mod_ban_id_seq;
DROP SEQUENCE IF EXISTS public.mod_ban_from_board_id_seq;
DROP SEQUENCE IF EXISTS public.mod_feature_post_id_seq;
DROP SEQUENCE IF EXISTS public.mod_hide_board_id_seq;
DROP SEQUENCE IF EXISTS public.mod_lock_post_id_seq;
DROP SEQUENCE IF EXISTS public.mod_remove_board_id_seq;
DROP SEQUENCE IF EXISTS public.mod_remove_comment_id_seq;
DROP SEQUENCE IF EXISTS public.mod_remove_post_id_seq;
DROP SEQUENCE IF EXISTS public.messages_id_seq;
DROP SEQUENCE IF EXISTS public.private_message_id_seq;
DROP SEQUENCE IF EXISTS public.notifications_id_seq;
DROP SEQUENCE IF EXISTS public.pm_notif_id_seq;
DROP SEQUENCE IF EXISTS public.comment_report_id_seq;
DROP SEQUENCE IF EXISTS public.comment_reports_id_seq;
DROP SEQUENCE IF EXISTS public.post_report_id_seq;
DROP SEQUENCE IF EXISTS public.post_reports_id_seq;
DROP SEQUENCE IF EXISTS public.email_verification_id_seq;
DROP SEQUENCE IF EXISTS public.password_resets_id_seq;
DROP SEQUENCE IF EXISTS public.registration_applications_id_seq;
DROP SEQUENCE IF EXISTS public.site_invite_id_seq;
DROP SEQUENCE IF EXISTS public.site_language_id_seq;
DROP SEQUENCE IF EXISTS public.uploads_id_seq;
DROP SEQUENCE IF EXISTS public.stray_images_id_seq;
DROP SEQUENCE IF EXISTS public.emoji_id_seq;
DROP SEQUENCE IF EXISTS public.emoji_keyword_id_seq;
DROP SEQUENCE IF EXISTS public.local_site_rate_limit_id_seq;
DROP SEQUENCE IF EXISTS public.user_ban_id_seq;
DROP SEQUENCE IF EXISTS public.user_blocks_id_seq;
DROP SEQUENCE IF EXISTS public.user_board_blocks_id_seq;
DROP SEQUENCE IF EXISTS public.user_language_id_seq;
DROP SEQUENCE IF EXISTS public.user_subscriber_id_seq;
DROP SEQUENCE IF EXISTS public.relations_id_seq;

-- Drop functions added in this migration (if any)
DROP FUNCTION IF EXISTS public.hot_rank(score numeric, published timestamp without time zone);
DROP FUNCTION IF EXISTS public.hot_rank(score numeric, published timestamp without time zone, now timestamp without time zone);
DROP FUNCTION IF EXISTS public.controversy_rank(upvotes numeric, downvotes numeric, published timestamp without time zone);
DROP FUNCTION IF EXISTS public.scaled_rank(score numeric, published timestamp without time zone, users_active_month numeric);
DROP FUNCTION IF EXISTS public.generate_unique_changeme();