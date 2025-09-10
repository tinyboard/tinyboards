-- Add content settings to local_site table

-- Add allowed post types (JSON array of allowed types: "text", "image", "link", "video")
ALTER TABLE local_site ADD COLUMN allowed_post_types TEXT DEFAULT '["text", "image", "link"]';

-- Add enable_nsfw_tagging setting (separate from enable_nsfw which controls if NSFW content is allowed at all)
ALTER TABLE local_site ADD COLUMN enable_nsfw_tagging BOOLEAN DEFAULT true;

-- Add word filter settings
ALTER TABLE local_site ADD COLUMN word_filter_enabled BOOLEAN DEFAULT false;
ALTER TABLE local_site ADD COLUMN filtered_words TEXT DEFAULT '[]'; -- JSON array of filtered words
ALTER TABLE local_site ADD COLUMN word_filter_applies_to_posts BOOLEAN DEFAULT true;
ALTER TABLE local_site ADD COLUMN word_filter_applies_to_comments BOOLEAN DEFAULT true;
ALTER TABLE local_site ADD COLUMN word_filter_applies_to_usernames BOOLEAN DEFAULT false;

-- Add link filter settings
ALTER TABLE local_site ADD COLUMN link_filter_enabled BOOLEAN DEFAULT false;
ALTER TABLE local_site ADD COLUMN banned_domains TEXT DEFAULT '[]'; -- JSON array of banned domains
ALTER TABLE local_site ADD COLUMN approved_image_hosts TEXT DEFAULT '["imgur.com", "i.imgur.com"]'; -- JSON array of approved image hosts
ALTER TABLE local_site ADD COLUMN image_embed_hosts_only BOOLEAN DEFAULT false; -- Only allow image embeds from approved hosts