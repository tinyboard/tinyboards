# CLAUDE.md — Project Context for Claude Code

Read this fully before touching any code.

---

## Living Document Policy

This file is the single source of project context. There are no other planning,
audit, or status documents — everything lives here.

**Every session must:**
- Read this file at the start
- Update this file before ending if anything changed: bugs fixed, features
  completed, new bugs found, architecture decisions made, or remaining work
  reprioritized
- Add a dated entry to the Session Log at the bottom when meaningful work is done

**Never** create separate planning, status, or audit markdown files at the repo
root. If information is worth preserving, it belongs in this file. Keep sections
concise — summarize, don't append endlessly. When a bug is fixed, move it from
Known Bugs to the Session Log entry and delete it from the bug list. When a
feature is complete and verified, move it to the "Fully Working" list and remove
it from "Code Complete" or "Not Yet Implemented."

---

## Project Overview

**tinyboards** is a self-hosted social media platform — a Reddit-like forum with
boards, posts, comments, voting, moderation, private messaging, notifications,
user profiles, flairs, custom emoji, wiki pages, and a full
admin panel.

A single operator deploys the instance, configures registration modes
(open, invite-only, application-required, closed), and manages it via a web
admin panel.

This monorepo combines what was originally two separate repositories (backend
and frontend) into a single codebase.

---

## Tech Stack

| Layer | Technology |
|---|---|
| Frontend | Nuxt 3 (Vue 3, Composition API, TypeScript) |
| Styling | TailwindCSS with custom theme system (6 themes) |
| State | Pinia (3 stores: auth, site, ui) |
| Data | GraphQL via BFF proxy (useGraphQL composable) |
| Backend | Rust, Actix-web 4.3 |
| GraphQL | async_graphql with DataLoader for N+1 prevention |
| Database | PostgreSQL, diesel-async 0.3, diesel 2.1 |
| Auth | JWT access tokens (15 min), hashed refresh tokens (30 days), Argon2 |
| Storage | OpenDAL (supports fs, s3, azure, gcs) |
| Deployment | Docker Compose (postgres + backend + frontend + nginx) |
| Configuration | tinyboards.hjson (single config file, generates .env via configure.sh) |

---

## Architecture

```
Browser
    |
    v
nginx (port 80/443)
    |-- /api/v2/*  --> Backend (Rust, port 8536)
    |-- /media/*   --> Backend (media serving)
    |-- /api/*     --> Frontend BFF (Nuxt server routes, port 3000)
    |-- /*         --> Frontend SSR (Nuxt, port 3000)
    |
Frontend (Nuxt 3, Port 3000)
    |-- BFF Proxy (/server/api/graphql.ts)
    |-- Auth REST proxy (/server/api/auth/*.ts)
    |-- SSR middleware reads httpOnly cookies, populates Pinia stores
    v
Backend (Rust/Actix-web, Port 8536)
    |-- GraphQL endpoint: POST /api/v2/graphql
    |-- Auth REST endpoints: /api/v2/auth/*
    |-- async_graphql schema with DataLoader
    |-- diesel-async for PostgreSQL
    v
PostgreSQL
```

Key design decisions:
- **BFF Pattern**: All browser traffic goes through Nuxt server routes, never directly to the Rust backend
- **Auth via REST**: Login/register/refresh/logout are REST endpoints (not GraphQL mutations), proxied through Nuxt BFF routes
- **httpOnly cookies only**: `tb_access` (15 min JWT) and `tb_refresh` (30 days, hashed in DB)
- **CSRF Protection**: X-Requested-With header validation on all auth endpoints
- **SSR Auth**: Nitro server middleware reads cookies and populates Pinia stores before render

---

## Repository Structure

