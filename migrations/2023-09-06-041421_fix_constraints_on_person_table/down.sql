ALTER TABLE person ADD CONSTRAINT idx_person_inbox_url UNIQUE(inbox_url);
ALTER TABLE person DROP CONSTRAINT apub_id_unique;