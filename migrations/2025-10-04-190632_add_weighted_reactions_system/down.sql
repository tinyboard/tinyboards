-- Drop board reaction settings
DROP TABLE IF EXISTS board_reaction_settings;

-- Drop trigger and function
DROP TRIGGER IF EXISTS reaction_aggregates_trigger ON reactions;
DROP FUNCTION IF EXISTS update_reaction_aggregates();

-- Drop reaction aggregates
DROP TABLE IF EXISTS reaction_aggregates;

-- Drop reactions
DROP TABLE IF EXISTS reactions;