```
tinyboards-rewrite/
├── backend/
│   ├── src/
│   │   ├── main.rs                 # Actix server setup, pool, CORS
│   │   ├── api_routes.rs           # Route config: graphql, media, health
│   │   ├── media_handler.rs        # OpenDAL-backed media serving
│   │   ├── scheduled_tasks.rs      # Background jobs (partition creation)
│   │   └── code_migrations.rs      # Runtime data migrations
│   └── crates/
│       ├── api/src/                 # GraphQL resolvers, DataLoaders, storage
│       │   ├── mutations/           # All mutation resolvers (13 modules)
│       │   ├── queries/             # All query resolvers
│       │   ├── loaders/             # DataLoader implementations
│       │   ├── structs/             # GraphQL type definitions
│       │   ├── storage/             # File storage (OpenDAL)
│       │   └── helpers/             # Validation, file upload, notifications
│       ├── auth/src/                # Auth REST handlers, JWT, sessions, middleware, cookies
│       ├── db/src/                  # Diesel models, schema.rs (50+ tables), CRUD traits
│       └── utils/src/               # Settings, errors, content filtering, slugs
├── frontend/
│   ├── pages/                       # ~78 route files
│   ├── components/                  # ~64 components
│   ├── composables/                 # ~18 composables (all with GraphQL integration)
│   ├── stores/                      # Pinia: auth, site, ui
│   ├── layouts/                     # default, admin, settings, auth, error
│   ├── server/
│   │   ├── api/graphql.ts           # BFF proxy (auth, CSRF, refresh)
│   │   ├── api/auth/*.ts            # 10 REST route proxies
│   │   ├── api/logout.ts            # Cookie clearing
│   │   ├── routes/robots.txt.ts     # Dynamic robots.txt
│   │   ├── routes/sitemap.xml.ts    # Dynamic sitemap
│   │   └── middleware/auth.ts       # SSR auth context population
│   ├── middleware/                   # Route guards (auth, admin, site loader)
│   ├── types/                       # TypeScript types (generated.ts)
│   └── schema.graphql               # Frontend GraphQL schema
├── migrations/                      # Diesel migration files (up.sql/down.sql)
├── schema.graphql                   # Root GraphQL schema (documentation/reference)
├── nginx/                           # nginx configs (HTTP + SSL)
├── deploy/
│   ├── postgres/                    # PostgreSQL tuning configs (1gb/2gb/4gb)
│   ├── systemd/                     # systemd service files
│   ├── scripts/                     # SSL setup script
│   └── .env.example                 # Environment variable documentation
├── scripts/                         # backup.sh, restore.sh
├── docker-compose.yml               # Production deployment
├── docker-compose.dev.yml           # Development
├── backend.Dockerfile
├── frontend.Dockerfile
├── tinyboards.example.hjson         # Configuration template
├── configure.sh                     # Generates .env from tinyboards.hjson
├── SELF_HOSTING.md                  # Self-hosting deployment guide
└── CLAUDE.md                        # This file
```

---

## Code Conventions

### Rust
- thiserror for custom errors; anyhow only in main.rs
- All handlers return Result<HttpResponse, AppError>
- No unwrap() or expect() in non-test code
- tracing for logging — no println!
- Module pattern: mod.rs + handlers.rs + models.rs + queries.rs
- diesel-async for all database operations (not sqlx)
- async_graphql derives: SimpleObject, ComplexObject, InputObject

### Nuxt / Vue
- Nuxt 3 App Router with file-based routing
- Composition API with `<script setup lang="ts">`
- Server Components by default; `use client` only when necessary
- All GraphQL operations go through `useGraphQL` composable → BFF proxy
- PascalCase components, camelCase utilities
- `useSeoMeta()` for page-level SEO
- `useToast()` for user feedback on mutations

### Database
- snake_case tables and columns
- Integer primary keys (auto-increment), not UUIDs
- created_at / updated_at on every table
- Soft deletes via `is_deleted` or `deleted` boolean where applicable
- Diesel embedded migrations in /migrations/ (up.sql/down.sql pairs)
- `notifications` table is range-partitioned by month (post_votes and comment_votes are NOT partitioned — they are standard tables)
- Source of truth: `backend/crates/db/src/schema.rs` (auto-generated by diesel)

### GraphQL Schema Sync Protocol
- `schema.graphql` in the repo root is the documentation/reference copy
- `frontend/schema.graphql` is the frontend's working copy
- async_graphql generates the actual schema from Rust structs at runtime

