# Code Style Guide

Conventions and style rules for contributing to TinyBoards.

## Table of Contents

- [Rust (Backend)](#rust-backend)
- [Vue / TypeScript (Frontend)](#vue--typescript-frontend)
- [SQL](#sql)
- [General](#general)

## Rust (Backend)

### Error Handling

- Use `thiserror` for all custom error types. `anyhow` is only used in `main.rs`.
- All GraphQL resolvers return `Result<T, async_graphql::Error>`.
- No `unwrap()` or `expect()` in non-test code. Use `?` for propagation or explicit error handling.

```rust
// Good
let user = users::table
    .find(user_id)
    .first::<User>(&mut conn)
    .await
    .map_err(|_| async_graphql::Error::new("User not found"))?;

// Bad
let user = users::table
    .find(user_id)
    .first::<User>(&mut conn)
    .await
    .unwrap(); // never do this
```

### Logging

- Use `tracing` for all logging. No `println!` or `eprintln!` in production code.

```rust
use tracing::{info, warn, error, debug};

info!("User {} logged in", user.name);
warn!("Rate limit approaching for IP {}", ip);
error!("Database connection failed: {}", err);
```

### Module Pattern

Feature modules follow this structure:

```
queries/
├── posts.rs       # Query resolvers
mutations/
├── post/
│   ├── submit_post.rs   # Create post mutation
│   ├── edit.rs           # Edit post mutation
│   ├── actions.rs        # Vote, save, delete, etc.
│   └── moderation.rs     # Remove, approve, lock
```

### Naming

- Types: `PascalCase` — `BoardStats`, `CreatePostInput`
- Functions: `snake_case` — `get_board_stats`, `create_post`
- Constants: `SCREAMING_SNAKE_CASE` — `MAX_TITLE_LENGTH`
- GraphQL fields: `camelCase` (async-graphql converts automatically from Rust's `snake_case`)

### Clippy

All code must pass `cargo clippy` with no warnings:

```bash
cargo clippy --all-targets -- -D warnings
```

Key clippy lints enforced:
- `clippy::unwrap_used` — Forbidden outside tests
- `clippy::expect_used` — Forbidden outside tests
- `clippy::needless_pass_by_value` — Prefer references
- `clippy::redundant_clone` — Don't clone unnecessarily

### Formatting

Use `rustfmt` with default settings:

```bash
cargo fmt
```

## Vue / TypeScript (Frontend)

### Components

- **PascalCase** for component names: `PostCard.vue`, `FlairBadge.vue`
- **Single-file components** (`.vue`) with `<script setup>` syntax
- Order within `.vue` files: `<template>`, then `<script setup>`, then `<style>`

```vue
<template>
  <div class="post-card">
    <h2>{{ post.title }}</h2>
  </div>
</template>

<script setup>
const props = defineProps({
  post: {
    type: Object,
    required: true,
  },
});
</script>

<style scoped>
.post-card {
  /* styles */
}
</style>
```

### Composables

- **camelCase** with `use` prefix: `useGraphQL`, `useStream`, `useFlairStyle`
- Return an object with named exports:

```typescript
export function useCounter() {
  const count = ref(0);
  const increment = () => count.value++;
  return { count, increment };
}
```

### Stores (Pinia)

- **PascalCase** file names with `Store` prefix: `StoreAuth.ts`, `StorePosts.ts`
- Use the setup store syntax:

```typescript
export const useAuthStore = defineStore('auth', () => {
  const user = ref(null);
  const isLoggedIn = computed(() => user.value !== null);
  return { user, isLoggedIn };
});
```

### TypeScript

- Prefer `interface` for object shapes, `type` for unions and intersections.
- All shared API types belong in `frontend/lib/types.ts`.
- No `any` — use `unknown` and narrow with type guards.

### CSS

- Use Tailwind utility classes as the primary styling approach.
- Scoped `<style>` blocks for component-specific styles.
- No global styles except in `assets/css/`.

### File Naming

| Type | Convention | Example |
|------|-----------|---------|
| Pages | `kebab-case` or Nuxt conventions | `login.vue`, `[board]/index.vue` |
| Components | `PascalCase` | `PostCard.vue`, `FlairBadge.vue` |
| Composables | `camelCase` with `use` prefix | `useGraphQL.ts` |
| Stores | `PascalCase` with `Store` prefix | `StoreAuth.ts` |
| Utilities | `camelCase` | `formatDate.ts` |

## SQL

### Table and Column Names

- `snake_case` for all table and column names
- UUID primary keys via `gen_random_uuid()`
- Every table has `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`
- Mutable tables have `updated_at TIMESTAMPTZ NOT NULL DEFAULT now()` with a trigger
- Soft deletes use `deleted_at TIMESTAMPTZ` (nullable)

### Migrations

- Named sequentially: `000001_foundation`, `000002_core_tables`, etc.
- Each migration has `up.sql` and `down.sql`
- `down.sql` must cleanly reverse `up.sql`
- Always include `IF NOT EXISTS` / `IF EXISTS` guards where appropriate

```sql
-- Good: up.sql
CREATE TABLE IF NOT EXISTS example (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

SELECT add_updated_at_trigger('example');

-- Good: down.sql
DROP TABLE IF EXISTS example;
```

### Indexes

- Name format: `idx_{table}_{column(s)}`
- Add indexes for foreign keys, frequently filtered columns, and sort columns

```sql
CREATE INDEX idx_posts_board ON posts (board_id);
CREATE INDEX idx_posts_board_created ON posts (board_id, created_at);
```

## General

### Comments

- Write comments that explain **why**, not **what**.
- Code should be self-documenting through clear naming.
- Don't add comments to code you didn't change.

```rust
// Good: explains why
// Hot rank decays over time to push older posts down the feed
let hours_diff = (now - published).num_hours() as f64;

// Bad: restates the code
// Calculate hours difference
let hours_diff = (now - published).num_hours() as f64;
```

### Commit Messages

Use conventional commit format:

```
feat(auth): implement JWT refresh token rotation
fix(posts): correct pagination offset calculation
refactor(db): replace raw SQL with Diesel query builder
test(comments): add tests for nested comment threading
docs(api): document rate limiting behavior
```

See [contributing.md](contributing.md) for the full PR process.
