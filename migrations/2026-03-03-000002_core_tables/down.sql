DROP TRIGGER IF EXISTS board_moderators_invite_accepted ON board_moderators;
DROP TRIGGER IF EXISTS board_moderators_delete_rerank ON board_moderators;
DROP TRIGGER IF EXISTS board_moderators_insert_rank ON board_moderators;
DROP FUNCTION IF EXISTS board_moderators_accept_invite();
DROP FUNCTION IF EXISTS board_moderators_rerank();
DROP FUNCTION IF EXISTS board_moderators_set_rank();

DROP TABLE IF EXISTS board_moderators CASCADE;
DROP TABLE IF EXISTS boards CASCADE;
DROP TABLE IF EXISTS users CASCADE;
DROP TABLE IF EXISTS site CASCADE;