When adding a new query or mutation (backend first):
1. Update the async-graphql resolver in `backend/crates/api/src/`
2. Update both `schema.graphql` files to match
3. Run: `cd frontend && npm run codegen`
4. Fix any TypeScript errors
5. Commit schema + generated types + code changes together

When adding a new query or mutation (frontend first):
1. Add the operation to both `schema.graphql` files
2. Run: `cd frontend && npm run codegen`
3. Write the frontend composable/page using generated types
4. Implement the backend resolver to match
5. Commit everything together

Never edit `frontend/types/generated.ts` by hand. Codegen runs automatically
before `npm run dev` and `npm run build`.

---

## Feature Status

### Fully Working (End-to-End Verified)
Login/register/logout, token refresh, SSR auth hydration, home feed
(subscribed + all), board viewing + subscription, post viewing + voting,
comment tree + voting + replies, user profiles + tabs, board directory,
search (multi-type), notifications, saved items, follow/block users.

### Code Complete (Not Runtime Verified)
Post creation, comment editing, messages (inbox + threads), all settings
pages (account, profile, notifications, security, privacy, appearance),
full admin panel (dashboard, site settings, users, bans, invites, content,
reports, mod queue), board settings (general, moderation, appearance),
board mod tools (bans, queue, log), wiki (list, view, create, edit,
revisions), flairs (management + post assignment),
toast notifications, SEO (useSeoMeta + JSON-LD + robots.txt + sitemap),
notification polling, thread system, rich user profiles (background,
avatar frame, karma, signature), link preview rendering (YouTube embeds,
video, thumbnails), distinguish feature (admin/mod badges on posts and
comments with toggle).

### Not Yet Implemented
- Email sending (SMTP) — blocks password reset & verification
- File upload for board icons/banners (uses URL strings, not direct upload)
- Wiki deletion + revert (backend mutations exist, no UI)
- User flair assignment to profiles
- Flair categories & filters
- Help/legal pages (route shells only)

---

## Auth System

### REST Endpoints (all registered in main.rs)
| Endpoint | Status |
|---|---|
| `POST /api/v2/auth/login` | Working |
| `POST /api/v2/auth/register` | Working (all reg modes) |
| `POST /api/v2/auth/logout` | Working |
| `POST /api/v2/auth/logout-all` | Working |
| `POST /api/v2/auth/refresh` | Working (rotates session) |
| `POST /api/v2/auth/change-password` | Working |
| `POST /api/v2/auth/password-reset/request` | Logs token (no email) |
| `POST /api/v2/auth/password-reset/complete` | Validates hashed token |
| `POST /api/v2/auth/email/verify` | Marks user verified |
| `POST /api/v2/auth/email/request-verification` | Logs token (no email) |

### Security Measures
- CSRF guard on all auth REST endpoints
- Auth middleware on protected endpoints
- httpOnly cookies (no client-side JS access)
- Argon2 password hashing
- Refresh token hashed before DB storage
- Token rotation on refresh
- Session invalidation on password change

---

## GraphQL API

### Queries (26+)
`me`, `site`, `siteStats`, `user`, `listUsers`, `searchUsernames`,
`getUserSettings`, `board`, `listBoards`, `post`, `listPosts`, `comment`,
`comments`, `getNotifications`, `getUnreadNotificationCount`,
`listConversations`, `getConversation`, `getUnreadMessageCount`,
`searchContent`, `getBoardModerators`, `getModeratedBoards`,
`getBoardSettings`, `getBoardBannedUsers`, `listBannedUsers`,
`getPostReports`, `getCommentReports`, `listRegistrationApplications`,
`listInvites`, `getWikiPage`, `listWikiPages`, `wikiPageHistory`,
`getBoardFlairs`, `getFlairTemplate`, `getModerationLog`

