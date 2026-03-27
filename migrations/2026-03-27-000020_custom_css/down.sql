ALTER TABLE boards DROP COLUMN IF EXISTS custom_css;

ALTER TABLE site
    DROP COLUMN IF EXISTS custom_css_enabled,
    DROP COLUMN IF EXISTS custom_css;
