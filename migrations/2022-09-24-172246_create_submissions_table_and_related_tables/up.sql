CREATE TABLE boards(
    id SERIAL PRIMARY KEY,
    board_name VARCHAR NOT NULL,
    created_utc INTEGER DEFAULT 0,
    board_description VARCHAR DEFAULT NULL,
    board_description_html VARCHAR DEFAULT NULL,
    over_18 BOOLEAN DEFAULT false,
    is_nsfl BOOLEAN DEFAULT false,
    is_banned BOOLEAN DEFAULT false,
    has_banner BOOLEAN DEFAULT false,
    has_profile BOOLEAN DEFAULT false,
    creator_id INTEGER NOT NULL,
    CONSTRAINT fk_creator_id
        FOREIGN KEY(creator_id)
            REFERENCES users(id)
         
);

CREATE TABLE badlinks(
    id SERIAL PRIMARY KEY,
    reason INTEGER NOT NULL,
    link VARCHAR(512) NOT NULL,
    autoban BOOLEAN DEFAULT false
);

CREATE TABLE domains(
    id SERIAL PRIMARY KEY,
    domain VARCHAR NOT NULL,
    can_submit BOOLEAN DEFAULT true,
    can_comment BOOLEAN DEFAULT true,
    reason INTEGER DEFAULT 0,
    show_thumbnail BOOLEAN DEFAULT false,
    embed_function VARCHAR(64) DEFAULT NULL,
    embed_template VARCHAR(32) DEFAULT NULL
);

CREATE TABLE oauth_apps(
    id SERIAL PRIMARY KEY
);

CREATE TABLE submissions(
    id BIGSERIAL PRIMARY KEY,
    author_id INTEGER NOT NULL,
    repost_id INTEGER DEFAULT 0,
    edited_utc INTEGER DEFAULT 0,
    created_utc INTEGER DEFAULT 0,
    is_banned BOOLEAN DEFAULT false,
    deleted_utc INTEGER DEFAULT 0,
    purged_utc INTEGER DEFAULT 0,
    distinguish_level SMALLINT DEFAULT 0,
    gm_distinguish SMALLINT DEFAULT 0,
    created_str VARCHAR(255) DEFAULT NULL,
    stickied BOOLEAN DEFAULT false,
    domain_ref INTEGER DEFAULT 0,


    CONSTRAINT fk_author_id
        FOREIGN KEY(author_id)
            REFERENCES users(id),
    CONSTRAINT fk_repost_id
        FOREIGN KEY(repost_id)
            REFERENCES submissions(id),
    CONSTRAINT fk_gm_distinguish
        FOREIGN KEY(gm_distinguish)
            REFERENCES boards(id),
    CONSTRAINT fk_domain_ref
        FOREIGN KEY(domain_ref)
            REFERENCES domains(id)
);