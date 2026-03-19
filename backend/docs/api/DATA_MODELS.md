# TinyBoards GraphQL Data Models and Relationships Documentation

This document provides comprehensive documentation for the TinyBoards GraphQL API data models, their relationships, and data fetching patterns.

## Core Domain Models Overview

TinyBoards is built around five core domain entities that form the backbone of the social platform:

### Primary Entities
- **User**: User accounts, profiles, and authentication
- **Board**: Topic-based communities (similar to subreddits)
- **Post**: Content submissions (links or text discussions)
- **Comment**: Nested discussion threads on posts
- **Vote**: Upvotes/downvotes on posts and comments

### Supporting Entities
- **Message**: Private messages between users
- **Notification**: System notifications for user activity
- **Report**: Content reports for moderation
- **Emoji**: Custom emojis for boards and site
- **Site**: Global site configuration

## Entity Relationship Diagram

```
User ──┬─── creates ──→ Post ──┬─── belongs to ──→ Board
       │                      │
       ├─── creates ──→ Comment ──┬─── belongs to ──→ Post
       │                         └─── parent ──→ Comment (nested)
       │
       ├─── votes on ──→ PostVote/CommentVote
       ├─── subscribes to ──→ Board
       ├─── moderates ──→ Board (via BoardModerator)
       ├─── sends/receives ──→ Message
       └─── receives ──→ Notification

Board ──┬─── contains ──→ Post
        ├─── has moderators ──→ User (via BoardModerator)
        ├─── has subscribers ──→ User (via BoardSubscriber)
        └─── has custom ──→ Emoji

Post ───┬─── has ──→ Comment
        └─── receives ──→ PostVote

Comment ──┬─── receives ──→ CommentVote
          └─── has replies ──→ Comment (self-referential)
```

## GraphQL Type Definitions

### User Type

**Fields:**
```graphql
type User {
  # Core Identity
  id: Int!
  name: String!                    # Username (unique, 30 chars max)
  displayName: String              # Display name (optional, 30 chars max)

  # Profile Information
  bio: String                      # User biography
  bioHTML: String                  # HTML-rendered biography
  avatar: String                   # Avatar image URL
  banner: String                   # Profile banner URL
  profileBackground: String        # Profile background image
  profileMusic: String             # Profile music URL
  profileMusicYoutube: String      # YouTube music URL

  # Status & Permissions
  isAdmin: Boolean!                # Computed: admin_level > 0
  adminLevel: Int!                 # Admin permission level (0-8)
  isBanned: Boolean!               # Account ban status
  isDeleted: Boolean!              # Account deletion status
  unbanDate: String                # When ban expires (if applicable)

  # Timestamps
  creationDate: String!            # Account creation timestamp
  updated: String                  # Last profile update

  # Computed Statistics (from UserAggregates)
  postCount: Int!                  # Total posts created
  commentCount: Int!               # Total comments created
  postScore: Int!                  # Total post karma
  commentScore: Int!               # Total comment karma
  rep: Int!                        # Computed: postScore + commentScore

  # Privacy-Protected Fields (only for own account)
  hasVerifiedEmail: Boolean        # Email verification status
  isApplicationAccepted: Boolean   # Registration application status
  settings: UserSettings           # User preferences

  # Relational Data
  joinedBoards(limit: Int, page: Int): [Board!]!
  moderates: [BoardMod!]!
  posts(limit: Int, sort: SortType, boardId: Int, page: Int): [Post!]!
  comments(sort: CommentSortType, limit: Int, page: Int): [Comment!]!
}

type UserSettings {
  id: Int!
  name: String!
  email: String
  showNSFW: Boolean!
  showBots: Boolean!
  theme: String!
  defaultSortType: Int!
  defaultListingType: Int!
  emailNotificationsEnabled: Boolean!
  interfaceLanguage: String!
  updated: String
}
```

**Database Mapping:**
- Primary table: `users`
- Aggregates: `user_aggregates` (post_count, comment_count, scores)
- Related: `board_subscriber`, `board_mods`, `user_ban`

### Board Type

