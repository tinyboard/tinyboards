alter table board_mods add column permissions integer not null default 32;
alter table board_mods add column rank integer not null default 1;
alter table board_mods add column invite_accepted boolean not null default true;
