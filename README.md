# tinyboards

[![Built with Rust](https://img.shields.io/badge/built_with-Rust-orange?logo=rust)](https://www.rust-lang.org)
[![Built with Vue](https://img.shields.io/badge/built_with-Vue-42b883?logo=vue.js)](https://vuejs.org)
[![Discord](https://img.shields.io/discord/1484217141812133938?label=Discord&logo=discord&color=5865F2)](https://discord.gg/7Q99vp5DHa)

a self-hosted platform for building communities you actually own — on hardware you control, with rules you set.

<!-- Add a screenshot or demo GIF here once available -->
<!-- Recommended: a GIF showing the feed, a board, and a post — roughly 1200x800px -->

## Why tinyboards

Most online communities exist on borrowed ground. The platform picks the defaults, controls the algorithm, and can change the terms whenever it wants. tinyboards is built for the people who'd rather run the whole thing themselves — small groups, interest communities, local scenes, professional circles, any group where the members and the people keeping the lights on are part of the same conversation. It's a project being built in the open by people who care about what community software could be if it was designed around the people using it instead of the people selling ads next to it. If that sounds like something you'd want to help shape, you're welcome here — this isn't a solo effort, and it's not trying to be.

## Features

**Content**
- Feed posts (text, link, image) and forum-style threads — two distinct post formats per board
- Nested comment trees with voting
- Post and comment reactions with custom emoji support

**Communities (Boards)**
- Create and customize boards with icons, banners, and color themes
- Per-board wiki with revision history
- Flair system — post flairs and user flairs with filtering
- Board-level moderation tools, mod log, and report queue

**Users**
- Profiles with avatars, bios, and post/comment history
- Follow users, block users, block boards
- Private messaging
- Notifications with per-type settings

**Discovery**
- Full-text search across posts, comments, users, and boards
- Custom feeds (Streams) — curate multi-board feeds and share them with others
- Six sort modes: hot, new, top (day/week/month/year/all), controversial, most comments

**Admin & Moderation**
- Five registration modes: open, email verification, application required, invite-only, closed
- Site-wide and per-board bans with expiry
- Content filtering (word filter, domain blocking)
- Rate limiting, invite codes, application review queue
- Unified moderation log across all action types

**Customization**
- Six built-in themes (light, dark, ocean, forest, sunset, purple)
- Custom emoji — site-scoped and board-scoped
- Board-level reaction settings

**Security**
- Dual-token auth: short-lived JWT (15 min) + hashed refresh token (30 days)
- httpOnly cookies — tokens never accessible to client-side JavaScript
- Row-level security on sensitive tables (messages, notifications, sessions, saved content)
- Argon2 password hashing with random salts

## Tech Stack

| Layer | Technology |
|---|---|
| Backend | Rust + Actix-web 4 |
| API | GraphQL (async-graphql) — single endpoint |
| Database | PostgreSQL 15+ |
| ORM | Diesel 2.1 (diesel-async) |
| Frontend | Nuxt 3 + Vue 3 (SSR) |
| Styling | Tailwind CSS |
| State | Pinia |
| Auth | Dual-token JWT + httpOnly cookies |

## Documentation

Full documentation — including the self-hosting guide, Docker deployment, configuration reference, and API docs — is being written and will live at `[DOCS_URL]`. For now, `deploy/.env.example` is the best reference for configuration options, and `SELF_HOSTING.md` covers deployment step by step.

## Contributing & Community

tinyboards is actively looking for people to build it with. This project is better when more people are involved — whether that means writing Rust resolvers, building Vue components, improving the UX, writing documentation, or just filing detailed bug reports when something breaks. There's no inner circle and no gatekeeping. If you're interested in community software and want to work on something with people who care about getting it right, come say hello.

**Ways to get involved:**
- Join the Discord: https://discord.gg/7Q99vp5DHa — introduce yourself, ask questions, share ideas
- Browse open issues on GitHub — look for `good first issue` labels to find a starting point
- Read `SELF_HOSTING.md` to try running your own instance
- Found a bug? Open an issue with as much detail as you can

---

tinyboards is licensed under the [GNU Affero General Public License v3.0](LICENSE).
This means: if you run a modified version of tinyboards publicly, you must share your modifications under the same license.