**Fields:**
```graphql
type Board {
  # Core Identity
  id: Int!
  name: String!                    # Board name (unique, 50 chars max)
  title: String!                   # Display title (150 chars max)
  description: String              # Board description

  # Visual Customization
  icon: String                     # Board icon URL
  banner: String                   # Board banner URL
  primaryColor: String!            # Theme primary color
  secondaryColor: String!          # Theme secondary color
  hoverColor: String!              # Theme hover color
  sidebar: String                  # Sidebar content (markdown)
  sidebarHTML: String              # Sidebar content (HTML)

  # Configuration
  isNSFW: Boolean!                 # NSFW content flag
  isRemoved: Boolean!              # Removed by admin
  isBanned: Boolean!               # Banned status
  isDeleted: Boolean!              # Soft deletion
  banReason: String                # Internal ban reason
  publicBanReason: String          # Public ban reason
  bannedBy: Int                    # Admin who banned
  bannedAt: String                 # Ban timestamp

  # Admin-Only Fields
  isHidden: Boolean!               # Hidden from public (admin-only)
  excludeFromAll: Boolean!         # Exclude from /all (admin-only)

  # Timestamps
  creationDate: String!            # Board creation
  updated: String                  # Last modification

  # Computed Statistics (from BoardAggregates)
  subscribers: Int!                # Subscriber count
  postCount: Int!                  # Total posts
  commentCount: Int!               # Total comments
  usersActiveDay: Int!             # Active users (24h)
  usersActiveWeek: Int!            # Active users (7d)
  usersActiveMonth: Int!           # Active users (30d)
  usersActiveHalfYear: Int!        # Active users (180d)

  # User-Specific Data (requires authentication)
  myModPermissions: Int!           # Current user's mod permissions
  subscribedType: SubscribedType!  # Current user's subscription status

  # Relational Data
  moderators: [BoardMod!]!
  posts(limit: Int, sort: SortType, userId: Int, page: Int): [Post!]!
}

enum SubscribedType {
  SUBSCRIBED
  NOT_SUBSCRIBED
  PENDING
}
```

**Database Mapping:**
- Primary table: `boards`
- Aggregates: `board_aggregates` (subscribers, posts, comments, activity)
- Related: `board_subscriber`, `board_mods`, `board_user_bans`

### Post Type

**Fields:**
```graphql
type Post {
  # Core Identity
  id: Int!
  title: String!                   # Post title (200 chars max)
  type: String!                    # Post type: "text", "link", "image"

  # Content
  body: String!                    # Post body content
  bodyHTML: String!                # HTML-rendered body
  url: String                      # External URL (for link posts)
  image: String                    # Image URL (for image posts)
  altText: String                  # Alt text for images

  # Metadata
  embedTitle: String               # Link preview title
  embedDescription: String         # Link preview description
  embedVideoUrl: String            # Video embed URL
  sourceUrl: String                # Original source URL
  lastCrawlDate: String            # Last metadata crawl
  titleChunk: String               # Title for search indexing

  # Ownership & Location
  creatorId: Int!                  # Author user ID
  boardId: Int!                    # Board ID

  # Status Flags
  isRemoved: Boolean!              # Removed by moderator
  isLocked: Boolean!               # Comments locked
  isDeleted: Boolean!              # Deleted by author
  isNSFW: Boolean!                 # NSFW content flag
  featuredBoard: Boolean!          # Featured in board
  featuredLocal: Boolean!          # Featured site-wide
  local: Boolean!                  # Local content (always true)

  # Timestamps
  creationDate: String!            # Post creation
  updated: String                  # Last edit

  # Computed Statistics (from PostAggregates)
  commentCount: Int!               # Total comments
  score: Int!                      # Net score (upvotes - downvotes)
  upvotes: Int!                    # Total upvotes
  downvotes: Int!                  # Total downvotes
  newestCommentTime: String!       # Latest comment timestamp
  hotRank: Int!                    # Hot ranking algorithm score
  hotRankActive: Int!              # Active hot rank
  controversyRank: Float!          # Controversy score

  # User-Specific Data (requires authentication)
  myVote: Int!                     # Current user's vote (-1, 0, 1)
  myModPermissions: Int!           # Current user's mod permissions
  isSaved: Boolean!                # Saved by current user

  # Relational Data
  creator: User                    # Post author (DataLoader)
  board: Board                     # Parent board (DataLoader)
  participants: [User!]!           # Users who commented
  comments(
    sort: CommentSortType,
    listingType: ListingType,
    page: Int,
    limit: Int,
    search: String,
    topCommentId: Int,
    context: Int,
    includeDeleted: Boolean,
    includeRemoved: Boolean,
    maxDepth: Int
  ): [Comment!]!
}
```

**Database Mapping:**
- Primary table: `posts`
- Aggregates: `post_aggregates` (scores, counts, rankings)
- Related: `post_votes`, `post_saved`, `post_read`, `post_hidden`

