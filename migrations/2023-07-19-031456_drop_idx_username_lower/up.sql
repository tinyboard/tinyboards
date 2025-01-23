drop index idx_user_name_lower;
create unique index idx_username_lower_and_instance_id on person (lower(name), instance_id);