### Mutations (38+)
**Posts:** create, edit, vote, save/unsave, hide/unhide, lock/unlock, feature, remove, restore, distinguish
**Comments:** create, edit, vote, save/unsave, remove, restore, distinguish
**Boards:** create, updateSettings
**Users:** follow/unfollow, block/unblock, updateSettings, updateProfile
**Messages:** send, edit, delete
**Notifications:** markRead, delete
**Reports:** reportPost, reportComment, resolve, dismiss
**Admin:** updateSiteConfig, banUserFromSite, unbanUserFromSite, createInvite, deleteAccount
**Moderation:** banUserFromBoard, unbanUserFromBoard, addModerator, removeModerator
**Wiki:** createWikiPage, editWikiPage, deleteWikiPage
**Flairs:** createFlairTemplate, updateFlairTemplate, deleteFlairTemplate, assignPostFlair, removePostFlair
**Files:** uploadFile

---

## Known Bugs (Unfixed)

### Critical
| ID | Issue | File | Fix Target |
|---|---|---|---|
| BUG-001 | Empty salt in dev mode makes password hashes predictable | `utils/src/passhash.rs:9-15` | Auth cleanup |
| BUG-002 | Old Claims struct with `validate_exp = false` still exported | `utils/src/claims.rs` | Delete the file |
| BUG-003 | Old non-httpOnly cookie pattern in StoreAuth.ts | `stores/StoreAuth.ts:15` | Remove old store |
| BUG-004 | localStorage/sessionStorage fallback for auth token | `composables/useGraphQL.ts:208-209` | Remove fallback |

### High
| ID | Issue | File |
|---|---|---|
| BUG-005 | Hardcoded default secrets (`somesalt`, `password`) — no startup validation | `utils/src/settings/structs.rs` |
| BUG-006 | Undefined `isServerSide` variable in useGraphQLRequest | `composables/useGraphQL.ts:512,518` |
| BUG-007 | `StatusCode::from_u16().expect()` in error handler can panic | `utils/src/error.rs:139` |
| BUG-008 | `pool.get().await.unwrap()` throughout reactions module (15 calls) | `db/src/impls/reaction/reactions.rs` |
| BUG-009 | `expect()` calls in scheduled_tasks crash background thread | `scheduled_tasks.rs:15,51,61,90,101` |
| BUG-010 | SQL built via `format!()` in scheduled_tasks | `scheduled_tasks.rs:89` |
| BUG-011 | `expect()` in email sending crashes request handlers | `utils/src/email.rs:41,52,56,64` |
| BUG-012 | Extensive `v-html` with no client-side sanitization (26+ instances) | Frontend components |
| BUG-013 | `unwrap()` on Option in board_mods can panic | `db/src/impls/board/board_mods.rs:253-254` |

### Medium (summary — 20 bugs)
- `unwrap()` in passhash functions (BUG-014)
- SSR disabled for almost all routes via nuxt.config.ts route rules (BUG-015)
- Regex compiled on every call instead of Lazy (BUG-016, BUG-017)
- `eprintln!()`/`println!()` used instead of tracing in 30+ locations (BUG-018)
- Old Claims struct dead code still exported (BUG-019)
- `std::sync::Mutex` in async context for rate limiting (BUG-020)
- `data_unchecked` used in 80+ GraphQL resolver call sites (BUG-021)
- Check-then-unwrap anti-pattern on Options (BUG-022)
- Various `unwrap()` on Option/Result without type-level guarantees (BUG-023, BUG-024, BUG-029, BUG-030, BUG-032, BUG-033)
- `format!()` for SQL in active_counts (BUG-025)
- `require("tls")` CommonJS in ESM nuxt.config.ts (BUG-026)
- No client-side markdown sanitization before v-html (BUG-027)
- Inconsistent admin permission checks for registration applications (BUG-028)
- Duplicated inline GraphQL queries in auth store (BUG-031)

### Low (summary — 11 bugs)
Dead code with `#[allow(dead_code)]`, `expect()` in Lazy regex (defensible),
startup `expect()` with terse messages, TODO/FIXME comments (25+ locations),
console logging in production frontend (50+ locations), `js-cookie` alongside
`useCookie`, scheduler loop never returns, sync Diesel in scheduled_tasks,
infallible `unwrap()` on date/time values.

