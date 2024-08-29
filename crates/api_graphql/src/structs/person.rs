use async_graphql::*;
use tinyboards_db::{
    aggregates::structs::PersonAggregates as DbPersonAggregates,
    models::{
        board::board_mods::BoardModerator as DbBoardMod,
        comment::comments::Comment as DbComment,
        person::{
            local_user::AdminPerms,
            person::{Person as DbPerson, PersonSafe as DbPersonSafe},
        },
        post::posts::Post as DbPost,
    },
    utils::DbPool,
};
use tinyboards_utils::TinyBoardsError;

use crate::{CommentSortType, ListingType, LoggedInUser, SortType};

use super::{board_mods::BoardMod, comment::Comment, post::Post};

/// GraphQL representation of Person.
#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Person {
    pub id: i32,
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

// resolvers for PersonAggregates fields
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

    pub async fn moderates(&self, ctx: &Context<'_>) -> Result<Vec<BoardMod>> {
        let pool = ctx.data_unchecked::<DbPool>();

        DbBoardMod::for_person(pool, self.id)
            .await
            .map(|res| {
                res.into_iter()
                    .map(BoardMod::from)
                    .collect::<Vec<BoardMod>>()
            })
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "Failed to load modded boards.").into()
            })
    }

    pub async fn posts<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        #[graphql(desc = "Limit of how many posts to load. Max value and default is 25.")]
        limit: Option<i64>,
        #[graphql(desc = "Sorting type.")] sort: Option<SortType>,
        #[graphql(desc = "If specified, only posts from the given user will be loaded.")]
        board_id: Option<i32>,
        #[graphql(desc = "Page.")] page: Option<i64>,
    ) -> Result<Vec<Post>> {
        let pool = ctx.data::<DbPool>()?;
        let v_opt = ctx.data::<LoggedInUser>()?.inner();

        let sort = sort.unwrap_or(SortType::NewComments);
        let listing_type = ListingType::All;
        let limit = std::cmp::min(limit.unwrap_or(25), 25);
        let person_id_join = match v_opt {
            Some(v) => v.person.id,
            None => -1,
        };

        let is_admin = match v_opt {
            Some(v) => v
                .local_user
                .has_permissions_any(AdminPerms::Boards + AdminPerms::Content),
            None => false,
        };

        let is_self = match v_opt {
            Some(v) => self.id == v.person.id,
            None => false,
        };

        // Post history of banned users is hidden
        if self.is_banned && !(is_admin || is_self) {
            return Ok(vec![]);
        }

        let (include_deleted, include_removed, include_banned_boards) = if is_admin {
            // admins see everything
            (true, true, true)
        } else if is_self {
            // you can see your own removed content, but not posts in banned boards, and posts that you've deleted
            (false, true, false)
        } else {
            // logged out; or logged in, but neither self nor admin
            (false, false, false)
        };

        let posts = DbPost::load_with_counts(
            pool,
            person_id_join,
            Some(limit),
            page,
            include_deleted,
            include_removed,
            include_banned_boards,
            false,
            board_id,
            Some(self.id),
            sort.into(),
            listing_type.into(),
        )
        .await?;

        Ok(posts.into_iter().map(Post::from).collect::<Vec<Post>>())
    }

    pub async fn comments(
        &self,
        ctx: &Context<'_>,
        sort: Option<CommentSortType>,
        limit: Option<i64>,
        page: Option<i64>,
    ) -> Result<Vec<Comment>> {
        let pool = ctx.data::<DbPool>()?;
        let v_opt = ctx.data::<LoggedInUser>()?.inner();

        let sort = sort.unwrap_or(CommentSortType::New);
        let listing_type = ListingType::All;
        let limit = std::cmp::min(limit.unwrap_or(25), 25);
        let person_id_join = match v_opt {
            Some(v) => v.person.id,
            None => -1,
        };

        let is_admin = match v_opt {
            Some(v) => v
                .local_user
                .has_permissions_any(AdminPerms::Boards + AdminPerms::Content),
            None => false,
        };

        let is_self = match v_opt {
            Some(v) => self.id == v.person.id,
            None => false,
        };

        // Comment history of banned users is hidden
        if self.is_banned && !(is_admin || is_self) {
            return Ok(vec![]);
        }

        let (include_deleted, include_removed, include_banned_boards) = if is_admin {
            // admins see everything
            (true, true, true)
        } else if is_self {
            // you can see your own removed content, but not comments in banned boards, and comments that you've deleted
            (false, true, false)
        } else {
            // logged out; or logged in, but neither self nor admin
            (false, false, false)
        };

        let comments = DbComment::load_with_counts(
            pool,
            person_id_join,
            sort.into(),
            listing_type.into(),
            page,
            Some(limit),
            Some(self.id),
            None,
            None,
            false,
            None,
            include_deleted,
            include_removed,
            include_banned_boards,
            None,
        )
        .await
        .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to load comments"))?;

        Ok(comments
            .into_iter()
            .map(Comment::from)
            .collect::<Vec<Comment>>())
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

impl From<(DbPersonSafe, DbPersonAggregates)> for Person {
    fn from((person, counts): (DbPersonSafe, DbPersonAggregates)) -> Self {
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
