DROP FUNCTION IF EXISTS site_aggregates_activity(TEXT) CASCADE;
DROP FUNCTION IF EXISTS board_aggregates_activity(TEXT) CASCADE;

DROP TRIGGER IF EXISTS site_aggregates_user ON users;
DROP TRIGGER IF EXISTS site_aggregates_board ON boards;
DROP TRIGGER IF EXISTS site_aggregates_comment ON comments;
DROP TRIGGER IF EXISTS site_aggregates_post ON posts;
DROP FUNCTION IF EXISTS trg_site_aggregates_user() CASCADE;
DROP FUNCTION IF EXISTS trg_site_aggregates_board() CASCADE;
DROP FUNCTION IF EXISTS trg_site_aggregates_comment() CASCADE;
DROP FUNCTION IF EXISTS trg_site_aggregates_post() CASCADE;

DROP TRIGGER IF EXISTS user_aggregates_comment_count ON comments;
DROP FUNCTION IF EXISTS trg_user_aggregates_comment_count() CASCADE;
DROP TRIGGER IF EXISTS user_aggregates_post_count ON posts;
DROP FUNCTION IF EXISTS trg_user_aggregates_post_count() CASCADE;

DROP TRIGGER IF EXISTS comment_vote_aggregates ON comment_votes;
DROP FUNCTION IF EXISTS trg_comment_vote_aggregates() CASCADE;
DROP TRIGGER IF EXISTS post_vote_aggregates ON post_votes;
DROP FUNCTION IF EXISTS trg_post_vote_aggregates() CASCADE;

DROP TRIGGER IF EXISTS comment_count_on_comment ON comments;
DROP FUNCTION IF EXISTS trg_comment_count_on_comment() CASCADE;
DROP TRIGGER IF EXISTS board_aggregates_post_count ON posts;
DROP FUNCTION IF EXISTS trg_board_aggregates_post_count() CASCADE;

DROP TRIGGER IF EXISTS user_aggregates_on_user ON users;
DROP FUNCTION IF EXISTS trg_user_aggregates_on_user() CASCADE;
DROP TRIGGER IF EXISTS board_aggregates_on_board ON boards;
DROP FUNCTION IF EXISTS trg_board_aggregates_on_board() CASCADE;
DROP TRIGGER IF EXISTS comment_aggregates_on_comment ON comments;
DROP FUNCTION IF EXISTS trg_comment_aggregates_on_comment() CASCADE;
DROP TRIGGER IF EXISTS post_aggregates_on_post ON posts;
DROP FUNCTION IF EXISTS trg_post_aggregates_on_post() CASCADE;

DROP TABLE IF EXISTS site_aggregates CASCADE;
DROP TABLE IF EXISTS user_aggregates CASCADE;
DROP TABLE IF EXISTS board_aggregates CASCADE;
DROP TABLE IF EXISTS comment_aggregates CASCADE;
DROP TABLE IF EXISTS post_aggregates CASCADE;
