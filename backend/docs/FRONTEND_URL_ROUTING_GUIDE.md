# Frontend URL Routing Implementation Guide

## Overview

The backend now supports hierarchical URL routing with SEO-friendly slugs. This guide details what needs to be implemented on the frontend.

## Backend Status: ✅ COMPLETE

The following backend features are ready:

- ✅ Database migrations with `slug` columns
- ✅ Automatic slug generation on content creation
- ✅ GraphQL `slug` and `urlPath` fields on Post and Comment types
- ✅ URL builder service respecting `boardsEnabled` setting
- ✅ Proper camelCase conversion in GraphQL

## URL Structure

### With boards enabled (`boardsEnabled: true`):
```
/b/:boardSlug/threads/:id/:slug       # Thread posts
/b/:boardSlug/feed/:id/:slug          # Feed posts
/b/:boardSlug/wiki/:pageSlug          # Wiki pages
/b/:boardSlug/threads                 # Section listing
/b/:boardSlug                         # Board homepage
```

### Without boards (`boardsEnabled: false`):
```
/threads/:id/:slug                    # Thread posts
/feed/:id/:slug                       # Feed posts
/wiki/:pageSlug                       # Wiki pages
/threads                              # Section listing
/                                     # Homepage
```

## Frontend Implementation Tasks

### Agent 5: Vue Router Configuration

#### 5.1 Update router/index.js (or equivalent)

```javascript
// Fetch boards_enabled setting first
const boardsEnabled = await fetchBoardsEnabled(); // See query below

const routes = [
  // Board-enabled routes
  ...(boardsEnabled ? [
    {
      path: '/b/:boardSlug',
      component: BoardLayout,
      children: [
        {
          path: '',
          name: 'board-home',
          component: BoardHome,
        },
        {
          path: 'threads',
          name: 'board-threads',
          component: ThreadsList,
        },
        {
          path: 'threads/:id/:slug?',
          name: 'board-thread-view',
          component: ThreadView,
        },
        {
          path: 'feed',
          name: 'board-feed',
          component: FeedList,
        },
        {
          path: 'feed/:id/:slug?',
          name: 'board-feed-view',
          component: FeedPostView,
        },
        {
          path: 'wiki/:slug',
          name: 'board-wiki-page',
          component: WikiPage,
        },
      ],
    },
  ] : []),

  // Single-board mode routes
  ...(!boardsEnabled ? [
    {
      path: '/threads',
      name: 'threads',
      component: ThreadsList,
    },
    {
      path: '/threads/:id/:slug?',
      name: 'thread-view',
      component: ThreadView,
    },
    {
      path: '/feed',
      name: 'feed',
      component: FeedList,
    },
    {
      path: '/feed/:id/:slug?',
      name: 'feed-view',
      component: FeedPostView,
    },
    {
      path: '/wiki/:slug',
      name: 'wiki-page',
      component: WikiPage,
    },
  ] : []),
];

const router = createRouter({
  history: createWebHistory(),
  routes,
  scrollBehavior(to, from, savedPosition) {
    if (savedPosition) {
      return savedPosition;
    } else if (to.hash) {
      return { el: to.hash };
    } else {
      return { top: 0 };
    }
  },
});
```

#### 5.2 Create URL Builder Composable

Create `composables/useUrlBuilder.js`:

```javascript
import { computed } from 'vue';
import { useConfigStore } from '@/stores/config';

export function useUrlBuilder() {
  const config = useConfigStore();
  const boardsEnabled = computed(() => config.boardsEnabled);

  function buildThreadUrl(threadId, slug, boardSlug) {
    if (boardsEnabled.value && boardSlug) {
      return `/b/${boardSlug}/threads/${threadId}/${slug}`;
    }
    return `/threads/${threadId}/${slug}`;
  }

  function buildFeedUrl(postId, slug, boardSlug) {
    if (boardsEnabled.value && boardSlug) {
      return `/b/${boardSlug}/feed/${postId}/${slug}`;
    }
    return `/feed/${postId}/${slug}`;
  }

  function buildWikiUrl(pageSlug, boardSlug) {
    if (boardsEnabled.value && boardSlug) {
      return `/b/${boardSlug}/wiki/${pageSlug}`;
    }
    return `/wiki/${pageSlug}`;
  }

  function buildSectionUrl(section, boardSlug) {
    if (boardsEnabled.value && boardSlug) {
      return `/b/${boardSlug}/${section}`;
    }
    return `/${section}`;
  }

  function buildBoardUrl(boardSlug) {
    return `/b/${boardSlug}`;
  }

  return {
    buildThreadUrl,
    buildFeedUrl,
    buildWikiUrl,
    buildSectionUrl,
    buildBoardUrl,
    boardsEnabled,
  };
}
```

