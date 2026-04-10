-- Add reaction_emojis column to board_reaction_settings
-- Stores the list of emojis shown in the quick-reaction picker for a board.
-- Empty array means "use site defaults" (the 6 hardcoded unicode emojis).
-- Each entry: {"type":"unicode","value":"👍"} or {"type":"custom","shortcode":"party_parrot","imageUrl":"https://..."}

ALTER TABLE board_reaction_settings
  ADD COLUMN reaction_emojis JSONB NOT NULL DEFAULT '[]';
