use crate::schema::local_site;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = local_site)]
pub struct LocalSite {
    pub id: i32,
    pub site_id: i32,
    pub site_setup: bool,
    pub invite_only: bool,
    pub enable_downvotes: bool,
    pub open_registration: bool,
    pub enable_nsfw: bool,
    pub board_creation_admin_only: bool,
    pub require_email_verification: bool,
    pub require_application: bool,
    pub application_question: Option<String>,
    pub private_instance: bool,
    pub default_theme: String,
    pub default_post_listing_type: String,
    pub default_avatar: Option<String>,
    pub legal_information: Option<String>,
    pub hide_modlog_mod_names: bool,
    pub application_email_admins: bool,
    pub actor_name_max_length: i32,
    pub federation_enabled: bool,
    pub federation_debug: bool,
    pub federation_strict_allowlist: bool,
    pub federation_http_fetch_retry_limit: i32,
    pub federation_worker_count: i32,
    pub captcha_enabled: bool,
    pub captcha_difficulty: String,
    pub creation_date: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = local_site)]
pub struct LocalSiteForm {
    pub site_setup: Option<bool>,
    pub invite_only: Option<bool>,
    pub enable_downvotes: Option<bool>,
    pub open_registration: Option<bool>,
    pub enable_nsfw: Option<bool>,
    pub board_creation_admin_only: Option<bool>,
    pub require_email_verification: Option<bool>,
    pub require_application: Option<bool>,
    pub application_question: Option<Option<String>>,
    pub private_instance: Option<bool>,
    pub default_theme: Option<String>,
    pub default_post_listing_type: Option<String>,
    pub default_avatar: Option<Option<String>>,
    pub legal_information: Option<Option<String>>,
    pub hide_modlog_mod_names: Option<bool>,
    pub application_email_admins: Option<bool>,
    pub actor_name_max_length: Option<i32>,
    pub federation_enabled: Option<bool>,
    pub federation_debug: Option<bool>,
    pub federation_strict_allowlist: Option<bool>,
    pub federation_http_fetch_retry_limit: Option<i32>,
    pub federation_worker_count: Option<i32>,
    pub captcha_enabled: Option<bool>,
    pub captcha_difficulty: Option<String>,
    pub updated: Option<NaiveDateTime>,
}