### Comment Type

**Fields:**
```graphql
type Comment {
  # Core Identity
  id: Int!
  creatorId: Int!                  # Author user ID
  postId: Int!                     # Parent post ID
  parentId: Int                    # Parent comment ID (for nesting)
  boardId: Int!                    # Board ID

  # Content
  body: String!                    # Comment content
  bodyHTML: String!                # HTML-rendered content

  # Structure
  level: Int!                      # Nesting level (0 = top-level)
  replies: [Comment!]              # Nested replies

  # Status Flags
  isRemoved: Boolean!              # Removed by moderator
  isDeleted: Boolean!              # Deleted by author
  isLocked: Boolean!               # Editing locked
  isPinned: Boolean!               # Pinned comment
  local: Boolean!                  # Local content

  # Timestamps
  creationDate: String!            # Comment creation
  updated: String                  # Last edit

  # Computed Statistics (from CommentAggregates)
  score: Int!                      # Net score
  upvotes: Int!                    # Total upvotes
  downvotes: Int!                  # Total downvotes
  replyCount: Int!                 # Child comment count

  # User-Specific Data (requires authentication)
  myVote: Int!                     # Current user's vote
  isSaved: Boolean!                # Saved by current user

  # Relational Data
  creator: User                    # Comment author (DataLoader)
  board: Board                     # Parent board (DataLoader)
  post: Post!                      # Parent post (DataLoader)
}
```

**Database Mapping:**
- Primary table: `comments`
- Aggregates: `comment_aggregates` (scores, child_count)
- Related: `comment_votes`, `comment_saved`

### Vote Types

**PostVote:**
```graphql
type PostVote {
  id: Int!
  userId: Int!
  postId: Int!
  score: Int!                      # -1 (downvote) or 1 (upvote)
  creationDate: String!
}
```

**CommentVote:**
```graphql
type CommentVote {
  id: Int!
  userId: Int!
  commentId: Int!
  postId: Int!                     # For performance joins
  score: Int!                      # -1 (downvote) or 1 (upvote)
  creationDate: String!
}
```

**Database Mapping:**
- Tables: `post_votes`, `comment_votes`
- Unique constraints on (user_id, post_id) and (user_id, comment_id)

## Aggregate Data and Computed Fields

### Performance Optimization Strategy

TinyBoards uses dedicated aggregate tables to avoid expensive real-time calculations:

1. **Aggregate Tables**: Pre-computed statistics stored in separate tables
2. **Database Triggers**: Automatically update aggregates when base data changes
3. **GraphQL Resolvers**: Expose aggregate fields as computed properties

### Aggregate Table Structures

**user_aggregates:**
```sql
- user_id: Int! (FK to users.id)
- post_count: Int! (total posts created)
- post_score: Int! (sum of all post votes)
- comment_count: Int! (total comments created)
- comment_score: Int! (sum of all comment votes)
```

**board_aggregates:**
```sql
- board_id: Int! (FK to boards.id)
- subscribers: Int! (subscriber count)
- posts: Int! (total posts in board)
- comments: Int! (total comments in board)
- users_active_day: Int! (active users in 24h)
- users_active_week: Int! (active users in 7d)
- users_active_month: Int! (active users in 30d)
- users_active_half_year: Int! (active users in 180d)
```

**post_aggregates:**
```sql
- post_id: Int! (FK to posts.id)
- comments: Int! (comment count)
- score: Int! (upvotes - downvotes)
- upvotes: Int! (total upvotes)
- downvotes: Int! (total downvotes)
- newest_comment_time: Timestamp! (last comment time)
- hot_rank: Int! (hot algorithm score)
- hot_rank_active: Int! (active hot score)
- controversy_rank: Float! (controversy score)
- featured_board: Boolean! (featured status)
- featured_local: Boolean! (site-wide featured)
```

**comment_aggregates:**
```sql
- comment_id: Int! (FK to comments.id)
- score: Int! (upvotes - downvotes)
- upvotes: Int! (total upvotes)
- downvotes: Int! (total downvotes)
- child_count: Int! (direct reply count)
- hot_rank: Int! (hot algorithm score)
- controversy_rank: Float! (controversy score)
```

### Computed Field Examples

**User Reputation:**
```rust
// GraphQL resolver in User type
pub async fn rep(&self) -> i64 {
    self.counts.post_score + self.counts.comment_score
}
```

