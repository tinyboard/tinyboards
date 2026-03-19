# Contributing to TinyBoards

Thank you for your interest in contributing to TinyBoards. This guide covers everything you need to get started.

## Table of Contents

- [Getting Started](#getting-started)
- [Guides](#guides)
- [Where to Contribute](#where-to-contribute)
- [Getting Help](#getting-help)

## Getting Started

1. **Set up your local environment** — Follow [local-setup.md](local-setup.md) to get PostgreSQL, the backend, and the frontend running locally.
2. **Understand the project structure** — Read [project-structure.md](project-structure.md) for a map of the codebase.
3. **Read the code style guide** — Follow [code-style.md](code-style.md) for conventions and linting rules.
4. **Learn the PR process** — See [contributing.md](contributing.md) for commit messages, PR workflow, and review guidelines.

## Guides

| Guide | Description |
|-------|-------------|
| [Local Setup](local-setup.md) | Step-by-step: PostgreSQL, backend, frontend |
| [Project Structure](project-structure.md) | Crate layout and Nuxt 3 directory structure |
| [Adding a GraphQL Query/Mutation](adding-graphql.md) | Concrete example: add a query, add a mutation |
| [Adding a Frontend Page](adding-pages.md) | Concrete example: add a Nuxt 3 route |
| [Testing](testing.md) | cargo nextest, vitest, playwright |
| [Code Style](code-style.md) | Rust clippy rules, Vue/TS conventions |
| [PR Process](contributing.md) | Commit format, review expectations |

## Where to Contribute

### Good First Issues

Look for issues labeled `good-first-issue` or `help-wanted`. These are scoped tasks with clear requirements.

### Feature Areas

| Area | Backend | Frontend | Description |
|------|---------|----------|-------------|
| Core | `backend/crates/api/src/graphql/` | `frontend/pages/` | Posts, comments, boards, users |
| Auth | `backend/crates/api/src/graphql/mutations/auth.rs` | `frontend/composables/api.ts` | Authentication and sessions |
| Moderation | `backend/crates/api/src/graphql/mutations/moderation_unified.rs` | `frontend/pages/b/[board]/mod/` | Board and site moderation tools |
| Flairs | `backend/crates/api/src/graphql/{queries,mutations}/flair*` | `frontend/components/flair/` | Flair system |
| Streams | `backend/crates/api/src/graphql/{queries,mutations}/stream/` | `frontend/pages/streams/` | Custom feed streams |
| Wiki | `backend/crates/api/src/graphql/{queries,mutations}/wiki/` | `frontend/pages/b/[board]/wiki/` | Board wikis |

## Getting Help

- Open an issue on GitHub for bugs or feature requests.
- Check existing issues before filing a new one.
- For questions about the codebase, reference the [project structure](project-structure.md) guide.
