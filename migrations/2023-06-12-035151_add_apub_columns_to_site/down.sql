alter table site
    drop column icon cascade,
    drop column banner cascade,
    drop column description cascade,
    drop column last_refreshed_date cascade,
    drop column inbox_url cascade,
    drop column private_key cascade,
    drop column public_key cascade;
alter table site rename column sidebar to description;
alter table site add column creator_id int references person on update cascade on delete cascade not null default 1; -- had to set a default here