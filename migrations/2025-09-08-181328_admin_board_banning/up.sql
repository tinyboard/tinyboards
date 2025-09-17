-- Add admin board banning fields to boards table
alter table boards add column is_banned boolean not null default false;
alter table boards add column public_ban_reason text;
alter table boards add column banned_by integer references person(id);
alter table boards add column banned_at timestamp;

-- Create admin board ban log table
create table admin_ban_board (
    id serial primary key,
    admin_id integer not null references person(id),
    board_id integer not null references boards(id),
    internal_notes text,
    public_ban_reason text,
    action varchar(10) not null, -- 'ban' or 'unban'
    when_ timestamp not null default now()
);