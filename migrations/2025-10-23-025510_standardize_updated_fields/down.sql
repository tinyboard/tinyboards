-- Revert update field naming standardization

-- Flair aggregates
ALTER TABLE flair_aggregates RENAME COLUMN creation_date TO created_at;
ALTER TABLE flair_aggregates RENAME COLUMN updated TO updated_at;

-- Flair templates
ALTER TABLE flair_templates RENAME COLUMN updated TO updated_at;

-- Streams
ALTER TABLE streams RENAME COLUMN updated TO updated_at;
