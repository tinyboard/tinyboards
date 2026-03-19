# Testing

TinyBoards uses different test frameworks for the backend and frontend.

## Table of Contents

- [Backend Tests (Rust)](#backend-tests-rust)
- [Frontend Tests](#frontend-tests)
- [Integration Tests](#integration-tests)

## Backend Tests (Rust)

### Running Tests

```bash
cd backend

# Standard cargo test
cargo test

# With cargo-nextest (recommended — better output, parallelism)
cargo install cargo-nextest
cargo nextest run
```

### Running Specific Tests

```bash
# Run tests in a specific crate
cargo test -p tinyboards-api

# Run tests matching a name pattern
cargo test auth

# Run a specific test function
cargo test test_login_valid_credentials

# With nextest
cargo nextest run -E 'test(auth)'
```

### Test Structure

Tests are co-located with the code they test:

```rust
// At the bottom of a source file
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_post_success() {
        let pool = create_test_pool().await;
        // ... test logic
    }

    #[tokio::test]
    async fn test_create_post_unauthorized() {
        let pool = create_test_pool().await;
        // ... test logic
    }
}
```

### Test Database

Tests use a separate test database. Set `DATABASE_URL` for tests in `.env.test` or as an environment variable:

```bash
DATABASE_URL=postgresql://tinyboards:tinyboards_dev@localhost:5432/tinyboards_test \
  cargo test
```

Create the test database:

```bash
sudo -u postgres psql -c "CREATE DATABASE tinyboards_test OWNER tinyboards;"
sudo -u postgres psql -d tinyboards_test -c "CREATE EXTENSION IF NOT EXISTS pgcrypto;"
sudo -u postgres psql -d tinyboards_test -c "CREATE EXTENSION IF NOT EXISTS pg_trgm;"
```

### Writing Good Tests

```rust
#[tokio::test]
async fn test_register_creates_user() {
    // Arrange
    let pool = create_test_pool().await;
    let schema = build_test_schema(pool.clone());

    // Act
    let result = schema.execute(r#"
        mutation {
            register(input: {
                username: "testuser",
                password: "testpassword123",
                passwordVerify: "testpassword123",
                email: "test@example.com"
            }) {
                user { id name }
            }
        }
    "#).await;

    // Assert
    assert!(result.errors.is_empty(), "Expected no errors: {:?}", result.errors);
    let data = result.data.into_json().unwrap();
    assert_eq!(data["register"]["user"]["name"], "testuser");
}
```

### Clippy and Formatting

Run these before submitting a PR:

```bash
# Linting
cargo clippy --all-targets -- -D warnings

# Formatting
cargo fmt --check
```

## Frontend Tests

### Unit Tests with Vitest

```bash
cd frontend

# Run all tests
pnpm test

# Run in watch mode
pnpm test:watch

# Run with coverage
pnpm test:coverage
```

### Writing Frontend Tests

Create test files alongside components with a `.test.ts` or `.spec.ts` suffix:

```
components/
├── cards/
│   ├── Post.vue
│   └── Post.test.ts
```

Example test:

```typescript
import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import Post from './Post.vue';

describe('Post component', () => {
  it('renders the post title', () => {
    const wrapper = mount(Post, {
      props: {
        post: {
          id: '123',
          title: 'Test Post',
          body: 'Test body',
          creator: { name: 'testuser' },
        },
      },
    });

    expect(wrapper.text()).toContain('Test Post');
  });

  it('shows NSFW tag when post is NSFW', () => {
    const wrapper = mount(Post, {
      props: {
        post: {
          id: '123',
          title: 'Test Post',
          isNsfw: true,
          creator: { name: 'testuser' },
        },
      },
    });

    expect(wrapper.text()).toContain('NSFW');
  });
});
```

### End-to-End Tests with Playwright

```bash
cd frontend

# Install Playwright browsers (first time)
npx playwright install

# Run E2E tests
pnpm test:e2e

# Run with UI mode
pnpm test:e2e --ui

# Run a specific test file
pnpm test:e2e tests/auth.spec.ts
```

Example E2E test:

```typescript
import { test, expect } from '@playwright/test';

test('user can log in', async ({ page }) => {
  await page.goto('/login');
  await page.fill('input[name="username"]', 'testuser');
  await page.fill('input[name="password"]', 'testpassword');
  await page.click('button[type="submit"]');

  await expect(page).toHaveURL('/home');
  await expect(page.locator('.navbar-username')).toContainText('testuser');
});

test('user can create a post', async ({ page }) => {
  // Login first
  await page.goto('/login');
  await page.fill('input[name="username"]', 'testuser');
  await page.fill('input[name="password"]', 'testpassword');
  await page.click('button[type="submit"]');

  // Create post
  await page.goto('/submit');
  await page.fill('input[name="title"]', 'My Test Post');
  await page.fill('.tiptap-editor', 'This is the post body.');
  await page.click('button:text("Submit")');

  await expect(page.locator('h1')).toContainText('My Test Post');
});
```

## Integration Tests

### Full-Stack Testing

For testing the complete flow (frontend → GraphQL → database), use Playwright E2E tests with a running backend.

### Test Environment Setup

```bash
# Start the backend with test database
DATABASE_URL=postgresql://tinyboards:tinyboards_dev@localhost:5432/tinyboards_test \
  cargo run

# In another terminal, start the frontend
cd frontend && pnpm dev

# In another terminal, run E2E tests
cd frontend && pnpm test:e2e
```

### CI Pipeline

Tests run in CI on every pull request. The pipeline:

1. Starts a PostgreSQL service container
2. Runs `cargo clippy` and `cargo fmt --check`
3. Runs `cargo nextest run`
4. Builds the frontend with `pnpm build`
5. Runs `pnpm test` (vitest)
6. Runs `pnpm test:e2e` (Playwright)
