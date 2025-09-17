alter table person drop constraint unique_name_and_instance;
alter table person add constraint user__name_key unique(name);