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

create table board (
    id serial primary key,
    name varchar(50) not null unique,
    title varchar(150) not null,
    description text,
    tag_id int references tag on update cascade on delete cascade not null,
    creator_id int references user_ on update cascade on delete cascade not null,
    removed boolean default false not null,
    published timestamp not null default now(),
    updated timestamp
);

create table board_moderator (
    id serial primary key,
    board_id int references board on update cascade on delete cascade not null,
    user_id int references user_ on update cascade on delete cascade not null,
    published timestamp not null default now(),
    unique (board_id, user_id)
);

create table board_subscriber (
    id serial primary key,
    board_id int references board on update cascade on delete cascade not null,
    user_id int references user_ on update cascade on delete cascade not null,
    published timestamp not null default now(),
    unique(board_id, user_id)
);

create table board_user_ban (
    id serial primary key,
    board_id int references board on update cascade on delete cascade not null,
    user_id int references user_ on update cascade on delete cascade not null,
    published timestamp not null default now(),
    unique (board_id, user_id)
);

insert into board (name, title, tag_id, creator_id) values ('main', 'The Default Board', 1, 1);

create table site (
    id serial primary key,
    name varchar(20) not null unique,
    description text,
    creator_id int references user_ on update cascade on delete cascade not null,
    published timestamp not null default now(),
    updated timestamp
);