#### 5.3 Create Config Store

Create `stores/config.js` (Pinia example):

```javascript
import { defineStore } from 'pinia';
import { useQuery } from '@vue/apollo-composable';
import gql from 'graphql-tag';

export const useConfigStore = defineStore('config', {
  state: () => ({
    boardsEnabled: true, // Default value
    configLoaded: false,
  }),

  actions: {
    async loadConfig() {
      const { result } = useQuery(gql`
        query GetSiteConfig {
          site {
            boardsEnabled
          }
        }
      `);

      if (result.value?.site) {
        this.boardsEnabled = result.value.site.boardsEnabled;
        this.configLoaded = true;
      }
    },
  },
});
```

Call `loadConfig()` in your app initialization (e.g., `main.js` or `App.vue`).

### Agent 6: GraphQL Query Updates

#### 6.1 Update All Post Queries

**CRITICAL**: Use camelCase for all field names!

```graphql
query GetPost($id: Int!) {
  post(id: $id) {
    id
    title
    slug           # ← Added
    urlPath        # ← Added (computed field)
    body
    bodyHTML
    creationDate   # ← camelCase (from creation_date)
    boardId        # ← camelCase (from board_id)
    postType       # ← camelCase (from post_type)

    board {
      id
      name         # This is the board slug
    }
  }
}
```

#### 6.2 Update All Comment Queries

```graphql
query GetComment($id: Int!) {
  comment(id: $id) {
    id
    body
    bodyHTML
    slug           # ← Added
    creationDate   # ← camelCase
    boardId        # ← camelCase
    postId         # ← camelCase
  }
}
```

#### 6.3 Field Naming Rules

**Database → GraphQL Conversion:**
- `creation_date` → `creationDate`
- `board_id` → `boardId`
- `user_id` → `userId`
- `post_id` → `postId`
- `body_html` → `bodyHTML`
- `is_nsfw` → `isNSFW`
- `post_type` → `postType`

**❌ NEVER use snake_case in frontend:**
```javascript
// ❌ WRONG
const date = post.creation_date;
const boardId = post.board_id;

// ✅ CORRECT
const date = post.creationDate;
const boardId = post.boardId;
```

### Agent 7: Component Updates

#### 7.1 Update Link Generation

Replace all hardcoded URLs with the URL builder:

```vue
<template>
  <!-- ❌ OLD: Hardcoded -->
  <router-link :to="`/post/${post.id}`">
    {{ post.title }}
  </router-link>

  <!-- ✅ NEW: Using URL builder -->
  <router-link :to="buildThreadUrl(post.id, post.slug, board.name)">
    {{ post.title }}
  </router-link>

  <!-- OR use the urlPath field directly -->
  <router-link :to="post.urlPath">
    {{ post.title }}
  </router-link>
</template>

<script setup>
import { useUrlBuilder } from '@/composables/useUrlBuilder';

const { buildThreadUrl } = useUrlBuilder();
</script>
```

#### 7.2 Create Breadcrumbs Component

Create `components/Breadcrumbs.vue`:

```vue
<template>
  <nav aria-label="Breadcrumb">
    <ol class="breadcrumb">
      <li><router-link to="/">Home</router-link></li>

      <li v-if="boardSlug && boardsEnabled">
        <router-link :to="buildBoardUrl(boardSlug)">
          {{ boardName }}
        </router-link>
      </li>

      <li v-if="section">
        <router-link :to="buildSectionUrl(section, boardSlug)">
          {{ section }}
        </router-link>
      </li>

      <li v-if="currentPage" aria-current="page">
        {{ currentPage }}
      </li>
    </ol>
  </nav>
</template>

<script setup>
import { computed } from 'vue';
import { useRoute } from 'vue-router';
import { useUrlBuilder } from '@/composables/useUrlBuilder';

const route = useRoute();
const { buildBoardUrl, buildSectionUrl, boardsEnabled } = useUrlBuilder();

const boardSlug = computed(() => route.params.boardSlug);
const section = computed(() => {
  // Extract section from path
  const path = route.path;
  if (path.includes('/threads')) return 'threads';
  if (path.includes('/feed')) return 'feed';
  if (path.includes('/wiki')) return 'wiki';
  return null;
});
</script>
```

