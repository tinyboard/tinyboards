use async_graphql::*;
use tinyboards_db::models::person::person::Person as DbPerson;

#[derive(SimpleObject)]
pub(crate) struct Person {
    id: i32,
    name: String,
    is_banned: bool,
    display_name: Option<String>,
    bio: Option<String>,
    bio_html: Option<String>,
    avatar: Option<String>,
    banner: Option<String>,
    profile_background: Option<String>,
    admin_level: i32,
    is_local: bool,
    instance: Option<String>,
}

impl From<DbPerson> for Person {
    fn from(value: DbPerson) -> Self {
        Self {
            id: value.id,
            name: value.name,
            is_banned: value.is_banned,
            display_name: value.display_name,
            bio: value.bio,
            bio_html: value.bio_html,
            avatar: value.avatar.map(|a| a.as_str().into()),
            banner: value.banner.map(|a| a.as_str().into()),
            profile_background: value.profile_background.map(|a| a.as_str().into()),
            admin_level: value.admin_level,
            is_local: value.local,
            instance: value.instance,
        }
    }
}

impl From<&DbPerson> for Person {
    fn from(value: &DbPerson) -> Self {
        Self::from(value.clone())
    }
}
