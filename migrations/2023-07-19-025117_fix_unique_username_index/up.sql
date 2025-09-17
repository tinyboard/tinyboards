alter table person drop constraint user__name_key;
alter table person add constraint unique_name_and_instance unique(name, instance_id);