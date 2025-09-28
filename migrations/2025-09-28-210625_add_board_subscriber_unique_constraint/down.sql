-- Remove the unique constraint on board_id and user_id combination
ALTER TABLE board_subscriber
DROP CONSTRAINT unique_board_user_subscription;
