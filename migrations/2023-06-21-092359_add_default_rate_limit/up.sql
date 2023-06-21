-- Use defaults for everything
insert into local_site_rate_limit(local_site_id) values ((select id from local_site limit 1));