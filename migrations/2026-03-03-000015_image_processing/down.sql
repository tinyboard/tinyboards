ALTER TABLE site DROP COLUMN IF EXISTS image_strip_exif;
ALTER TABLE site DROP COLUMN IF EXISTS image_convert_to_webp;
ALTER TABLE site DROP COLUMN IF EXISTS image_webp_quality;
ALTER TABLE site DROP COLUMN IF EXISTS image_thumbnail_width;
ALTER TABLE site DROP COLUMN IF EXISTS image_max_height;
ALTER TABLE site DROP COLUMN IF EXISTS image_max_width;

DROP INDEX IF EXISTS idx_uploads_processing_status;
ALTER TABLE uploads DROP COLUMN IF EXISTS processing_status;
ALTER TABLE uploads DROP COLUMN IF EXISTS optimized_url;
ALTER TABLE uploads DROP COLUMN IF EXISTS thumbnail_url;
