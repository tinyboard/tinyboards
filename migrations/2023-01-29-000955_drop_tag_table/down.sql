-- create tags table
create table tag (
    id serial primary key,
    name varchar(100) not null unique
);

-- insert a couple default tags into the table
insert into tag (name) values
('Discussion'),
('Memes'),
('Gaming'),
('Movies'),
('TV'),
('Music'),
('Literature'),
('Photography'),
('Art'),
('Learning'),
('DIY'),
('Lifestyle'),
('News'),
('Politics'),
('Religion'),
('Science'),
('Technology'),
('Programming'),
('Health'),
('Fitness'),
('Sports'),
('Places'),
('Meta'),
('Other');

alter table boards add column tag_id int references tag on update cascade on delete cascade;