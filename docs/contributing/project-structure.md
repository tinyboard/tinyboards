# Project Structure

TinyBoards is a monorepo with a Rust backend and Nuxt 3 frontend.

## Table of Contents

- [Top-Level Layout](#top-level-layout)
- [Backend Structure](#backend-structure)
- [Frontend Structure](#frontend-structure)
- [Database Migrations](#database-migrations)

## Top-Level Layout

```
tinyboards-rewrite/
├── backend/                 # Rust backend (Actix-web + async-graphql)
├── frontend/                # Nuxt 3 frontend (Vue 3 + Tailwind)
├── migrations/              # SQL migration files
├── config/                  # nginx and PostgreSQL config templates
├── docker-compose.yml       # Production compose file
├── tinyboards.example.hjson # Configuration template (single source of truth)
├── configure.sh             # Generates .env from tinyboards.hjson
├── backend.Dockerfile       # Backend container build
├── frontend.Dockerfile      # Frontend container build
├── CLAUDE.md                # Project conventions
├── REWRITE_PLAN.md          # Feature map and architecture reference
├── DATABASE_SCHEMA.md       # Schema documentation
└── docs/                    # Documentation (you are here)
```

## Backend Structure

The backend is a Rust workspace with multiple crates:

```
backend/
├── Cargo.toml               # Workspace root
├── crates/
│   ├── api/                 # Main API crate
│   │   ├── src/
│   │   │   ├── main.rs      # Entry point, Actix-web server setup
│   │   │   ├── graphql/     # GraphQL schema
│   │   │   │   ├── mod.rs           # Schema builder, root Query + Mutation
│   │   │   │   ├── queries/         # All query resolvers
│   │   │   │   │   ├── me.rs            # MeQuery (current user)
│   │   │   │   │   ├── posts.rs         # QueryPosts
│   │   │   │   │   ├── comments.rs      # QueryComments
│   │   │   │   │   ├── boards.rs        # QueryBoards
│   │   │   │   │   ├── board_management.rs
│   │   │   │   │   ├── user.rs          # QueryUser
│   │   │   │   │   ├── site.rs          # QuerySite
│   │   │   │   │   ├── messages.rs      # QueryMessages
│   │   │   │   │   ├── notifications.rs
│   │   │   │   │   ├── invites.rs
│   │   │   │   │   ├── board_moderators.rs
│   │   │   │   │   ├── banned_users.rs
│   │   │   │   │   ├── search.rs
│   │   │   │   │   ├── registration_applications.rs
│   │   │   │   │   ├── emojis.rs
│   │   │   │   │   ├── flairs.rs
│   │   │   │   │   ├── reports.rs
│   │   │   │   │   ├── moderation_unified.rs
│   │   │   │   │   ├── streams.rs
│   │   │   │   │   └── wiki.rs
│   │   │   │   └── mutations/       # All mutation resolvers
│   │   │   │       ├── auth.rs          # Login, register
│   │   │   │       ├── admin/           # Admin mutations
│   │   │   │       ├── board/           # Board mutations
│   │   │   │       ├── user/            # User mutations
│   │   │   │       ├── post/            # Post mutations
│   │   │   │       ├── comment/         # Comment mutations
│   │   │   │       ├── message/         # Message mutations
│   │   │   │       ├── site/            # Site config mutations
│   │   │   │       ├── stream/          # Stream mutations
│   │   │   │       ├── flair/           # Flair mutations
│   │   │   │       ├── wiki/            # Wiki mutations
│   │   │   │       ├── notifications.rs
│   │   │   │       ├── reactions.rs
│   │   │   │       ├── reports.rs
│   │   │   │       ├── board_moderation.rs
│   │   │   │       ├── emoji.rs
│   │   │   │       ├── flair_categories.rs
│   │   │   │       ├── moderation_unified.rs
│   │   │   │       └── file_upload.rs
│   │   │   ├── middleware/  # Actix middleware (auth, rate limiting)
│   │   │   └── errors.rs   # Error types
│   │   └── Cargo.toml
│   ├── db/                  # Database layer
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── schema.rs   # Diesel auto-generated schema
│   │   │   ├── models/     # Diesel model structs
│   │   │   └── queries/    # Database query functions
│   │   └── Cargo.toml
│   └── utils/               # Shared utilities
│       ├── src/
│       │   ├── lib.rs
│       │   └── ...          # Hashing, validation, etc.
│       └── Cargo.toml
└── docker/                  # Docker configs (dev/prod compose files)
```

### Key Backend Patterns

- **GraphQL schema** — Built with `async-graphql`. All queries and mutations are merged into a single root `Query` and `Mutation` type.
- **Module pattern** — Each feature area (posts, comments, flairs, etc.) has its own query and mutation files.
- **Database** — Diesel 2.1 with `diesel-async` and `bb8` connection pool.
- **Error handling** — `thiserror`-based error types that map to GraphQL errors.
- **File storage** — OpenDAL abstraction supporting local filesystem, S3, Azure, and GCS.

## Frontend Structure

```
frontend/
├── nuxt.config.ts            # Nuxt configuration
├── app.vue                   # Root Vue component
├── pages/                    # File-based routing
│   ├── index.vue                 # / → redirect to /home
│   ├── home/[[sort]].vue         # Home feed
│   ├── all/[[sort]].vue          # All posts feed
│   ├── login.vue                 # Login page
│   ├── register.vue              # Registration page
│   ├── boards.vue                # Board directory
│   ├── members/[[sort]].vue      # Members directory
│   ├── search.vue                # Search
│   ├── submit.vue                # New post
│   ├── @[username]/              # User profiles
│   │   ├── index.vue
│   │   ├── posts.vue
│   │   ├── comments.vue
│   │   └── saved.vue
│   ├── b/[board]/                # Board routes
│   │   ├── index.vue
│   │   ├── feed/
│   │   ├── threads/
│   │   ├── flair/
│   │   ├── flairs/               # Flair management
│   │   ├── mod/                  # Moderation panel
│   │   └── wiki/                 # Board wiki
│   ├── settings/                 # User settings
│   ├── inbox/                    # Messages
│   ├── streams/                  # Custom feeds
│   ├── admin/                    # Admin panel
│   ├── createBoard/              # Board creation wizard
│   └── help/                     # Help pages
├── components/               # Vue components
│   ├── cards/                    # Post, Comment, Board cards, etc.
│   ├── containers/               # Sidebar, CommentSection, etc.
│   ├── dialogs/                  # Modal components
│   ├── flair/                    # Flair display, editor, management
│   ├── input/                    # Form inputs, TipTap editor
│   ├── nav/                      # Navbar, Breadcrumbs, Pagination
│   ├── lists/                    # Post and comment lists
│   ├── menus/                    # Dropdown menus
│   └── streams/                  # Stream components
├── composables/              # Vue composables (shared logic)
│   ├── useGraphQL.ts             # Main GraphQL client
│   ├── api.ts                    # Legacy REST wrapper
│   ├── graphql_multipart.ts      # File upload support
│   ├── admin.ts                  # Admin operations
│   ├── comments.ts               # Comment operations
│   ├── posts.ts                  # Post list operations
│   ├── useStream.ts              # Stream operations
│   └── ...                       # 33 composables total
├── stores/                   # Pinia state stores
│   ├── StoreAuth.ts              # Auth state, token management
│   ├── StoreBoard.ts             # Current board state
│   ├── StoreComments.ts          # Comment tree
│   ├── StorePosts.ts             # Post list
│   ├── StoreSite.ts              # Site configuration
│   └── ...                       # 11 stores total
├── middleware/                # Route middleware
│   ├── 01.site.global.js         # Load site config on every route
│   ├── 02.auth.global.js         # Check auth state
│   ├── admin.global.js           # Admin route guard
│   └── guards.global.js          # Auth route guards
├── layouts/                  # Page layouts
│   ├── default.vue               # Standard layout
│   ├── admin.vue                 # Admin panel layout
│   └── ...                       # 8 layouts total
├── assets/                   # Static assets, CSS
├── public/                   # Public static files
└── plugins/                  # Nuxt plugins
```

### Key Frontend Patterns

- **File-based routing** — Pages in `pages/` map directly to URL routes. See [adding-pages.md](adding-pages.md) for examples.
- **GraphQL client** — `composables/useGraphQL.ts` handles all API communication with retry logic, deduplication, and health monitoring.
- **State management** — Pinia stores manage client-side state. The `StoreAuth` store handles token lifecycle.
- **Layouts** — Different page layouts for the main site, admin panel, settings, etc.
- **Middleware** — Global middleware loads site config and checks auth on every navigation.

## Database Migrations

```
migrations/
├── 2026-03-03-000001_foundation/        # Extensions, enums, utility functions
├── 2026-03-03-000002_core_tables/       # site, users, boards, board_moderators
├── 2026-03-03-000003_content/           # posts, comments, votes (partitioned)
├── 2026-03-03-000004_aggregates/        # *_aggregates tables
├── 2026-03-03-000005_social/            # Subscriptions, follows, blocks, saves
├── 2026-03-03-000006_messaging_notifications/  # Messages, notifications
├── 2026-03-03-000007_moderation/        # Moderation log, reports, legacy mod tables
├── 2026-03-03-000008_media_emoji_reactions/    # Uploads, emoji, reactions
├── 2026-03-03-000009_flairs/            # Flair system
├── 2026-03-03-000010_streams/           # Custom feed streams
├── 2026-03-03-000011_wiki/              # Wiki pages and revisions
├── 2026-03-03-000012_auth_config_rls/   # Auth config, RLS policies
├── 2026-03-03-000013_seed_data/         # Default site config, languages
└── 2026-03-03-000014_auth_sessions/     # Refresh token sessions
```

Each migration has `up.sql` (apply) and `down.sql` (revert). Migrations run automatically when the backend starts.
