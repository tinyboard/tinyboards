alter table posts drop column ap_id;
alter table comments drop column ap_id;
alter table comments drop column if exists language_id cascade;