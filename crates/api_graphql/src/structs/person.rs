use async_graphql::*;
use tinyboards_db::models::person::person::Person as DbPerson;

/// GraphQL representation of Person.
#[derive(SimpleObject)]
pub struct Person {
    id: i32,
    name: String,
    is_banned: bool,
    is_deleted: bool,
    unban_date: Option<String>,
    display_name: Option<String>,
    bio: Option<String>,
    bio_html: Option<String>,
    creation_date: String,
    updated: Option<String>,
    avatar: Option<String>,
    banner: Option<String>,
    profile_background: Option<String>,
    admin_level: i32,
    is_local: bool,
    instance: Option<String>,
    profile_music: Option<String>,
    profile_music_youtube: Option<String>,
}

/// Own profile
#[derive(SimpleObject)]
pub struct Me {
    pub person: Option<Person>,
    pub unread_replies_count: Option<i64>,
    pub unread_mentions_count: Option<i64>,
}

impl From<DbPerson> for Person {
    fn from(value: DbPerson) -> Self {
        Self {
            id: value.id,
            name: value.name,
            is_banned: value.is_banned,
            is_deleted: value.is_deleted,
            unban_date: value.unban_date.map(|t| t.to_string()),
            display_name: value.display_name,
            bio: value.bio,
            bio_html: value.bio_html,
            creation_date: value.creation_date.to_string(),
            updated: value.updated.map(|t| t.to_string()),
            avatar: value.avatar.map(|a| a.as_str().into()),
            banner: value.banner.map(|a| a.as_str().into()),
            profile_background: value.profile_background.map(|a| a.as_str().into()),
            admin_level: value.admin_level,
            is_local: value.local,
            instance: value.instance,
            profile_music: value.profile_music.map(|m| m.as_str().into()),
            profile_music_youtube: value.profile_music_youtube,
        }
    }
}

impl From<&DbPerson> for Person {
    fn from(value: &DbPerson) -> Self {
        Self::from(value.clone())
    }
}
