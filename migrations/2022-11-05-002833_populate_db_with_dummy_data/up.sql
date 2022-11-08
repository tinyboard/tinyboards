-- create dummy users
insert into user_ (name, passhash, email) values ('elon_m', 'letthatsinkin', 'elon@twitter.com');
insert into user_ (name, passhash, email) values ('schwab_dawg69', 'youvillownnothing', 'eatingthebugs@evil.illuminati');
insert into user_ (name, passhash, email) values ('admiralmeta5', 'totallynotruqqus', 'sussy@amogus.com');

-- dummy posts
truncate table post cascade;
truncate table post_vote cascade;
-- id 1
insert into post (title, body, body_html, creator_id, board_id) values ('Come check out my latest Electric Car', 'wow this is a post lmao I do not know what I am doing', '', 2, 1);
--id 2
insert into post (title, body, body_html, creator_id, board_id) values ('Sorry you need to pay me 8$ a month, and here is why!', 'darkness dies in democracy folks', '', 2, 1);
--id 3
insert into post (title, body, body_html, creator_id, board_id) values ('DogeCoin is definitely going to moon this time, trust me', 'meme coin good, trad fi bad', '', 2, 1);
--id 4
insert into post (title, body, body_html, creator_id, board_id) values ('I am the first African American owner of Twitter, AMA', 'no you still cannot use the n wordr', '', 2, 1);
--id 5
insert into post (title, body, body_html, creator_id, board_id) values ('Is this a meme?', 'surprised pikachu jpg', '', 2, 1);
--id 6
insert into post (title, body, body_html, creator_id, board_id) values ('Test post pls ignore', '', '', 2, 1);
--id 7
insert into post (title, body, body_html, creator_id, board_id) values ('Fortune favors the bold', 'what is Matt Damon doing here?', '', 2, 1);
--id 8
insert into post (title, body, body_html, creator_id, board_id) values ('Crickets are actually tasty', 'No seriously, tastes like you own nothing or something teehee', '', 3, 1);
--id 9
insert into post (title, body, body_html, creator_id, board_id) values ('Top 10 Reasons why you will own nothing, you vill be happy about reason 4', '', '', 3, 1);
--id 10
insert into post (title, body, body_html, creator_id, board_id) values ('Generic Shitpost 1', '', '', 3, 1);
--id 11
insert into post (title, body, body_html, creator_id, board_id) values ('WEF Greatest Hits 2022', '', '', 3, 1);
--id 12
insert into post (title, body, body_html, creator_id, board_id) values ('Plausible Deniability mode is coming and you should be pumped', '', '', 4, 1);
--id 13
insert into post (title, body, body_html, creator_id, board_id) values ('Generic Post 3', '', '', 4, 1);
--id 14
insert into post (title, body, body_html, creator_id, board_id) values ('TEST', '', '', 4, 1);
--id 15
insert into post (title, body, body_html, creator_id, board_id) values ('Woah this is a post', '', '', 4, 1);
-- dummy post votes
insert into post_vote (post_id, user_id, score) values (1, 2, 1);
insert into post_vote (post_id, user_id, score) values (1, 3, -1);
insert into post_vote (post_id, user_id, score) values (1, 4, 0);
insert into post_vote (post_id, user_id, score) values (2, 2, -1);
insert into post_vote (post_id, user_id, score) values (2, 3, 1);
insert into post_vote (post_id, user_id, score) values (2, 4, 1);
insert into post_vote (post_id, user_id, score) values (3, 2, 1);
insert into post_vote (post_id, user_id, score) values (3, 3, 1);
insert into post_vote (post_id, user_id, score) values (3, 4, 1);
insert into post_vote (post_id, user_id, score) values (4, 2, 0);
insert into post_vote (post_id, user_id, score) values (4, 3, 1);
insert into post_vote (post_id, user_id, score) values (4, 4, -1);
insert into post_vote (post_id, user_id, score) values (5, 2, 1);
insert into post_vote (post_id, user_id, score) values (6, 3, -1);
insert into post_vote (post_id, user_id, score) values (7, 4, 0);
insert into post_vote (post_id, user_id, score) values (8, 2, -1);
insert into post_vote (post_id, user_id, score) values (9, 3, 1);
insert into post_vote (post_id, user_id, score) values (10, 4, 1);
insert into post_vote (post_id, user_id, score) values (11, 2, 1);
insert into post_vote (post_id, user_id, score) values (12, 3, 1);
insert into post_vote (post_id, user_id, score) values (13, 4, 1);
insert into post_vote (post_id, user_id, score) values (14, 2, 0);
insert into post_vote (post_id, user_id, score) values (15, 3, 1);


--dummy comments
truncate table comment cascade;
truncate table comment_vote cascade;
--id 1
insert into comment (creator_id, post_id, parent_id, body, body_html) values (2, 10, null, 'Haha yessss! that is such a funny meme!', '');
--id 2
insert into comment (creator_id, post_id, parent_id, body, body_html) values (2, 5, null, 'yup, definitely a meme', '');
--id 3
insert into comment (creator_id, post_id, parent_id, body, body_html) values (3, 7, null, 'Matt Damon vill own nothing when the plan goes through!', '');
--id 4
insert into comment (creator_id, post_id, parent_id, body, body_html) values (3, 8, null, 'My favorite cricket flavor is lemon meringue.', '');
--id 5
insert into comment (creator_id, post_id, parent_id, body, body_html) values (4, 9, null, 'generic comment', '');


--dummy comment_votes
insert into comment_vote (user_id, comment_id, score) values (2, 1, 1);
insert into comment_vote (user_id, comment_id, score) values (2, 4, -1);
insert into comment_vote (user_id, comment_id, score) values (3, 3, 0);
insert into comment_vote (user_id, comment_id, score) values (4, 2, 1);
insert into comment_vote (user_id, comment_id, score) values (3, 4, 1);
insert into comment_vote (user_id, comment_id, score) values (4, 1, 1);
