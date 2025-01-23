alter table posts add constraint posts_unique_ap_id unique(ap_id);
alter table comments add constraint comments_unique_ap_id unique(ap_id);