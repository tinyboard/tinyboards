-- Take care of inserting required stuff
-- This can be deleted by the BE code after the admin account is created during onboarding I think
INSERT INTO users (id, username, passhash, created_utc) VALUES (
    1,
    '$_account_1',
    'x',
    (SELECT extract(epoch from now() at time zone 'utc'))
);

-- category stuff (do we need this?)
INSERT INTO public.categories VALUES (1, '', '', '', '', true, false);
INSERT INTO public.subcategories VALUES (1, 1, '', '', true);

INSERT INTO boards (board_name, created_utc, creator_id) VALUES (
    'general',
    (SELECT extract(epoch from now() at time zone 'utc')),
    1
);
