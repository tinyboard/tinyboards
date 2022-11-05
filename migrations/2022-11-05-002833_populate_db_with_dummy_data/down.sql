truncate table comment_vote cascade;
truncate table comment cascade;
truncate table post_vote cascade;
truncate table post cascade;

delete from user_ where name in ('elon_m', 'schwab_dawg69', 'admiralmeta5');