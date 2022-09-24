-- submissions constraints
ALTER TABLE IF EXISTS submissions ADD CONSTRAINT fk_author_id
    FOREIGN KEY(author_id)
        REFERENCES users(id);
ALTER TABLE IF EXISTS submissions ADD CONSTRAINT fk_repost_id
    FOREIGN KEY(repost_id)
        REFERENCES submissions(id);
ALTER TABLE IF EXISTS submissions ADD CONSTRAINT fk_gm_distinguish
    FOREIGN KEY(gm_distinguish)
        REFERENCES boards(id);
ALTER TABLE IF EXISTS submissions ADD CONSTRAINT fk_domain_ref
    FOREIGN KEY(domain_ref)
        REFERENCES domains(id);
ALTER TABLE IF EXISTS submissions ADD CONSTRAINT fk_is_approved
    FOREIGN KEY(is_approved)
        REFERENCES users(id);
ALTER TABLE IF EXISTS submissions ADD CONSTRAINT fk_board_id
    FOREIGN KEY(board_id)
        REFERENCES boards(id);
ALTER TABLE IF EXISTS submissions ADD CONSTRAINT fk_original_board_id
    FOREIGN KEY(board_id)
        REFERENCES boards(id);
ALTER TABLE IF EXISTS submissions ADD CONSTRAINT fk_app_id
    FOREIGN KEY(app_id)
        REFERENCES oauth_apps(id);
-- save_relationship constraints
ALTER TABLE IF EXISTS save_relationship ADD CONSTRAINT fk_user_id
    FOREIGN KEY(user_id)
        REFERENCES users(id);
-- alts constraints
ALTER TABLE IF EXISTS alts ADD CONSTRAINT fk_user1
    FOREIGN KEY(user1)
        REFERENCES users(id);
ALTER TABLE IF EXISTS alts ADD CONSTRAINT fk_user2
    FOREIGN KEY(user2) 
        REFERENCES users(id);
-- badges constraints
ALTER TABLE IF EXISTS badges ADD CONSTRAINT fk_badge_id
    FOREIGN KEY(badge_id)
        REFERENCES badge_defs(id);
ALTER TABLE IF EXISTS badges ADD CONSTRAINT fk_user_id
    FOREIGN KEY(user_id)
        REFERENCES users(id);
-- mods constraints
ALTER TABLE IF EXISTS mods ADD CONSTRAINT fk_user_id
    FOREIGN KEY(user_id)
        REFERENCES users(id);
ALTER TABLE IF EXISTS mods ADD CONSTRAINT fk_board_id
    FOREIGN KEY(board_id)
        REFERENCES boards(id);
-- bans constraints
ALTER TABLE IF EXISTS bans ADD CONSTRAINT fk_user_id
    FOREIGN KEY(user_id)
        REFERENCES users(id);
ALTER TABLE IF EXISTS bans ADD CONSTRAINT fk_board_id
    FOREIGN KEY(board_id)
        REFERENCES boards(id);
ALTER TABLE IF EXISTS bans ADD CONSTRAINT fk_banning_mod_id
    FOREIGN KEY(banning_mod_id)
        REFERENCES users(id);