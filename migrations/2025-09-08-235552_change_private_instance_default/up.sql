-- Change private_instance column default to true (private by default)
ALTER TABLE local_site ALTER COLUMN private_instance SET DEFAULT true;