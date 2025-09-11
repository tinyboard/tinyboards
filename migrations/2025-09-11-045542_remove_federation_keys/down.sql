-- Add back federation keys (though they probably won't be used)
ALTER TABLE person ADD COLUMN public_key TEXT;
ALTER TABLE person ADD COLUMN private_key TEXT;

ALTER TABLE boards ADD COLUMN public_key TEXT;
ALTER TABLE boards ADD COLUMN private_key TEXT;

ALTER TABLE site ADD COLUMN public_key TEXT;
ALTER TABLE site ADD COLUMN private_key TEXT;