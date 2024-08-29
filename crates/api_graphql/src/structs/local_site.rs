use async_graphql::*;
use tinyboards_db::models::{
    person::local_user::AdminPerms, site::local_site::LocalSite as DbLocalSite,
};

use crate::LoggedInUser;

#[derive(SimpleObject)]
#[graphql(complex)]
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
    pub creation_date: String,
    pub updated: Option<String>,
    pub reports_email_admins: bool,
    pub name: String,
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub hover_color: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    #[graphql(skip)]
    pub welcome_message_: Option<String>,
    pub boards_enabled: bool,
}

#[ComplexObject]
impl LocalSite {
    // only admins can read the welcome message
    pub async fn welcome_message(&self, ctx: &Context<'_>) -> Option<String> {
        let v_opt = ctx.data_unchecked::<LoggedInUser>().inner();

        match v_opt {
            Some(v) => {
                if v.local_user.has_permission(AdminPerms::Config) {
                    self.welcome_message_.clone()
                } else {
                    None
                }
            }
            None => None,
        }
    }
}

impl From<DbLocalSite> for LocalSite {
    fn from(value: DbLocalSite) -> Self {
        Self {
            id: value.id,
            site_id: value.site_id,
            site_setup: value.site_setup,
            invite_only: value.invite_only,
            enable_downvotes: value.enable_downvotes,
            open_registration: value.open_registration,
            enable_nsfw: value.enable_nsfw,
            board_creation_admin_only: value.board_creation_admin_only,
            require_email_verification: value.require_email_verification,
            require_application: value.require_application,
            application_question: value.application_question,
            private_instance: value.private_instance,
            default_theme: value.default_theme,
            default_post_listing_type: value.default_post_listing_type,
            default_avatar: value.default_avatar,
            legal_information: value.legal_information,
            hide_modlog_mod_names: value.hide_modlog_mod_names,
            application_email_admins: value.application_email_admins,
            actor_name_max_length: value.actor_name_max_length,
            federation_enabled: value.federation_enabled,
            federation_debug: value.federation_debug,
            federation_strict_allowlist: value.federation_strict_allowlist,
            federation_http_fetch_retry_limit: value.federation_http_fetch_retry_limit,
            federation_worker_count: value.federation_worker_count,
            captcha_enabled: value.captcha_enabled,
            captcha_difficulty: value.captcha_difficulty,
            creation_date: value.creation_date.to_string(),
            updated: value.updated.map(|u| u.to_string()),
            reports_email_admins: value.reports_email_admins,
            name: value.name,
            primary_color: value.primary_color,
            secondary_color: value.secondary_color,
            hover_color: value.hover_color,
            description: value.description,
            icon: value.icon,
            welcome_message_: value.welcome_message,
            boards_enabled: value.boards_enabled,
        }
    }
}
