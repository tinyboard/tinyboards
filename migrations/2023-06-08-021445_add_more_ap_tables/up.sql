create table instance (
    id serial primary key,
    domain text not null unique,
    creation_date timestamp not null default now(),
    updated timestamp null
);

alter table site add column actor_id text not null default 'http://fake.com';

insert into instance (domain)
select distinct substring(p.actor_id from '(?:.*://)?(?:www\.)?([^/?]*)') from (
    select actor_id from site
    union
    select actor_id from person
    union 
    select actor_id from boards
) as p;

alter table site add column instance_id int references instance on update cascade on delete cascade;
alter table person add column instance_id int references instance on update cascade on delete cascade;
alter table boards add column instance_id int references instance on update cascade on delete cascade;

update site set instance_id = i.id 
from instance i
where substring(actor_id from '(?:.*://)?(?:www\.)?([^/?]*)') = i.domain;

update person set instance_id = i.id 
from instance i
where substring(actor_id from '(?:.*://)?(?:www\.)?([^/?]*)') = i.domain;

update boards set instance_id = i.id 
from instance i
where substring(actor_id from '(?:.*://)?(?:www\.)?([^/?]*)') = i.domain;

alter table site alter column instance_id set not null;
alter table site add constraint idx_site_instance_unique unique (instance_id);

alter table person alter column instance_id set not null;
alter table boards alter column instance_id set not null;

create table federation_allowlist (
  id serial primary key,
  instance_id int references instance on update cascade on delete cascade not null unique,
  creation_date timestamp not null default now(),
  updated timestamp null
);

create table federation_blocklist (
  id serial primary key,
  instance_id int references instance on update cascade on delete cascade not null unique,
  creation_date timestamp not null default now(),
  updated timestamp null
);

create table local_site (
  id serial primary key,
  site_id int references site on update cascade on delete cascade not null unique,
  site_setup boolean default false not null,
  invite_only boolean default false not null,
  enable_downvotes boolean default true not null,
  open_registration boolean default true not null,
  enable_nsfw boolean default true not null,
  board_creation_admin_only boolean default false not null,
  require_email_verification boolean default false not null,
  require_application boolean default true not null,
  application_question text default 'To verify that you are a human, please explain why you want to create an account on this site'::text,
  private_instance boolean default false not null,
  default_theme text default 'browser'::text not null,
  default_post_listing_type text default 'Local'::text not null,
  default_avatar text,
  legal_information text,
  hide_modlog_mod_names boolean default true not null,
  application_email_admins boolean default false not null,

  -- Fields from config
  actor_name_max_length int default 20 not null,
  federation_enabled boolean default true not null,
  federation_debug boolean default false not null,
  federation_strict_allowlist boolean default true not null,
  federation_http_fetch_retry_limit int default 25 not null,
  federation_worker_count int default 64 not null,
  captcha_enabled boolean default false not null,
  captcha_difficulty varchar(255) default 'medium' not null,

  -- Time fields
  creation_date timestamp without time zone default now() not null,
  updated timestamp without time zone
);

create table local_site_rate_limit (
  id serial primary key,
  local_site_id int references local_site on update cascade on delete cascade not null unique,
  message int default 180 not null,
  message_per_second int default 60 not null,
  post int default 6 not null,
  post_per_second int default 600 not null,
  register int default 3 not null,
  register_per_second int default 3600 not null,
  image int default 6 not null,
  image_per_second int default 3600 not null,
  comment int default 6 not null,
  comment_per_second int default 600 not null,
  search int default 60 not null,
  search_per_second int default 600 not null,
  creation_date timestamp without time zone default now() not null,
  updated timestamp without time zone
);

-- Insert the data into local_site
insert into local_site (
  site_id, 
  site_setup,
  invite_only,
  enable_downvotes,
  open_registration,
  enable_nsfw,
  board_creation_admin_only,
  require_email_verification,
  require_application,
  application_question,
  private_instance,
  default_theme,
  default_post_listing_type,
  default_avatar,
  legal_information,
  hide_modlog_mod_names,
  application_email_admins,
  creation_date,
  updated
) 
select 
  id, 
  true, -- Assume site if setup if there's already a site row
  invite_only,
  enable_downvotes,
  open_registration,
  enable_nsfw,
  true,
  email_verification_required,
  require_application,
  application_question,
  private_instance,
  'browser',
  'Local',
  default_avatar,
  '',
  false,
  true,
  now(),
  now()
from site
order by id limit 1;

insert into local_site_rate_limit (
  local_site_id
)
select id from local_site
order by id limit 1;

alter table site
  drop column enable_downvotes cascade,
  drop column open_registration cascade,
  drop column enable_nsfw cascade,
  drop column email_verification_required cascade,
  drop column require_application cascade,
  drop column application_question cascade,
  drop column private_instance cascade,
  drop column invite_only cascade,
  drop column default_avatar cascade;