**Admin Status:**
```rust
// GraphQL resolver in User type
pub async fn is_admin(&self) -> bool {
    self.admin_level > 0
}
```

## Data Fetching Patterns

### DataLoader Implementation

TinyBoards uses DataLoader pattern to efficiently batch database queries and avoid N+1 problems:

**Key DataLoader Types:**
```rust
UserId(i32)                    // Batch load users by ID
BoardId(i32)                   // Batch load boards by ID
PostIdForComment(i32)          // Batch load posts for comments
VoteForPostId(i32)             // Batch load user's post votes
VoteForCommentId(i32)          // Batch load user's comment votes
SavedForPostId(i32)            // Batch load user's saved posts
SavedForCommentId(i32)         // Batch load user's saved comments
ModPermsForBoardId(i32)        // Batch load mod permissions
SubscribedTypeForBoardId(i32)  // Batch load subscription status
```

**Example DataLoader Usage:**
```rust
// In Post GraphQL resolver
pub async fn creator(&self, ctx: &Context<'_>) -> Result<Option<User>> {
    let loader = ctx.data_unchecked::<DataLoader<PostgresLoader>>();
    loader.load_one(UserId(self.creator_id)).await.map_err(|e| e.into())
}
```

### Pagination Strategies

**Offset-Based Pagination:**
```graphql
query Posts($limit: Int = 25, $page: Int = 0) {
  listPosts(limit: $limit, page: $page) {
    id
    title
    score
  }
}
```

**Cursor-Based Options:**
- Posts: Use `creationDate` or `hotRank` for cursor
- Comments: Use comment hierarchy and sorting
- Users: Use registration date or reputation

### Filtering and Sorting

**SortType Enum:**
```graphql
enum SortType {
  ACTIVE      # Recently commented
  HOT         # Hot algorithm (trending)
  NEW         # Most recent
  OLD         # Oldest first
  TOP_DAY     # Highest score in 24h
  TOP_WEEK    # Highest score in 7d
  TOP_MONTH   # Highest score in 30d
  TOP_YEAR    # Highest score in 365d
  TOP_ALL     # Highest score all-time
  MOST_COMMENTS       # Most comments
  NEW_COMMENTS        # Recently commented
  CONTROVERSIAL       # High engagement both ways
}
```

**ListingType Enum:**
```graphql
enum ListingType {
  ALL         # All content
  SUBSCRIBED  # Only subscribed boards
  LOCAL       # Only local content
  MODERATED   # Only boards user moderates
}
```

**Filter Examples:**
```graphql
# Hot posts from subscribed boards
query {
  listPosts(sort: HOT, listingType: SUBSCRIBED, limit: 25) {
    id
    title
    score
    board { name }
  }
}

# User's own posts
query {
  user(username: "alice") {
    posts(sort: NEW, limit: 10) {
      id
      title
      creationDate
    }
  }
}
```

## Database Schema Mapping

### Primary Key Relationships

```sql
-- Core entities
users.id → user_aggregates.user_id
boards.id → board_aggregates.board_id
posts.id → post_aggregates.post_id
comments.id → comment_aggregates.comment_id

-- Foreign key relationships
posts.creator_id → users.id
posts.board_id → boards.id
comments.creator_id → users.id
comments.post_id → posts.id
comments.parent_id → comments.id
comments.board_id → boards.id

-- Voting relationships
post_votes.user_id → users.id
post_votes.post_id → posts.id
comment_votes.user_id → users.id
comment_votes.comment_id → comments.id

-- Subscription relationships
board_subscriber.user_id → users.id
board_subscriber.board_id → boards.id

-- Moderation relationships
board_mods.user_id → users.id
board_mods.board_id → boards.id
```

### Join Patterns for Complex Queries

**Posts with Creator and Board:**
```sql
SELECT p.*, pa.*, u.name as creator_name, b.name as board_name
FROM posts p
JOIN post_aggregates pa ON p.id = pa.post_id
JOIN users u ON p.creator_id = u.id
JOIN boards b ON p.board_id = b.id
WHERE p.is_deleted = false AND p.is_removed = false
ORDER BY pa.hot_rank DESC
LIMIT 25;
```

**Comments with Vote Status:**
```sql
SELECT c.*, ca.*, cv.score as my_vote
FROM comments c
JOIN comment_aggregates ca ON c.id = ca.comment_id
LEFT JOIN comment_votes cv ON c.id = cv.comment_id AND cv.user_id = $1
WHERE c.post_id = $2
ORDER BY ca.hot_rank DESC;
```

