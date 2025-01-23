-- Add back site columns
alter table site
  add column enable_downvotes boolean default true not null,
  add column open_registration boolean default true not null,
  add column enable_nsfw boolean default true not null,
  add column email_verification_required boolean default false not null,
  add column require_application boolean default true not null,
  add column application_question text default 'to verify that you are human, please explain why you want to create an account on this site'::text,
  add column private_instance boolean default false not null,
  add column default_avatar text,
  add column invite_only boolean default false not null;

alter table site drop column actor_id cascade;

-- Insert the data back from local_site
update site set
  enable_downvotes = ls.enable_downvotes,
  open_registration = ls.open_registration,
  enable_nsfw = ls.enable_nsfw,
  email_verification_required = ls.require_email_verification,
  require_application = ls.require_application,
  application_question = ls.application_question,
  private_instance = ls.private_instance
from (select 
  site_id, 
  enable_downvotes,
  open_registration,
  enable_nsfw,
  require_email_verification,
  require_application,
  application_question,
  private_instance
from local_site) as ls
where site.id = ls.site_id;

alter table site drop column instance_id;
alter table person drop column instance_id;
alter table boards drop column instance_id;

drop table local_site_rate_limit;
drop table local_site;
drop table federation_allowlist;
drop table federation_blocklist;
drop table instance;