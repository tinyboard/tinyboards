-- Remove content settings from local_site table

ALTER TABLE local_site DROP COLUMN IF EXISTS allowed_post_types;
ALTER TABLE local_site DROP COLUMN IF EXISTS enable_nsfw_tagging;
ALTER TABLE local_site DROP COLUMN IF EXISTS word_filter_enabled;
ALTER TABLE local_site DROP COLUMN IF EXISTS filtered_words;
ALTER TABLE local_site DROP COLUMN IF EXISTS word_filter_applies_to_posts;
ALTER TABLE local_site DROP COLUMN IF EXISTS word_filter_applies_to_comments;
ALTER TABLE local_site DROP COLUMN IF EXISTS word_filter_applies_to_usernames;
ALTER TABLE local_site DROP COLUMN IF EXISTS link_filter_enabled;
ALTER TABLE local_site DROP COLUMN IF EXISTS banned_domains;
ALTER TABLE local_site DROP COLUMN IF EXISTS approved_image_hosts;
ALTER TABLE local_site DROP COLUMN IF EXISTS image_embed_hosts_only;