### Performance Indexes

**Critical Indexes:**
```sql
-- Post queries
CREATE INDEX idx_posts_board_id ON posts(board_id);
CREATE INDEX idx_posts_creator_id ON posts(creator_id);
CREATE INDEX idx_posts_creation_date ON posts(creation_date DESC);
CREATE INDEX idx_post_aggregates_hot_rank ON post_aggregates(hot_rank DESC);
CREATE INDEX idx_post_aggregates_score ON post_aggregates(score DESC);

-- Comment queries
CREATE INDEX idx_comments_post_id ON comments(post_id);
CREATE INDEX idx_comments_parent_id ON comments(parent_id);
CREATE INDEX idx_comment_aggregates_hot_rank ON comment_aggregates(hot_rank DESC);

-- Vote queries
CREATE UNIQUE INDEX idx_post_votes_user_post ON post_votes(user_id, post_id);
CREATE UNIQUE INDEX idx_comment_votes_user_comment ON comment_votes(user_id, comment_id);

-- Board queries
CREATE INDEX idx_board_subscriber_user_id ON board_subscriber(user_id);
CREATE INDEX idx_board_subscriber_board_id ON board_subscriber(board_id);
```

## Security and Privacy Considerations

### Field-Level Access Control

**Private Fields (User):**
- `email`: Only accessible to self and admins
- `hasVerifiedEmail`: Only accessible to self and user admins
- `settings`: Only accessible to self

**Admin-Only Fields (Board):**
- `isHidden`: Only visible to board admins
- `excludeFromAll`: Only visible to board admins

**Context-Dependent Fields:**
- User's vote status: Only available when authenticated
- Mod permissions: Based on user's role in specific board
- Saved status: Personal to each user

### Data Censoring

**Removed/Deleted Content:**
```rust
impl Censorable for Post {
    fn censor(&mut self, my_user_id: i32, is_admin: bool, is_mod: bool) {
        if !(self.is_removed || self.is_deleted) {
            return;
        }

        if is_admin {
            return; // Admins see everything
        }

        // Mods can see removed content, users can see their own
        if self.is_removed && (is_mod || self.creator_id == my_user_id) {
            return;
        }

        let obscure_text = if self.is_deleted {
            "[ deleted by creator ]"
        } else {
            "[ removed by mod ]"
        };

        self.body = obscure_text.clone();
        self.body_html = obscure_text;
    }
}
```

## Example Queries

### Basic Content Fetching

**Get Post with Comments:**
```graphql
query GetPost($id: Int!) {
  post(id: $id) {
    id
    title
    body
    score
    upvotes
    downvotes
    creationDate
    creator {
      id
      name
      avatar
    }
    board {
      id
      name
      title
    }
    comments(sort: HOT, limit: 50) {
      id
      body
      score
      level
      creator {
        name
      }
    }
  }
}
```

**Get Board with Posts:**
```graphql
query GetBoard($name: String!) {
  board(name: $name) {
    id
    name
    title
    description
    subscribers
    posts(sort: HOT, limit: 25) {
      id
      title
      score
      commentCount
      creator {
        name
      }
    }
  }
}
```

**Get User Profile:**
```graphql
query GetUser($username: String!) {
  user(username: $username) {
    id
    name
    bio
    rep
    postCount
    commentCount
    creationDate
    posts(limit: 10, sort: NEW) {
      id
      title
      score
      board {
        name
      }
    }
    comments(limit: 10, sort: NEW) {
      id
      body
      score
      post {
        title
      }
    }
  }
}
```

### Advanced Queries with Filters

**Trending Posts from Subscribed Boards:**
```graphql
query TrendingPosts {
  listPosts(
    sort: HOT
    listingType: SUBSCRIBED
    limit: 25
    page: 0
  ) {
    id
    title
    score
    hotRank
    commentCount
    creator {
      name
      rep
    }
    board {
      name
      title
    }
    myVote
    isSaved
  }
}
```

**User's Moderated Boards:**
```graphql
query MyModeratedBoards {
  me {
    user {
      moderates {
        board {
          id
          name
          title
          subscribers
        }
        permissions
        rank
      }
    }
  }
}
```

This documentation provides a comprehensive overview of the TinyBoards GraphQL data models, their relationships, and efficient data fetching patterns. The API is designed around clear domain boundaries while providing powerful querying capabilities and optimized performance through aggregate data and DataLoader batching.