drop index idx_username_lower_and_instance_id;
create unique index idx_user_name_lower on person (lower(name));