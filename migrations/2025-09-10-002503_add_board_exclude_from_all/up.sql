-- Add exclude_from_all column to boards table
-- This will allow admins to exclude specific boards from the global feed (/all)

ALTER TABLE boards ADD COLUMN exclude_from_all BOOLEAN NOT NULL DEFAULT false;