---

## Remaining Work — Prioritized

### Tier 1: Production Blockers
1. **Email sending** — Password reset and verification non-functional without SMTP
2. **Docker validation** — Full stack build+run never tested with Docker daemon
3. **Runtime testing** — Many features are code-complete but untested with real data
4. **Remove old auth code paths** — Old Claims struct, non-httpOnly patterns still exported

### Tier 2: Quality & Correctness
5. **Backend unwrap()/expect() cleanup** — Production panics waiting to happen (BUG-007/008/009/011/013)
6. **SSR re-enablement** — Almost all routes have `ssr: false`, defeating Nuxt's purpose (BUG-015)
7. **Client-side HTML sanitization** — DOMPurify added for wiki but not all v-html usage (BUG-012/027)
8. **Compiler warnings** — 43 warnings in tinyboards_api, 1 in tinyboards_server
9. **Replace println/eprintln with tracing** (BUG-018)

### Tier 3: Feature Gaps
10. File upload integration for board icons/banners
11. Registration applications admin flow (untested)
12. Wiki deletion + revert UI
13. Advanced flair features (user assignment, categories, filters)

### Tier 4: Polish
14. Loading skeletons (currently only spinners)
15. Error boundaries for SSR failures
16. Help/legal pages
17. Mobile experience (responsive sidebar, bottom nav)
18. Micro-interactions (vote animations, page transitions)

---

## Git Rules — Non-Negotiable

1. Never run `git commit`. Stage with `git add` only.
   The developer writes every commit message manually.
2. Never mention Claude, AI, or any tool in commit messages,
   code comments, or documentation.
3. Suggested commit messages (only when asked) use conventional format:
   `feat(auth): implement JWT refresh token flow`
   `fix(posts): correct pagination offset`
   No co-author lines. No attribution of any kind.
   Never append session links, tracking URLs, or any external references to
   commit messages.
4. Code comments explain what and why, as any developer would write them.

---

## Working Principles

1. One module, one file at a time.
2. Preserve existing behavior unless it is a confirmed bug.
3. Write tests alongside new code, not after.
4. Read the code before suggesting changes — do not assume.
5. When touching the database, read `backend/crates/db/src/schema.rs` and relevant migrations first.
6. When touching GraphQL, follow the schema sync protocol above.

---

## Session Startup Checklist

1. Read CLAUDE.md in full
2. Read the actual code in the area you're working on
3. Check the Known Bugs section for bugs relevant to the current module
4. Check the Remaining Work section for priority context

---

## Session Log

### 2026-03-08 — Phase 1 Audit + Frontend Build
- Audited entire codebase (backend + frontend), cataloged 45 bugs
- Built all 78 frontend pages, 64 components, 18 composables from scratch

### 2026-03-08 to 2026-03-09 — Integration (Sessions 1–5)
- Registered auth REST routes in main.rs (were missing)
- Moved login/register from GraphQL to REST-only with CSRF protection
- Fixed SSR auth (cookie names, cookie forwarding, server middleware rewrite)
- Fixed JWT Claims struct mismatch between crates
- Fixed refresh cookie path blocking BFF refresh
- Fixed admin panel schema mismatches (enableNSFW, isPrivate, updateSiteConfig, ban/unban mutations)
- Implemented toast notification system
- Implemented admin reports pages (posts + comments)
- Fixed pagination hasMore pattern across 9 files (limit+1 fetch)
- Implemented board settings (general + moderation)
- Added subscribed boards to auth store (fixed FeedSidebar crash)
- First live run: auth flow validated end-to-end

### 2026-03-11 — Session 6: Feature Completion
- Wiki: 4 pages (list, view, create, edit) + revision history
- Flairs: management page + post flair selector on submit + flair edit page
- Board mod tools: ban management, mod queue, mod log
- Board appearance: icon/banner upload, color pickers
- Fixed wiki and flair composables to match backend schema field names

