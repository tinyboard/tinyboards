# tinyboards

[![Built with Rust](https://img.shields.io/badge/built_with-Rust-orange?logo=rust)](https://www.rust-lang.org)
[![Built with Vue](https://img.shields.io/badge/built_with-Vue-42b883?logo=vue.js)](https://vuejs.org)
[![License: AGPL v3](https://img.shields.io/badge/license-AGPL--v3-blue)](LICENSE)
[![Discord](https://img.shields.io/discord/1484217141812133938?label=Discord&logo=discord&color=5865F2)](https://discord.gg/7Q99vp5DHa)

A self-hosted platform for building communities you actually own — on hardware you control, with rules you set.

<!-- Add a screenshot or demo GIF here once available -->

---

## Why tinyboards

Most online communities exist on borrowed ground. The platform picks the defaults, controls the algorithm, and can change the terms whenever it wants. tinyboards is built for the people who'd rather run the whole thing themselves — small groups, interest communities, local scenes, professional circles, any group where the members and the people keeping the lights on are part of the same conversation.

It's a project being built in the open by people who care about what community software could be if it was designed around the people using it instead of the people selling ads next to it. If that sounds like something you'd want to help shape, you're welcome here.

## Features

**Content & Discussion**
- Feed posts (text, link, image) and forum-style threads — two distinct post formats per board
- Nested comment trees with voting
- Post and comment reactions with custom emoji support

**Communities (Boards)**
- Create and customize boards with icons, banners, and color themes
- Per-board wiki with revision history
- Flair system — post flairs and user flairs with filtering
- Board-level moderation tools, mod log, and report queue

**Users & Social**
- Profiles with avatars, bios, and post/comment history
- Follow users, block users, block boards
- Private messaging and notifications with per-type settings

**Discovery**
- Full-text search across posts, comments, users, and boards
- Custom feeds (Streams) — curate multi-board feeds and share them with others
- Six sort modes: hot, new, top (day/week/month/year/all), controversial, most comments

**Administration**
- Five registration modes: open, email verification, application required, invite-only, closed
- Site-wide and per-board bans with expiry
- Content filtering (word filter, domain blocking)
- Rate limiting, invite codes, application review queue

**Theming & Customization**
- Six built-in themes (light, dark, ocean, forest, sunset, purple)
- Custom emoji — site-scoped and board-scoped
- Board-level reaction settings

**Security**
- Dual-token auth: short-lived JWT (15 min) + hashed refresh token (30 days)
- httpOnly cookies — tokens never accessible to client-side JavaScript
- Row-level security on sensitive tables
- Argon2 password hashing with random salts

## Tech Stack

| Layer | Technology |
|-------|------------|
| Backend | Rust + [Actix-web](https://actix.rs/) 4 |
| API | GraphQL ([async-graphql](https://github.com/async-graphql/async-graphql)) |
| Database | PostgreSQL 15+ with [Diesel](https://diesel.rs/) 2.1 (diesel-async) |
| Frontend | [Nuxt 3](https://nuxt.com/) + [Vue 3](https://vuejs.org/) (SSR) |
| Styling | [Tailwind CSS](https://tailwindcss.com/) |
| State | [Pinia](https://pinia.vuejs.org/) |
| Auth | Dual-token JWT + httpOnly cookies |
| Storage | [OpenDAL](https://opendal.apache.org/) (fs, S3, Azure, GCS) |
| Deployment | Docker Compose |

## Quick Start

```bash
git clone https://github.com/tinyboard/tinyboards.git
cd tinyboards
cp .env.example .env

# Generate secrets and set your domain
sed -i "s/changeme_generate_with_openssl_rand_base64_48/$(openssl rand -base64 48)/" .env
sed -i "s/changeme_generate_with_openssl_rand_hex_32/$(openssl rand -hex 32)/" .env
sed -i "s/DOMAIN=example.com/DOMAIN=yourdomain.com/" .env

docker compose up -d
```

Visit `https://yourdomain.com` to complete initial setup. See the [Self-Hosting Guide](docs/self-hosting/) for the full walkthrough.

## Architecture

```
                    ┌─────────┐
  Internet ────────►│  nginx  │
                    │ :80/443 │
                    └────┬────┘
                         │
              ┌──────────┼──────────┐
              ▼                     ▼
        ┌──────────┐         ┌──────────┐
        │ Frontend │         │ Backend  │
        │  Nuxt 3  │────────►│ Actix-web│
        │  :3000   │ GraphQL │  :8536   │
        └──────────┘         └────┬─────┘
                                  │
                           ┌──────▼──────┐
                           │ PostgreSQL  │
                           │   :5432     │
                           └─────────────┘
```

All browser traffic goes through Nuxt server routes (BFF pattern) — the frontend never talks directly to the Rust backend. Only nginx exposes ports to the host.

## Documentation

| Section | Description |
|---------|-------------|
| **[Self-Hosting Guide](docs/self-hosting/)** | Deployment, Docker, nginx, SSL, backups, upgrades |
| **[API Reference](docs/api/)** | GraphQL endpoint, authentication, error codes, rate limiting |
| **[User Guide](docs/user-guide/)** | Boards, posting, flairs, streams, wiki, moderation |
| **[Contributing](docs/contributing/)** | Local setup, project structure, code style, PR process |

Additional references:

- [`SELF_HOSTING.md`](SELF_HOSTING.md) — Step-by-step deployment guide
- [`deploy/.env.example`](deploy/.env.example) — All environment variables documented
- [`tinyboards.example.hjson`](tinyboards.example.hjson) — Configuration template

## Contributing

tinyboards is actively looking for people to build it with — whether that means writing Rust resolvers, building Vue components, improving the UX, writing documentation, or filing detailed bug reports. There's no inner circle and no gatekeeping.

**Get involved:**

- **[Discord](https://discord.gg/7Q99vp5DHa)** — Introduce yourself, ask questions, share ideas
- **[Contributing Guide](docs/contributing/)** — Local setup, code style, and PR process
- **Issues** — Browse open issues or look for `good first issue` labels
- **[Local Setup](docs/contributing/local-setup.md)** — Get the dev environment running

## License

tinyboards is licensed under the [GNU Affero General Public License v3.0](LICENSE).
If you run a modified version of tinyboards publicly, you must share your modifications under the same license.
