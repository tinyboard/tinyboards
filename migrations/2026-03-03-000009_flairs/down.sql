DROP TRIGGER IF EXISTS flair_usage_on_user_flair ON user_flairs;
DROP FUNCTION IF EXISTS trg_flair_usage_user() CASCADE;
DROP TRIGGER IF EXISTS flair_usage_on_post_flair ON post_flairs;
DROP FUNCTION IF EXISTS trg_flair_usage_post() CASCADE;
DROP TRIGGER IF EXISTS flair_aggregates_on_template ON flair_templates;
DROP FUNCTION IF EXISTS trg_flair_aggregates_on_template() CASCADE;

DROP TABLE IF EXISTS flair_aggregates CASCADE;
DROP TABLE IF EXISTS user_flair_filters CASCADE;
DROP TABLE IF EXISTS user_flairs CASCADE;
DROP TABLE IF EXISTS post_flairs CASCADE;
DROP TABLE IF EXISTS flair_templates CASCADE;
DROP TABLE IF EXISTS flair_categories CASCADE;
