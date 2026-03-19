# API Reference

TinyBoards exposes a **GraphQL API** at a single endpoint. All queries and mutations go through this endpoint.

## Table of Contents

- [Endpoint](#endpoint)
- [Authentication](#authentication)
- [Schema Overview](#schema-overview)
- [Guides](#guides)

## Endpoint

```
POST /api/v2/graphql
```

All requests are HTTP POST with a JSON body containing `query` and optional `variables`:

```bash
curl -X POST https://yourdomain.com/api/v2/graphql \
  -H "Content-Type: application/json" \
  -d '{"query": "{ site { name } }"}'
```

There are two additional HTTP routes:

| Method | Path | Purpose |
|--------|------|---------|
| `GET` | `/media/{filename}` | Serve uploaded media files |
| `GET` | `/` | Health check (returns `ok`) |

## Authentication

TinyBoards uses a dual-token system:

- **Access token** — Short-lived JWT (15 minutes), sent as an `httpOnly` cookie.
- **Refresh token** — Long-lived (30 days), hashed and stored in the `auth_sessions` database table, also sent as an `httpOnly` cookie.

See [authentication.md](authentication.md) for the full auth flow.

For authenticated GraphQL requests, the access token is automatically included via cookies. No `Authorization` header is needed when using a browser.

For programmatic access:

```bash
# Login to get cookies
curl -X POST https://yourdomain.com/api/v2/graphql \
  -H "Content-Type: application/json" \
  -c cookies.txt \
  -d '{
    "query": "mutation { login(usernameOrEmail: \"admin\", password: \"yourpassword\") { jwt } }"
  }'

# Use cookies for subsequent requests
curl -X POST https://yourdomain.com/api/v2/graphql \
  -H "Content-Type: application/json" \
  -b cookies.txt \
  -d '{"query": "{ me { id name } }"}'
```

## Schema Overview

### Queries (20 query types)

| Type | Description |
|------|-------------|
| `MeQuery` | Current user info, unread counts |
| `QueryPosts` | Single post, list posts, hidden posts, threads |
| `QueryComments` | Single comment, list comments with filtering |
| `QueryBoards` | Board by name, list boards |
| `QueryBoardManagement` | Board settings, banned users, moderated boards |
| `QueryUser` | User profiles, followers, settings, history |
| `QuerySite` | Site configuration and statistics |
| `QueryMessages` | Conversations and messages |
| `QueryNotifications` | Notifications with filtering |
| `QueryInvites` | Site invites |
| `QueryBoardModerators` | Board moderator list |
| `QueryBannedUsers` | Site-banned users (admin) |
| `QuerySearch` | Unified search across content types |
| `RegistrationApplicationQueries` | Registration applications (admin) |
| `EmojiQueries` | Custom emoji listing |
| `FlairQueries` | Flair templates, assignments, categories |
| `ReportQueries` | Content reports (mod/admin) |
| `ModerationQueries` | Moderation queue, log, statistics |
| `StreamQueries` | Custom feed streams |
| `QueryWiki` | Board wiki pages and revisions |

### Mutations (37 mutation types)

| Type | Description |
|------|-------------|
| `Auth` | Login, register |
| `UserManagement` | Admin user management |
| `AdminBoardModeration` | Admin board actions |
| `RegistrationApplicationMutations` | Approve/deny applications |
| `BoardActions` | Subscribe/unsubscribe to boards |
| `CreateBoard` | Create new boards |
| `UpdateBoardSettings` | Board settings (mod/admin) |
| `UserActions` | Follow, block users and boards |
| `ProfileManagement` | Update profile |
| `UpdateSettings` | User preferences |
| `SubmitPost` | Create posts |
| `EditPost` | Edit posts |
| `PostActions` | Vote, save, feature, delete, hide |
| `PostModeration` | Remove/approve, lock posts |
| `SubmitComment` | Create comments |
| `EditComment` | Edit comments |
| `CommentActions` | Vote, save, delete comments |
| `CommentModeration` | Remove/approve, pin comments |
| `SendMessageMutations` | Send private messages |
| `EditMessageMutations` | Edit messages |
| `MessageActionMutations` | Mark read, delete messages |
| `SiteConfig` | Update site configuration (admin) |
| `SiteInvite` | Create/delete invites |
| `NotificationMutations` | Manage notifications |
| `ReactionMutations` | Add/remove reactions |
| `ReportMutations` | Report content, resolve reports |
| `BoardModerationMutations` | Board-level moderation |
| `EmojiMutations` | Manage custom emoji |
| `FlairTemplateMutations` | Manage flair templates |
| `FlairAssignmentMutations` | Assign/remove flairs |
| `FlairFilterMutations` | Flair filter preferences |
| `MutationFlairCategories` | Flair category management |
| `ModerationMutations` | Unified moderation actions |
| `StreamManageMutations` | Create/update/delete streams |
| `StreamSubscriptionMutations` | Stream board/flair subscriptions |
| `StreamFollowMutations` | Follow/unfollow streams |
| `FileUploadMutation` | Upload files |
| `CreateWikiPage` / `WikiPageActions` | Wiki page management |

## Guides

| Guide | Description |
|-------|-------------|
| [Auth Endpoints](auth-endpoints.md) | Login, register, refresh, and logout flows with examples |
| [Authentication](authentication.md) | Dual-token system design and cookie flow |
| [Errors](errors.md) | Error format and error codes |
| [Rate Limiting](rate-limiting.md) | Rate limit tiers and response format |
