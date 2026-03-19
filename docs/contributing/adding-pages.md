# Adding a Frontend Page

This guide walks through adding a new page to the Nuxt 3 frontend.

## Table of Contents

- [How Routing Works](#how-routing-works)
- [Example: Adding a Leaderboard Page](#example-adding-a-leaderboard-page)
- [Dynamic Routes](#dynamic-routes)
- [Layouts](#layouts)
- [Fetching Data](#fetching-data)

## How Routing Works

Nuxt 3 uses file-based routing. Files in the `frontend/pages/` directory automatically become routes:

| File | Route |
|------|-------|
| `pages/index.vue` | `/` |
| `pages/boards.vue` | `/boards` |
| `pages/b/[board]/index.vue` | `/b/:board` |
| `pages/b/[board]/feed/[id]/[[slug]].vue` | `/b/:board/feed/:id/:slug?` |
| `pages/@[username]/index.vue` | `/@:username` |
| `pages/settings/account.vue` | `/settings/account` |

- `[param]` — Required dynamic segment
- `[[param]]` — Optional dynamic segment
- `index.vue` — Default page for a directory

## Example: Adding a Leaderboard Page

Let's add a `/leaderboard` page that shows top users by reputation.

### 1. Create the Page File

Create `frontend/pages/leaderboard.vue`:

```vue
<template>
  <div class="max-w-4xl mx-auto p-4">
    <h1 class="text-2xl font-bold mb-4">Leaderboard</h1>

    <div v-if="pending" class="text-center py-8">
      Loading...
    </div>

    <div v-else-if="error" class="text-red-500">
      Failed to load leaderboard.
    </div>

    <table v-else class="w-full">
      <thead>
        <tr class="border-b">
          <th class="text-left py-2">Rank</th>
          <th class="text-left py-2">User</th>
          <th class="text-right py-2">Posts</th>
          <th class="text-right py-2">Comments</th>
          <th class="text-right py-2">Score</th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="(user, index) in users"
          :key="user.id"
          class="border-b hover:bg-gray-50"
        >
          <td class="py-2">{{ index + 1 }}</td>
          <td class="py-2">
            <NuxtLink :to="`/@${user.name}`" class="text-blue-600 hover:underline">
              {{ user.displayName || user.name }}
            </NuxtLink>
          </td>
          <td class="py-2 text-right">{{ user.postCount }}</td>
          <td class="py-2 text-right">{{ user.commentCount }}</td>
          <td class="py-2 text-right">{{ user.postScore + user.commentScore }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script setup>
const { graphql } = useGraphQL();

const { data: users, pending, error } = await useAsyncData('leaderboard', async () => {
  const result = await graphql(`
    query {
      listUsers(sort: "top", limit: 50) {
        id
        name
        displayName
        aggregates {
          postCount
          commentCount
          postScore
          commentScore
        }
      }
    }
  `);
  return result.listUsers.map(u => ({
    ...u,
    postCount: u.aggregates.postCount,
    commentCount: u.aggregates.commentCount,
    postScore: u.aggregates.postScore,
    commentScore: u.aggregates.commentScore,
  }));
});
</script>
```

### 2. Add Navigation Link (Optional)

To add the page to the navbar or a menu, edit the relevant navigation component. For example, in `frontend/components/nav/Navbar.vue`:

```vue
<NuxtLink to="/leaderboard">Leaderboard</NuxtLink>
```

### 3. Verify

Start the dev server and navigate to `http://localhost:3000/leaderboard`.

## Dynamic Routes

### Required Parameter

For a route like `/b/:board/members`:

Create `frontend/pages/b/[board]/members.vue`:

```vue
<script setup>
const route = useRoute();
const boardName = route.params.board;
</script>
```

### Optional Parameter

For a route like `/home/:sort?`:

Create `frontend/pages/home/[[sort]].vue`:

```vue
<script setup>
const route = useRoute();
const sort = route.params.sort || 'hot'; // default to 'hot'
</script>
```

## Layouts

Pages use the `default` layout unless specified otherwise. To use a different layout:

```vue
<script setup>
definePageMeta({
  layout: 'admin',
});
</script>
```

Available layouts:
- `default` — Standard site layout with navbar and footer
- `admin` — Admin panel layout
- `settings` — User settings layout
- `inbox` — Message inbox layout
- `wizard` — Board creation wizard
- `registration` — Login/register pages
- `error` — Error pages

## Fetching Data

### Using the GraphQL Composable

The primary way to fetch data is through `useGraphQL`:

```vue
<script setup>
const { graphql } = useGraphQL();

// With useAsyncData for SSR support
const { data, pending, error } = await useAsyncData('key', () =>
  graphql(`
    query GetBoard($name: String!) {
      board(name: $name) {
        id
        name
        title
        description
      }
    }
  `, { name: 'general' })
);
</script>
```

### Key Points

- Use `useAsyncData` for data that should be fetched during SSR.
- The first argument is a unique cache key.
- GraphQL variables are passed as the second argument to `graphql()`.
- The composable handles authentication (cookies), retries, and error formatting.

### Reactive Data

For data that changes based on route params:

```vue
<script setup>
const route = useRoute();

const { data: board } = await useAsyncData(
  () => `board-${route.params.board}`,
  () => graphql(`
    query GetBoard($name: String!) {
      board(name: $name) { id name title }
    }
  `, { name: route.params.board }),
  { watch: [() => route.params.board] }
);
</script>
```

### Mutations

For mutations (creating, updating, deleting data):

```vue
<script setup>
const { graphql } = useGraphQL();

async function submitPost(title, body, boardId) {
  const result = await graphql(`
    mutation SubmitPost($input: SubmitPostInput!) {
      submitPost(input: $input) { id slug }
    }
  `, {
    input: { title, body, boardId, postType: 'text' }
  });

  navigateTo(`/b/${boardName}/feed/${result.submitPost.id}/${result.submitPost.slug}`);
}
</script>
```
