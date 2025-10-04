use crate::{
    models::reaction::reactions::{Reaction, ReactionForm, ReactionAggregate, BoardReactionSettings, BoardReactionSettingsForm},
    schema::{reactions, reaction_aggregates, board_reaction_settings},
    traits::Crud,
    utils::DbPool,
};
use diesel::{result::Error, *};
use diesel_async::RunQueryDsl;

#[async_trait::async_trait]
impl Crud for Reaction {
    type Form = ReactionForm;
    type IdType = i32;

    async fn create(pool: &DbPool, form: &Self::Form) -> Result<Self, Error> {
        let mut conn = pool.get().await.unwrap();
        insert_into(reactions::table)
            .values(form)
            .get_result::<Self>(&mut conn)
            .await
    }

    async fn read(pool: &DbPool, reaction_id: Self::IdType) -> Result<Self, Error> {
        let mut conn = pool.get().await.unwrap();
        reactions::table
            .find(reaction_id)
            .first::<Self>(&mut conn)
            .await
    }

    async fn update(pool: &DbPool, reaction_id: Self::IdType, form: &Self::Form) -> Result<Self, Error> {
        let mut conn = pool.get().await.unwrap();
        diesel::update(reactions::table.find(reaction_id))
            .set((
                reactions::emoji.eq(&form.emoji),
                reactions::score.eq(form.score),
            ))
            .get_result::<Self>(&mut conn)
            .await
    }

    async fn delete(pool: &DbPool, reaction_id: Self::IdType) -> Result<usize, Error> {
        let mut conn = pool.get().await.unwrap();
        diesel::delete(reactions::table.find(reaction_id))
            .execute(&mut conn)
            .await
    }
}

impl Reaction {
    /// Get a user's reaction to a post or comment by emoji
    pub async fn get_user_reaction(
        pool: &DbPool,
        user_id: i32,
        post_id: Option<i32>,
        comment_id: Option<i32>,
        emoji: &str,
    ) -> Result<Self, Error> {
        let mut conn = pool.get().await.unwrap();
        let mut query = reactions::table
            .filter(reactions::user_id.eq(user_id))
            .filter(reactions::emoji.eq(emoji))
            .into_boxed();

        if let Some(pid) = post_id {
            query = query.filter(reactions::post_id.eq(pid));
        }
        if let Some(cid) = comment_id {
            query = query.filter(reactions::comment_id.eq(cid));
        }

        query.first::<Self>(&mut conn).await
    }

    /// Get all reactions for a post
    pub async fn list_for_post(pool: &DbPool, post_id: i32) -> Result<Vec<Self>, Error> {
        let mut conn = pool.get().await.unwrap();
        reactions::table
            .filter(reactions::post_id.eq(post_id))
            .order_by(reactions::creation_date.asc())
            .load::<Self>(&mut conn)
            .await
    }

    /// Get all reactions for a comment
    pub async fn list_for_comment(pool: &DbPool, comment_id: i32) -> Result<Vec<Self>, Error> {
        let mut conn = pool.get().await.unwrap();
        reactions::table
            .filter(reactions::comment_id.eq(comment_id))
            .order_by(reactions::creation_date.asc())
            .load::<Self>(&mut conn)
            .await
    }

    /// Delete a user's reaction by emoji
    pub async fn delete_user_reaction(
        pool: &DbPool,
        user_id: i32,
        post_id: Option<i32>,
        comment_id: Option<i32>,
        emoji: &str,
    ) -> Result<usize, Error> {
        let mut conn = pool.get().await.unwrap();
        let mut query = diesel::delete(reactions::table)
            .filter(reactions::user_id.eq(user_id))
            .filter(reactions::emoji.eq(emoji))
            .into_boxed();

        if let Some(pid) = post_id {
            query = query.filter(reactions::post_id.eq(pid));
        }
        if let Some(cid) = comment_id {
            query = query.filter(reactions::comment_id.eq(cid));
        }

        query.execute(&mut conn).await
    }
}

impl ReactionAggregate {
    /// Get all reaction aggregates for a post
    pub async fn list_for_post(pool: &DbPool, post_id: i32) -> Result<Vec<Self>, Error> {
        let mut conn = pool.get().await.unwrap();
        reaction_aggregates::table
            .filter(reaction_aggregates::post_id.eq(post_id))
            .order_by(reaction_aggregates::count.desc())
            .load::<Self>(&mut conn)
            .await
    }

    /// Get all reaction aggregates for a comment
    pub async fn list_for_comment(pool: &DbPool, comment_id: i32) -> Result<Vec<Self>, Error> {
        let mut conn = pool.get().await.unwrap();
        reaction_aggregates::table
            .filter(reaction_aggregates::comment_id.eq(comment_id))
            .order_by(reaction_aggregates::count.desc())
            .load::<Self>(&mut conn)
            .await
    }
}

#[async_trait::async_trait]
impl Crud for BoardReactionSettings {
    type Form = BoardReactionSettingsForm;
    type IdType = i32;

    async fn create(pool: &DbPool, form: &Self::Form) -> Result<Self, Error> {
        let mut conn = pool.get().await.unwrap();
        insert_into(board_reaction_settings::table)
            .values(form)
            .get_result::<Self>(&mut conn)
            .await
    }

    async fn read(pool: &DbPool, settings_id: Self::IdType) -> Result<Self, Error> {
        let mut conn = pool.get().await.unwrap();
        board_reaction_settings::table
            .find(settings_id)
            .first::<Self>(&mut conn)
            .await
    }

    async fn update(pool: &DbPool, settings_id: Self::IdType, form: &Self::Form) -> Result<Self, Error> {
        let mut conn = pool.get().await.unwrap();

        // Build update query conditionally
        if let Some(ref weights) = form.emoji_weights {
            if let Some(enabled) = form.reactions_enabled {
                diesel::update(board_reaction_settings::table.find(settings_id))
                    .set((
                        board_reaction_settings::emoji_weights.eq(weights),
                        board_reaction_settings::reactions_enabled.eq(enabled),
                    ))
                    .get_result::<Self>(&mut conn)
                    .await
            } else {
                diesel::update(board_reaction_settings::table.find(settings_id))
                    .set(board_reaction_settings::emoji_weights.eq(weights))
                    .get_result::<Self>(&mut conn)
                    .await
            }
        } else if let Some(enabled) = form.reactions_enabled {
            diesel::update(board_reaction_settings::table.find(settings_id))
                .set(board_reaction_settings::reactions_enabled.eq(enabled))
                .get_result::<Self>(&mut conn)
                .await
        } else {
            // No fields to update, just return the existing record
            Self::read(pool, settings_id).await
        }
    }

    async fn delete(pool: &DbPool, settings_id: Self::IdType) -> Result<usize, Error> {
        let mut conn = pool.get().await.unwrap();
        diesel::delete(board_reaction_settings::table.find(settings_id))
            .execute(&mut conn)
            .await
    }
}

impl BoardReactionSettings {
    /// Get settings for a board
    pub async fn get_for_board(pool: &DbPool, board_id: i32) -> Result<Self, Error> {
        let mut conn = pool.get().await.unwrap();
        board_reaction_settings::table
            .filter(board_reaction_settings::board_id.eq(board_id))
            .first::<Self>(&mut conn)
            .await
    }
}
