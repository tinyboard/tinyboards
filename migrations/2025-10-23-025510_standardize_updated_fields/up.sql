-- Standardize update field naming to match existing convention (updated, not updated_at)
-- Also fix flair_aggregates created_at -> creation_date

-- Flair aggregates - rename both created_at and updated_at
ALTER TABLE flair_aggregates RENAME COLUMN created_at TO creation_date;
ALTER TABLE flair_aggregates RENAME COLUMN updated_at TO updated;

-- Flair templates - rename updated_at to updated
ALTER TABLE flair_templates RENAME COLUMN updated_at TO updated;

-- Streams - rename updated_at to updated
ALTER TABLE streams RENAME COLUMN updated_at TO updated;
