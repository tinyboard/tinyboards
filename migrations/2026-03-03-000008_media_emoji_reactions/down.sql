DROP TRIGGER IF EXISTS reaction_aggregates_on_reaction ON reactions;
DROP FUNCTION IF EXISTS trg_reaction_aggregates() CASCADE;

DROP TABLE IF EXISTS board_reaction_settings CASCADE;
DROP TABLE IF EXISTS reaction_aggregates CASCADE;
DROP TABLE IF EXISTS reactions CASCADE;
DROP TABLE IF EXISTS emoji_keywords CASCADE;
DROP TABLE IF EXISTS emoji CASCADE;
DROP TABLE IF EXISTS content_uploads CASCADE;
DROP TABLE IF EXISTS uploads CASCADE;
