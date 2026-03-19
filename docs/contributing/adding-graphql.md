# Adding a GraphQL Query or Mutation

This guide walks through adding a new query and a new mutation to the TinyBoards API.

## Table of Contents

- [Adding a Query](#adding-a-query)
- [Adding a Mutation](#adding-a-mutation)
- [Testing](#testing)

## Adding a Query

Let's add a query `boardStats(boardName: String!): BoardStats` that returns post and subscriber counts for a board.

### 1. Define the Return Type

Create or edit the relevant query file. For board-related queries, this is `backend/crates/api/src/graphql/queries/boards.rs`.

```rust
use async_graphql::{Object, Context, Result, SimpleObject};

/// Statistics for a single board.
#[derive(SimpleObject)]
pub struct BoardStats {
    pub board_id: uuid::Uuid,
    pub post_count: i64,
    pub comment_count: i64,
    pub subscriber_count: i64,
}
```

### 2. Add the Query Resolver

In the same file, add the resolver method to the existing query struct:

```rust
#[Object]
impl QueryBoards {
    // ... existing queries ...

    /// Get aggregate statistics for a board.
    async fn board_stats(
        &self,
        ctx: &Context<'_>,
        board_name: String,
    ) -> Result<BoardStats> {
        let pool = ctx.data::<DbPool>()?;
        let mut conn = pool.get().await?;

        // Look up the board by name
        let board = boards::table
            .filter(boards::name.eq(&board_name))
            .first::<Board>(&mut conn)
            .await
            .map_err(|_| async_graphql::Error::new("Board not found"))?;

        // Fetch aggregates
        let agg = board_aggregates::table
            .filter(board_aggregates::board_id.eq(board.id))
            .first::<BoardAggregates>(&mut conn)
            .await?;

        Ok(BoardStats {
            board_id: board.id,
            post_count: agg.post_count,
            comment_count: agg.comment_count,
            subscriber_count: agg.subscriber_count,
        })
    }
}
```

### 3. Verify the Schema

The query struct (`QueryBoards`) is already merged into the root `Query` in `graphql/mod.rs`. If you're creating an entirely new query type, you'd register it there:

```rust
#[derive(MergedObject, Default)]
pub struct Query(
    QueryPosts,
    QueryComments,
    QueryBoards,      // already registered
    // ...
    YourNewQueryType, // add new types here
);
```

### 4. Test the Query

```bash
curl -X POST http://localhost:8536/api/v2/graphql \
  -H "Content-Type: application/json" \
  -d '{
    "query": "{ boardStats(boardName: \"general\") { boardId postCount commentCount subscriberCount } }"
  }'
```

## Adding a Mutation

Let's add a mutation `pinPost(postId: UUID!): Post` that pins a post to the top of a board.

### 1. Define the Input (if needed)

For simple mutations with one or two arguments, you can use inline parameters. For complex inputs, define an `InputObject`:

```rust
use async_graphql::InputObject;

#[derive(InputObject)]
pub struct PinPostInput {
    pub post_id: uuid::Uuid,
    /// Pin at board level (true) or site level (false).
    pub board_level: Option<bool>,
}
```

### 2. Add the Mutation Resolver

Edit the relevant mutation file — for post moderation actions, this is `backend/crates/api/src/graphql/mutations/post/moderation.rs`:

```rust
#[Object]
impl PostModeration {
    // ... existing mutations ...

    /// Pin a post to the top of its board. Requires moderator permissions.
    async fn pin_post(
        &self,
        ctx: &Context<'_>,
        input: PinPostInput,
    ) -> Result<Post> {
        // Extract the authenticated user from context
        let user = ctx.data::<AuthUser>()
            .map_err(|_| async_graphql::Error::new("Authentication required"))?;

        let pool = ctx.data::<DbPool>()?;
        let mut conn = pool.get().await?;

        // Fetch the post
        let post = posts::table
            .find(input.post_id)
            .first::<Post>(&mut conn)
            .await
            .map_err(|_| async_graphql::Error::new("Post not found"))?;

        // Check moderator permissions
        let is_mod = board_moderators::table
            .filter(board_moderators::board_id.eq(post.board_id))
            .filter(board_moderators::user_id.eq(user.id))
            .first::<BoardModerator>(&mut conn)
            .await
            .is_ok();

        if !is_mod && !user.is_admin {
            return Err(async_graphql::Error::new("Moderator access required"));
        }

        // Update the post
        let board_level = input.board_level.unwrap_or(true);
        let updated = diesel::update(posts::table.find(input.post_id))
            .set(posts::is_featured_board.eq(board_level))
            .get_result::<Post>(&mut conn)
            .await?;

        // Log the moderation action
        diesel::insert_into(moderation_log::table)
            .values((
                moderation_log::moderator_id.eq(user.id),
                moderation_log::action.eq(ModerationAction::FeaturePost),
                moderation_log::target_post_id.eq(Some(post.id)),
                moderation_log::board_id.eq(Some(post.board_id)),
            ))
            .execute(&mut conn)
            .await?;

        Ok(updated)
    }
}
```

### 3. Register the Mutation (if new type)

If adding to an existing mutation struct, no registration is needed. For a new mutation type:

```rust
// In graphql/mod.rs
#[derive(MergedObject, Default)]
pub struct Mutation(
    Auth,
    SubmitPost,
    PostModeration,    // already registered
    // ...
    YourNewMutation,   // add new types here
);
```

### 4. Test the Mutation

```bash
curl -X POST http://localhost:8536/api/v2/graphql \
  -H "Content-Type: application/json" \
  -b cookies.txt \
  -d '{
    "query": "mutation { pinPost(input: { postId: \"550e8400-e29b-41d4-a716-446655440000\" }) { id title isFeaturedBoard } }"
  }'
```

## Testing

### Write a Test

Add tests in the same crate or in a dedicated test file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_board_stats_returns_counts() {
        // Set up test database and context
        let pool = create_test_pool().await;
        let schema = build_test_schema(pool.clone());

        // Create test data
        let board = create_test_board(&pool, "testboard").await;
        create_test_post(&pool, board.id).await;

        // Execute query
        let result = schema
            .execute(r#"{ boardStats(boardName: "testboard") { postCount } }"#)
            .await;

        assert!(result.errors.is_empty());
        let data = result.data.into_json().unwrap();
        assert_eq!(data["boardStats"]["postCount"], 1);
    }
}
```

### Run Tests

```bash
cd backend
cargo test
# Or with nextest for better output
cargo nextest run
```

See [testing.md](testing.md) for the full testing guide.
