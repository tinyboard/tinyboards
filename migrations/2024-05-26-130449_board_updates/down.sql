
alter table local_site drop column boards_enabled;


alter table boards add column is_banned boolean not null default false;
alter table boards drop column ban_reason;
alter table boards drop column primary_color;
alter table boards drop column secondary_color;
alter table boards drop column hover_color;
alter table boards drop column sidebar;
alter table boards drop column sidebar_html;
