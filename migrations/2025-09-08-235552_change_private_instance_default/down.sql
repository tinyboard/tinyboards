-- Revert private_instance column default back to false
ALTER TABLE local_site ALTER COLUMN private_instance SET DEFAULT false;