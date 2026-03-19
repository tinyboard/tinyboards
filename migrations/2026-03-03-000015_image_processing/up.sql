-- Add image processing columns to the uploads table
ALTER TABLE uploads ADD COLUMN thumbnail_url TEXT;
ALTER TABLE uploads ADD COLUMN optimized_url TEXT;
-- processing_status tracks the state of the image optimization pipeline.
-- Currently, processing is synchronous (completes before the upload response),
-- so rows are immediately set to 'complete'. The 'pending' and 'failed' states
-- exist to support a future async processing flow where uploads are optimized
-- in a background task after the initial upload completes.
ALTER TABLE uploads ADD COLUMN processing_status VARCHAR(20) NOT NULL DEFAULT 'pending'
    CHECK (processing_status IN ('pending', 'complete', 'failed'));

CREATE INDEX idx_uploads_processing_status ON uploads (processing_status)
    WHERE processing_status = 'pending';

-- Add image processing settings to the site table
ALTER TABLE site ADD COLUMN image_max_width INTEGER NOT NULL DEFAULT 4096;
ALTER TABLE site ADD COLUMN image_max_height INTEGER NOT NULL DEFAULT 4096;
ALTER TABLE site ADD COLUMN image_thumbnail_width INTEGER NOT NULL DEFAULT 300;
ALTER TABLE site ADD COLUMN image_webp_quality INTEGER NOT NULL DEFAULT 85;
ALTER TABLE site ADD COLUMN image_convert_to_webp BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE site ADD COLUMN image_strip_exif BOOLEAN NOT NULL DEFAULT true;
