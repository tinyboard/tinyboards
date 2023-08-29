create table private_message (
    id serial primary key,
    creator_id int references person on update cascade on delete cascade not null,
    recipient_user_id int references person on update cascade on delete cascade,
    recipient_board_id int references person on update cascade on delete cascade,
    body text not null,
    body_html text not null,
    published timestamp not null default now(),
    updated timestamp,
    -- either a board or user must be set as recipient, but not both
    constraint either_user_or_board check ((recipient_user_id is null) <> (recipient_board_id is null))
);

-- Notifications for PMs
create table pm_notif (
    id serial primary key,
    recipient_id int references person on update cascade on delete cascade not null,
    pm_id int references private_message on update cascade on delete cascade not null,
    read boolean default false not null,
    creation_date timestamp not null default now(),
    unique(recipient_id, pm_id)
);
