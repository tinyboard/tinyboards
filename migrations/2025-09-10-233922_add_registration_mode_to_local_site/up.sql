-- Add registration_mode column to local_site table
ALTER TABLE local_site 
ADD COLUMN registration_mode VARCHAR NOT NULL DEFAULT 'Open';

-- Set initial registration_mode values based on existing boolean columns
UPDATE local_site SET registration_mode = 
CASE 
    WHEN require_application = true THEN 'RequireApplication'
    WHEN invite_only = true THEN 'InviteOnlyAdmin'
    WHEN require_email_verification = true AND open_registration = true THEN 'OpenWithEmailVerification'
    WHEN open_registration = true THEN 'Open'
    ELSE 'Closed'
END;