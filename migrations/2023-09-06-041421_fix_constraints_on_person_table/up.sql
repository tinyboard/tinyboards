ALTER TABLE person DROP CONSTRAINT idx_person_inbox_url;
ALTER TABLE person ADD CONSTRAINT apub_id_unique UNIQUE(actor_id);