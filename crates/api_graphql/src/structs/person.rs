use async_graphql::*;
use tinyboards_db::{
    aggregates::structs::PersonAggregates as DbPersonAggregates,
    models::person::person::Person as DbPerson,
};

/// GraphQL representation of Person.
#[derive(SimpleObject)]
#[graphql(complex)]
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
    // `counts` is not queryable, instead, its fields are available for Person thru dynamic resolvers
    #[graphql(skip)]
    counts: DbPersonAggregates,
}

/// Own profile
#[derive(SimpleObject)]
pub struct Me {
    pub person: Option<Person>,
    pub unread_replies_count: Option<i64>,
    pub unread_mentions_count: Option<i64>,
}

// resolvers for Person
#[ComplexObject]
impl Person {
    pub async fn post_count(&self) -> i64 {
        self.counts.post_count
    }

    pub async fn comment_count(&self) -> i64 {
        self.counts.comment_count
    }

    pub async fn post_score(&self) -> i64 {
        self.counts.post_score
    }

    pub async fn comment_score(&self) -> i64 {
        self.counts.comment_score
    }

    pub async fn rep(&self) -> i64 {
        self.counts.rep
    }
}

impl From<(DbPerson, DbPersonAggregates)> for Person {
    fn from((person, counts): (DbPerson, DbPersonAggregates)) -> Self {
        Self {
            id: person.id,
            name: person.name,
            is_banned: person.is_banned,
            is_deleted: person.is_deleted,
            unban_date: person.unban_date.map(|t| t.to_string()),
            display_name: person.display_name,
            bio: person.bio,
            bio_html: person.bio_html,
            creation_date: person.creation_date.to_string(),
            updated: person.updated.map(|t| t.to_string()),
            avatar: person.avatar.map(|a| a.as_str().into()),
            banner: person.banner.map(|a| a.as_str().into()),
            profile_background: person.profile_background.map(|a| a.as_str().into()),
            admin_level: person.admin_level,
            is_local: person.local,
            instance: person.instance,
            profile_music: person.profile_music.map(|m| m.as_str().into()),
            profile_music_youtube: person.profile_music_youtube,
            counts,
        }
    }
}

impl From<(&DbPerson, &DbPersonAggregates)> for Person {
    fn from((ref_person, ref_counts): (&DbPerson, &DbPersonAggregates)) -> Self {
        Self::from((ref_person.clone(), ref_counts.clone()))
    }
}