#### 7.3 Update Detail Views

In `ThreadView.vue`, `FeedPostView.vue`, etc.:

```vue
<script setup>
import { useRoute, useRouter } from 'vue-router';
import { useQuery } from '@vue/apollo-composable';
import { watch } from 'vue';

const route = useRoute();
const router = useRouter();

const { id, slug } = route.params;

const { result } = useQuery(GET_POST_QUERY, { id: parseInt(id) });

// Redirect to canonical URL if slug is missing or incorrect
watch(result, (newResult) => {
  if (newResult?.post) {
    const post = newResult.post;

    // Use the urlPath field from GraphQL for canonical URL
    const canonicalUrl = post.urlPath;

    if (route.path !== canonicalUrl) {
      router.replace(canonicalUrl);
    }
  }
});
</script>
```

#### 7.4 Update Form Submissions

After creating a post/comment:

```javascript
async function submitPost(formData) {
  const { data } = await createPostMutation({
    variables: { ...formData },
  });

  if (data?.createPost) {
    const post = data.createPost;

    // Navigate using the urlPath field
    router.push(post.urlPath);

    // OR build manually:
    // const url = buildThreadUrl(post.id, post.slug, board.name);
    // router.push(url);
  }
}
```

### Field Access Checklist

Before deploying, verify:

- [ ] All GraphQL queries use camelCase field names
- [ ] No `snake_case` field access in `.vue` or `.js` files
- [ ] `urlPath` field is used or URL builder is used
- [ ] Breadcrumbs component is integrated
- [ ] Detail views redirect to canonical URLs
- [ ] Forms navigate to correct URLs after submission
- [ ] TypeScript types regenerated (if using codegen)

### Testing Checklist

- [ ] Toggle `boardsEnabled` setting and verify routes change
- [ ] Thread URLs work in both board modes
- [ ] Feed URLs work in both board modes
- [ ] Wiki URLs work in both board modes
- [ ] Breadcrumbs display correctly
- [ ] Missing slugs redirect properly
- [ ] URL sharing works (paste URL in new tab)
- [ ] Mobile navigation works
- [ ] Special characters in titles handled correctly
- [ ] Duplicate titles get unique slugs

## GraphQL Schema Reference

### New Fields Available:

**Post type:**
```graphql
type Post {
  # ... existing fields
  slug: String!           # SEO-friendly URL slug
  urlPath: String!        # Computed canonical URL path
}
```

**Comment type:**
```graphql
type Comment {
  # ... existing fields
  slug: String!           # URL slug for comment
}
```

**Site type:**
```graphql
type Site {
  # ... existing fields
  boardsEnabled: Boolean! # Whether multi-board mode is enabled
}
```

## Migration Path

1. **Phase 1**: Backend deployed (already done)
2. **Phase 2**: Frontend reads new fields
3. **Phase 3**: Frontend uses URL builder for new content
4. **Phase 4**: Update all existing links to use URL builder
5. **Phase 5**: Add breadcrumbs and canonical URL redirects

## Support

For questions or issues, refer to:
- Backend code: `crates/api/src/utils/url_builder.rs`
- Slug generation: `crates/utils/src/slug.rs`
- GraphQL types: `crates/api/src/structs/post.rs` and `comment.rs`

## Schema Conventions Reference

For future reference when working with the database:

**Timestamp Fields:**
- Created: `creation_date` (NOT `created_at`)
- Updated: `updated` (NOT `updated_at`)

**Naming Patterns:**
- Database: snake_case (`board_id`, `post_id`)
- GraphQL: camelCase (`boardId`, `postId`)
- Conversion is automatic via async-graphql

**Table Names:**
- Plural: `posts`, `comments`, `boards`
- Aggregates: `post_aggregates`, `board_aggregates`

**Indexes:**
- Format: `idx_{table}_{column(s)}`
- Example: `idx_posts_board_slug`
