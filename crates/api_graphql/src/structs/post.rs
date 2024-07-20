use async_graphql::*;
use tinyboards_db::aggregates::structs::PostAggregates as DbPostAggregates;
use tinyboards_db_views::structs::PostView;

use super::person::Person;

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
#[graphql(remote = "tinyboards_db::SortType")]
pub enum SortType {
    #[graphql(name = "active")]
    Active,
    #[graphql(name = "hot")]
    Hot,
    #[graphql(name = "new")]
    New,
    #[graphql(name = "old")]
    Old,
    #[graphql(name = "topDay")]
    TopDay,
    #[graphql(name = "topWeek")]
    TopWeek,
    #[graphql(name = "topMonth")]
    TopMonth,
    #[graphql(name = "topYear")]
    TopYear,
    #[graphql(name = "topAll")]
    TopAll,
    #[graphql(name = "mostComments")]
    MostComments,
    #[graphql(name = "newComments")]
    NewComments,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
#[graphql(remote = "tinyboards_db::ListingType")]
pub enum ListingType {
    #[graphql(name = "all")]
    All,
    #[graphql(name = "subscribed")]
    Subscribed,
    #[graphql(name = "local")]
    Local,
    #[graphql(name = "moderated")]
    Moderated,
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Post {
    id: i32,
    title: String,
    type_: String,
    url: Option<String>,
    body: String,
    body_html: String,
    creator_id: i32,
    board_id: i32,
    is_removed: bool,
    is_locked: bool,
    creation_date: String,
    is_deleted: bool,
    is_nsfw: bool,
    updated: Option<String>,
    image: Option<String>,
    local: bool,
    featured_board: bool,
    featured_local: bool,
    title_chunk: String,
    #[graphql(skip)]
    counts: DbPostAggregates,
    creator: Option<Person>,
    is_creator_banned_from_board: bool,
    is_saved: bool,
    my_vote: Option<i16>,
    mod_permissions: Option<i32>,
}

#[ComplexObject]
impl Post {
    pub async fn comment_count(&self) -> i64 {
        self.counts.comments
    }

    pub async fn score(&self) -> i64 {
        self.counts.score
    }

    pub async fn upvotes(&self) -> i64 {
        self.counts.upvotes
    }

    pub async fn newest_comment_time(&self) -> String {
        self.counts.newest_comment_time.to_string()
    }
}

impl From<PostView> for Post {
    #[allow(unused)]
    fn from(
        PostView {
            post,
            creator,
            creator_counts,
            board,
            creator_banned_from_board,
            counts,
            subscribed,
            saved,
            read,
            creator_blocked,
            my_vote,
            report_count,
            mod_permissions,
        }: PostView,
    ) -> Self {
        Self {
            id: post.id,
            title: post.title,
            type_: post.type_,
            url: post.url.map(|url| url.as_str().into()),
            body: post.body,
            body_html: post.body_html,
            creator_id: post.creator_id,
            board_id: post.board_id,
            is_removed: post.is_removed,
            is_locked: post.is_locked,
            creation_date: post.creation_date.to_string(),
            is_deleted: post.is_deleted,
            is_nsfw: post.is_nsfw,
            updated: post.updated.map(|t| t.to_string()),
            image: post.image.map(|i| i.as_str().into()),
            local: post.local,
            featured_board: post.featured_board,
            featured_local: post.featured_local,
            title_chunk: post.title_chunk,
            counts,
            creator: creator.map(|c| Person::from((c, creator_counts.unwrap()))),
            is_creator_banned_from_board: creator_banned_from_board,
            is_saved: saved,
            my_vote,
            mod_permissions,
        }
    }
}
