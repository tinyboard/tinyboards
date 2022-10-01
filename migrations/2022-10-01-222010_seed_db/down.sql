-- This file should undo anything in `up.sql`
DELETE FROM users WHERE username='$_account_1';
DELETE FROM boards WHERE board_name='general';
