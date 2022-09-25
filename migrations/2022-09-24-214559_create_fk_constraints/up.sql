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
-- chatbans constraints
ALTER TABLE IF EXISTS chatbans ADD CONSTRAINT fk_user_id
    FOREIGN KEY(user_id)
        REFERENCES users(id);
ALTER TABLE IF EXISTS chatbans ADD CONSTRAINT fk_board_id
    FOREIGN KEY(board_id)
        REFERENCES boards(id);
ALTER TABLE IF EXISTS chatbans ADD CONSTRAINT fk_banning_mod_id
    FOREIGN KEY(banning_mod_id)
        REFERENCES users(id);
-- contributors constraints
ALTER TABLE IF EXISTS contributors ADD CONSTRAINT fk_user_id
    FOREIGN KEY(user_id)
        REFERENCES users(id);
ALTER TABLE IF EXISTS contributors ADD CONSTRAINT fk_board_id
    FOREIGN KEY(board_id)
        REFERENCES boards(id);
ALTER TABLE IF EXISTS contributors ADD CONSTRAINT fk_approving_mod_id
    FOREIGN KEY(approving_mod_id)
        REFERENCES users(id);
-- postrels constraints
ALTER TABLE IF EXISTS postrels ADD CONSTRAINT fk_post_id
    FOREIGN KEY(post_id)
        REFERENCES submissions(id);
ALTER TABLE IF EXISTS postrels ADD CONSTRAINT fk_board_id
    FOREIGN KEY(board_id)
        REFERENCES boards(id);
-- boardblocks constraints
ALTER TABLE IF EXISTS boardblocks ADD CONSTRAINT fk_user_id
    FOREIGN KEY(user_id)
        REFERENCES users(id);
ALTER TABLE IF EXISTS boardblocks ADD CONSTRAINT fk_board_id
    FOREIGN KEY(board_id)
        REFERENCES boards(id);
-- boards constraints
ALTER TABLE IF EXISTS boards ADD CONSTRAINT fk_creator_id
    FOREIGN KEY(creator_id)
        REFERENCES users(id);
ALTER TABLE IF EXISTS boards ADD CONSTRAINT fk_subcat_id
    FOREIGN KEY(subcat_id)
        REFERENCES subcategories(id);
-- subcategories constraints
ALTER TABLE IF EXISTS subcategories ADD CONSTRAINT fk_cat_id
    FOREIGN KEY(cat_id)
        REFERENCES categories(id);
-- oauth_apps constraints
ALTER TABLE IF EXISTS oauth_apps ADD CONSTRAINT fk_author_id
    FOREIGN KEY(author_id)
        REFERENCES users(id);
-- client_auths constraints
ALTER TABLE IF EXISTS client_auths ADD CONSTRAINT fk_user_id
    FOREIGN KEY(user_id)
        REFERENCES users(id);
ALTER TABLE IF EXISTS client_auths ADD CONSTRAINT fk_oauth_client
    FOREIGN KEY(oauth_client)
        REFERENCES oauth_apps(id);
-- comments constraints
ALTER TABLE IF EXISTS comments ADD CONSTRAINT fk_author_id
    FOREIGN KEY(author_id)
        REFERENCES users(id);
ALTER TABLE IF EXISTS comments ADD CONSTRAINT fk_parent_submission
    FOREIGN KEY(parent_submission)
        REFERENCES submissions(id);
ALTER TABLE IF EXISTS comments ADD CONSTRAINT fk_gm_distinguish
    FOREIGN KEY(gm_distinguish)
        REFERENCES boards(id);
ALTER TABLE IF EXISTS comments ADD CONSTRAINT fk_parent_comment_id
    FOREIGN KEY(parent_comment_id)
        REFERENCES comments(id);
ALTER TABLE IF EXISTS comments ADD CONSTRAINT fk_original_board_id
    FOREIGN KEY(original_board_id)
        REFERENCES boards(id);
ALTER TABLE IF EXISTS comments ADD CONSTRAINT fk_app_id
    FOREIGN KEY(app_id)
        REFERENCES oauth_apps(id);
-- notifications constraints
ALTER TABLE IF EXISTS notifications ADD CONSTRAINT fk_user_id
    FOREIGN KEY(user_id)
        REFERENCES users(id);
ALTER TABLE IF EXISTS notifications ADD CONSTRAINT fk_comment_id
    FOREIGN KEY(comment_id)
        REFERENCES comments(id);
ALTER TABLE IF EXISTS notifications ADD CONSTRAINT fk_submission_id 
    FOREIGN KEY(submission_id)
        REFERENCES submissions(id);
-- flags constraints
ALTER TABLE IF EXISTS flags ADD CONSTRAINT fk_post_id
    FOREIGN KEY(post_id)
        REFERENCES submissions(id);
ALTER TABLE IF EXISTS flags ADD CONSTRAINT fk_user_id
    FOREIGN KEY(user_id)
        REFERENCES users(id);
-- commentflags constraints
ALTER TABLE IF EXISTS commentflags ADD CONSTRAINT fk_user_id
    FOREIGN KEY(user_id)
        REFERENCES users(id);
ALTER TABLE IF EXISTS commentflags ADD CONSTRAINT fk_comment_id
    FOREIGN KEY(comment_id)
        REFERENCES comments(id);
-- reports constraints
ALTER TABLE IF EXISTS reports ADD CONSTRAINT fk_post_id
    FOREIGN KEY(post_id)
        REFERENCES submissions(id);
ALTER TABLE IF EXISTS reports ADD CONSTRAINT fk_user_id
    FOREIGN KEY(user_id)
        REFERENCES users(id);
-- ips constraints
ALTER TABLE IF EXISTS ips ADD CONSTRAINT fk_banned_by
    FOREIGN KEY(banned_by)
        REFERENCES users(id);
-- useragents constraints
ALTER TABLE IF EXISTS useragents ADD CONSTRAINT fk_banned_by
    FOREIGN KEY(banned_by)
        REFERENCES users(id);
-- lodges constraints
ALTER TABLE IF EXISTS lodges ADD CONSTRAINT fk_user_id
    FOREIGN KEY(user_id)
        REFERENCES users(id);
ALTER TABLE IF EXISTS lodges ADD CONSTRAINT fk_board_id
    FOREIGN KEY(board_id)
        REFERENCES boards(id);