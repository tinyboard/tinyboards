# Pull Request Process

How to submit changes to TinyBoards.

## Table of Contents

- [Before You Start](#before-you-start)
- [Branch Naming](#branch-naming)
- [Commit Messages](#commit-messages)
- [Pull Request Guidelines](#pull-request-guidelines)
- [Review Process](#review-process)
- [After Merge](#after-merge)

## Before You Start

1. Check existing issues and PRs to avoid duplicating work.
2. For large changes, open an issue first to discuss the approach.
3. Set up your local environment per [local-setup.md](local-setup.md).

## Branch Naming

Create a feature branch from `main`:

```bash
git checkout main
git pull origin main
git checkout -b feat/add-leaderboard
```

Branch name conventions:

| Prefix | Purpose | Example |
|--------|---------|---------|
| `feat/` | New features | `feat/add-leaderboard` |
| `fix/` | Bug fixes | `fix/pagination-offset` |
| `refactor/` | Code improvements | `refactor/auth-middleware` |
| `docs/` | Documentation | `docs/api-rate-limiting` |
| `test/` | Test additions | `test/comment-threading` |

## Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/) format:

```
type(scope): description
```

### Types

| Type | When to Use |
|------|-------------|
| `feat` | New feature |
| `fix` | Bug fix |
| `refactor` | Code change that neither fixes a bug nor adds a feature |
| `test` | Adding or updating tests |
| `docs` | Documentation changes |
| `style` | Formatting, semicolons, etc. (no code change) |
| `perf` | Performance improvement |
| `chore` | Build, CI, dependency updates |

### Scopes

Use the module or area affected:

- `auth`, `posts`, `comments`, `boards`, `users`
- `flairs`, `streams`, `wiki`, `moderation`
- `db`, `api`, `graphql`
- `frontend`, `components`, `stores`
- `docker`, `nginx`, `ci`

### Examples

```
feat(streams): add stream sharing via share tokens
fix(comments): handle deleted parent in nested thread view
refactor(db): replace raw SQL in scheduled tasks with Diesel queries
test(auth): add integration tests for refresh token rotation
docs(api): document GraphQL error codes
```

### Rules

- Keep the first line under 72 characters.
- Use imperative mood ("add feature" not "added feature").
- Reference related issues: `fix(posts): correct sort order (closes #42)`.
- No co-author lines or attribution.

## Pull Request Guidelines

### PR Title

Follow the same format as commit messages:

```
feat(streams): add stream sharing via share tokens
```

### PR Description

Include:

1. **What** — Brief description of the change.
2. **Why** — Motivation or issue being solved.
3. **How** — High-level approach (if not obvious from the diff).
4. **Testing** — How you tested the change.

Template:

```markdown
## What

Added share token generation and redemption for streams.

## Why

Users want to share private streams with specific people without
making them public.

## How

- Added `share_token` column to `streams` table
- Added `regenerateShareToken` mutation
- Added `/s/[token]` route in the frontend

## Testing

- Added backend tests for token generation and validation
- Tested manually: create stream → generate token → open in incognito
```

### Checklist

Before submitting:

- [ ] Code compiles without warnings (`cargo build`, `pnpm build`)
- [ ] Linting passes (`cargo clippy -- -D warnings`, `pnpm lint`)
- [ ] Formatting is correct (`cargo fmt --check`)
- [ ] Tests pass (`cargo nextest run`, `pnpm test`)
- [ ] New code has tests where appropriate
- [ ] No `unwrap()` in production Rust code
- [ ] No `any` in TypeScript code
- [ ] GraphQL schema changes are backwards-compatible (or noted as breaking)

## Review Process

1. Open a PR against `main`.
2. Automated CI checks run (linting, tests, build).
3. A maintainer reviews the code.
4. Address review feedback with new commits (don't force-push during review).
5. Once approved, the maintainer merges.

### What Reviewers Look For

- Correctness and completeness
- Test coverage for new behavior
- No security vulnerabilities
- Consistent code style
- Clear error handling
- No unnecessary complexity

## After Merge

- Delete your feature branch.
- If the change requires documentation updates, file a follow-up issue or include docs in the same PR.
- If the change affects deployment, note it in the PR so release notes reflect it.