### 2026-03-19 — Session 7: Gap Closing (SEO, Notifications, Threads, Profiles, Link Previews, Deployment)
- SEO: useSeoMeta on all key pages, JSON-LD structured data, dynamic robots.txt + sitemap.xml
- Notification polling: 30s interval, tab visibility aware, header badge
- Thread system: forum-style thread list layout
- Rich user profiles: profile background, avatar frame, karma display, signature rendering
- Link previews: YouTube embeds, video elements, thumbnails, link preview cards
- PostgreSQL tuning: docker-compose wired to use profile-based config
- Backup/restore scripts: pg_dump --format=custom, configurable retention
- Image optimization: added optimized_url + processing_status columns, updated upload handler
- Environment variable documentation in deploy/.env.example
- Cleaned up session artifact files, consolidated all docs into CLAUDE.md

### 2026-03-19 — Loose Ends Cleanup (Pre-Phase 7)
- **Partition discrepancy resolved**: Confirmed post_votes and comment_votes are NOT partitioned (standard tables per 003_content.sql). Only notifications uses range partitioning. scheduled_tasks.rs correctly handles only notifications partitions — no changes needed.
- **Root schema.graphql synced to Rust implementation**: Added missing User fields (signature, profileMusic, profileMusicYoutube), Post fields (isThread, hotRankActive, controversyRank, isSaved, reactionCounts, myReaction, flairs, myModPermissions, lastCrawlDate), Comment fields (quotedCommentId, isSaved, reactionCounts, myReaction), PrivateMessage fields (bodyHTML, isSenderHidden, isDeleted, creator, recipient), WikiPage fields (creatorId, lastEditedBy, isLocked, isDeleted), UpdateProfileInput fields (profileBackground, avatarFrame, profileMusic, profileMusicYoutube, signature), EditWikiPageInput (viewPermission, editPermission), FlairStyle output type.
- **Frontend vs root schema intentional differences**: Frontend schema omits admin-only mutations (approveApplication, denyApplication, adminBanBoard, adminUnbanBoard, excludeBoardFromAll, transferBoardOwnership, revertWikiPage, user flair ops, flair categories, reactions, updateEmoji, deleteInvite). Frontend Board type omits isSubscribed and sectionConfig. Frontend FlairTemplate is simplified (fewer fields). These are intentional — frontend only includes what it consumes.
- **Upload API docs clarified**: Added inline comments to upload handler explaining synchronous processing model and future async path. Added column comment to 015_image_processing migration. Added image processing config reference to deploy/.env.example.

### 2026-03-22 — Session 8: Remove Streams Feature
- Removed the entire streams feature (too complex for initial small-community focus; board directory + home feed is sufficient)
- Deleted: 6 frontend pages, 4 components, 2 composables, all backend resolvers (queries + mutations + helpers), DB models, stream tables from schema.rs
- Added migration 000018_remove_streams to drop all 9 stream-related tables and their triggers
- Cleaned both GraphQL schema files (root + frontend) of all stream types, queries, and mutations
- Removed stream references from AppNav, AppSidebar, HomeSidebar, AllSidebar
- Removed streams user guide documentation
- Updated CLAUDE.md: project overview, feature status, GraphQL API lists, remaining work, session log

### 2026-03-27 — Session 9: Admin Level Fix + Distinguish Feature
- **Fixed initial admin level**: Changed from 256 to 7 (Owner) in code_migrations.rs. Level 256 was outside the valid 0-7 range and broke admin management.
- **Owner-level admin promotion**: Updated set_user_admin_level and delete_account mutations so level 7 (Owner) admins can promote other users to Owner level and manage peer owners. Non-owner admins still cannot promote to their own level.
- **Distinguish feature (admin/mod badges)**: Added `distinguished_as` nullable column to posts and comments tables (migration 000020). Values: NULL (not distinguished), 'admin', 'mod'. Admins and board mods can toggle a distinguish badge on their own posts/comments via `distinguishPost` and `distinguishComment` GraphQL mutations. Badge shows as green "Admin" or blue "Mod" in PostCard, PostDetail, and CommentItem. Toggle button appears in moderation menus for own content only.
- Updated both GraphQL schema files, generated types, and all relevant frontend queries/composables
