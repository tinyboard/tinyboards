-- Add unique constraint on board_id and user_id combination
ALTER TABLE board_subscriber
ADD CONSTRAINT unique_board_user_subscription
UNIQUE (board_id, user_id);
