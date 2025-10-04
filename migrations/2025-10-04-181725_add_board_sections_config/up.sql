-- Add flexible section configuration using bit flags
ALTER TABLE boards
ADD COLUMN section_config INTEGER NOT NULL DEFAULT 1;

COMMENT ON COLUMN boards.section_config IS
  'Bit flags for enabled sections: 1=Feed, 2=Threads, 4=Wiki, 8=Gallery, 16=Events';

-- Index for efficient filtering
CREATE INDEX idx_boards_section_config ON boards(section_config);

-- Ensure at least one section enabled
ALTER TABLE boards
ADD CONSTRAINT boards_section_config_not_zero
CHECK (section_config > 0);

-- Helper functions
CREATE OR REPLACE FUNCTION board_has_section(config INTEGER, section_flag INTEGER)
RETURNS BOOLEAN AS $$
BEGIN
    RETURN (config & section_flag) = section_flag;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

CREATE OR REPLACE FUNCTION enable_board_section(config INTEGER, section_flag INTEGER)
RETURNS INTEGER AS $$
BEGIN
    RETURN config | section_flag;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

CREATE OR REPLACE FUNCTION disable_board_section(config INTEGER, section_flag INTEGER)
RETURNS INTEGER AS $$
BEGIN
    RETURN config & ~section_flag;
END;
$$ LANGUAGE plpgsql IMMUTABLE;
