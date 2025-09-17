create table site_language(
    id serial primary key,
    site_id int references site on update cascade on delete cascade not null,
    language_id int references language on update cascade on delete cascade not null,
    unique(site_id, language_id)
);

create table board_language(
    id serial primary key,
    board_id int references boards on update cascade on delete cascade not null,
    language_id int references language on update cascade on delete cascade not null,
    unique(board_id, language_id)
);

-- update existing users, sites, etc to have all languages enabled
do $$
    declare
        xid integer;
begin
    for xid in select id from local_user
    loop
        insert into local_user_language (local_user_id, language_id)
        (select xid, language.id as lid from language);
    end loop;

    for xid in select id from site
    loop
        insert into site_language (site_id, language_id)
        (select xid, language.id as lid from language);
    end loop;

    for xid in select id from boards
    loop
        insert into board_language (board_id, language_id)
        (select xid, language.id as lid from language);
    end loop;
end;
$$;