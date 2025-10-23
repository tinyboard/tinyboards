# Flair System Documentation

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Database Schema](#database-schema)
4. [GraphQL API](#graphql-api)
5. [Permission System](#permission-system)
6. [Frontend Integration](#frontend-integration)
7. [Configuration & Limits](#configuration--limits)
8. [Best Practices](#best-practices)

---

## Overview

The TinyBoards flair system provides customizable labels for posts and users within boards. Flairs can include text, colors, gradients, custom emojis, and visual effects to help categorize content and identify users.

### Key Features

- **Post Flairs**: Categorize and tag posts within boards
- **User Flairs**: Allow users to personalize their identity on boards
- **Custom Styling**: Support for colors, gradients, borders, text shadows, and animations
- **Emoji Integration**: Add custom emojis to flairs for visual appeal
- **Permission Controls**: Moderator-only flairs and approval workflows
- **User Filtering**: Users can hide/highlight specific flairs in their feed
- **Board-Specific**: Each board manages its own flair templates

---

## Architecture

### Core Components

```
┌─────────────────────┐
│  Flair Templates    │  ← Created by moderators
│  (board-specific)   │
└──────────┬──────────┘
           │
           ├──────────────────┬──────────────────┐
           ▼                  ▼                  ▼
    ┌─────────────┐    ┌─────────────┐   ┌──────────────┐
    │ Post Flairs │    │ User Flairs │   │ Flair Filters│
    └─────────────┘    └─────────────┘   └──────────────┘
           │                  │                  │
           ▼                  ▼                  ▼
    ┌─────────────────────────────────────────────────┐
    │           Flair Aggregates (statistics)         │
    └─────────────────────────────────────────────────┘
```

### Database Tables

1. **flair_templates**: Template definitions for post/user flairs
2. **post_flairs**: Active flairs assigned to posts
3. **user_flairs**: Active flairs assigned to users
4. **flair_aggregates**: Usage statistics and metrics
5. **flair_filters**: User preferences for hiding/highlighting flairs

---

## Database Schema

### Flair Templates

```sql
CREATE TABLE flair_templates (
    id SERIAL PRIMARY KEY,
    board_id INTEGER REFERENCES boards(id),
    flair_type VARCHAR(10) NOT NULL CHECK (flair_type IN ('post', 'user')),

    -- Display Properties
    template_name VARCHAR(100) NOT NULL,
    template_key VARCHAR(50) UNIQUE,
    text_display VARCHAR(200),
    text_color VARCHAR(25) DEFAULT 'ffffff',
    background_color VARCHAR(25),

    -- Advanced Styling
    style_config JSONB DEFAULT '{}'::jsonb,  -- Gradients, shadows, animations
    emoji_ids INTEGER[] DEFAULT ARRAY[]::INTEGER[],

    -- Permissions & Behavior
    mod_only BOOLEAN DEFAULT false,
    is_editable BOOLEAN DEFAULT true,
    requires_approval BOOLEAN DEFAULT false,
    max_text_length INTEGER DEFAULT 64,

    -- Management
    display_order INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    usage_count INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    created_by INTEGER REFERENCES users(id)
);
```

**Key Fields:**

- `flair_type`: Either 'post' or 'user'
- `style_config`: JSONB containing gradients, text shadows, borders, animations
- `emoji_ids`: Array of custom emoji IDs to display
- `mod_only`: If true, only moderators can assign this flair
- `is_editable`: If true, users can customize text when applying
- `requires_approval`: If true, user flair assignments need moderator approval

### Post Flairs

```sql
CREATE TABLE post_flairs (
    id SERIAL PRIMARY KEY,
    post_id INTEGER REFERENCES posts(id) ON DELETE CASCADE,
    template_id INTEGER REFERENCES flair_templates(id),

    -- Custom Overrides
    custom_text VARCHAR(200),
    custom_background_color VARCHAR(25),
    custom_emoji_ids INTEGER[],

    -- Metadata
    assigned_by INTEGER REFERENCES users(id),
    assigned_at TIMESTAMP DEFAULT NOW()
);
```

### User Flairs

```sql
CREATE TABLE user_flairs (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id) ON DELETE CASCADE,
    board_id INTEGER REFERENCES boards(id) ON DELETE CASCADE,
    template_id INTEGER REFERENCES flair_templates(id),

    -- Custom Overrides
    custom_text VARCHAR(200),
    custom_background_color VARCHAR(25),
    custom_emoji_ids INTEGER[],

    -- Approval Workflow
    is_approved BOOLEAN DEFAULT true,
    approved_by INTEGER REFERENCES users(id),

    -- Metadata
    assigned_at TIMESTAMP DEFAULT NOW()
);
```

### Flair Filters

```sql
CREATE TABLE flair_filters (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id) ON DELETE CASCADE,
    board_id INTEGER REFERENCES boards(id) ON DELETE CASCADE,
    template_id INTEGER REFERENCES flair_templates(id),

    filter_type VARCHAR(10) CHECK (filter_type IN ('hide', 'show')),
    created_at TIMESTAMP DEFAULT NOW()
);
```

**Filter Types:**
- `hide`: Exclude posts with this flair from user's feed
- `show`: Only show posts with this flair (when any 'show' filters exist)

---

## GraphQL API

### Queries

#### Get Flair Templates

```graphql
query GetFlairTemplates($boardId: Int!, $flairType: String) {
  flairTemplates(boardId: $boardId, flairType: $flairType) {
    id
    templateName
    textDisplay
    textColor
    backgroundColor
    styleConfig
    emojiIds
    modOnly
    isEditable
    requiresApproval
    usageCount
    createdAt
  }
}
```

#### Get User's Flair Filters

```graphql
query GetFlairFilters($boardId: Int) {
  myFlairFilters(boardId: $boardId) {
    id
    boardId
    templateId
    filterType
    template {
      id
      templateName
      textDisplay
    }
  }
}
```

#### Get Pending Flair Approvals (Moderators)

```graphql
query GetPendingFlairApprovals($boardId: Int!, $limit: Int, $offset: Int) {
  pendingUserFlairs(boardId: $boardId, limit: $limit, offset: $offset) {
    id
    userId
    templateId
    customText
    assignedAt
    user {
      username
    }
    template {
      templateName
    }
  }
}
```

### Mutations

#### Create Flair Template (Moderators)

```graphql
mutation CreateFlairTemplate($input: CreateFlairTemplateInput!) {
  createFlairTemplate(input: $input) {
    id
    templateName
    textDisplay
    styleConfig
  }
}

# Input type
input CreateFlairTemplateInput {
  boardId: Int!
  flairType: String!  # "post" or "user"
  templateName: String!
  textDisplay: String
  textColor: String
  backgroundColor: String
  styleConfig: JSON  # Contains gradients, shadows, etc.
  emojiIds: [Int!]
  modOnly: Boolean
  isEditable: Boolean
  requiresApproval: Boolean
  maxTextLength: Int
  displayOrder: Int
}
```

**Style Config Structure:**

```json
{
  "gradient": {
    "type": "linear",
    "angle": 90,
    "stops": [
      { "color": "#667eea", "position": 0 },
      { "color": "#764ba2", "position": 100 }
    ]
  },
  "textShadow": {
    "offsetX": 1,
    "offsetY": 1,
    "blur": 2,
    "color": "rgba(0, 0, 0, 0.5)"
  },
  "borderColor": "#764ba2",
  "borderWidth": 2,
  "borderRadius": 4,
  "animation": "shimmer",
  "glow": true,
  "glowColor": "#667eea"
}
```

#### Update Flair Template (Moderators)

```graphql
mutation UpdateFlairTemplate($id: Int!, $input: UpdateFlairTemplateInput!) {
  updateFlairTemplate(id: $id, input: $input) {
    id
    templateName
    # ... fields
  }
}
```

#### Delete Flair Template (Moderators)

```graphql
mutation DeleteFlairTemplate($id: Int!) {
  deleteFlairTemplate(id: $id)
}
```

#### Assign Post Flair

```graphql
mutation AssignPostFlair(
  $postId: Int!
  $templateId: Int!
  $customText: String
  $customBackgroundColor: String
  $emojiIds: [Int!]
) {
  assignPostFlair(
    postId: $postId
    templateId: $templateId
    customText: $customText
    customBackgroundColor: $customBackgroundColor
    emojiIds: $emojiIds
  ) {
    id
    postId
    customText
    template {
      templateName
    }
  }
}
```

**Permissions:**
- Post author can assign any flair (unless mod_only)
- Moderators can assign any flair to any post

#### Remove Post Flair

```graphql
mutation RemovePostFlair($postId: Int!) {
  removePostFlair(postId: $postId)
}
```

#### Assign User Flair

```graphql
mutation AssignUserFlair(
  $boardId: Int!
  $templateId: Int!
  $customText: String
  $customBackgroundColor: String
  $emojiIds: [Int!]
) {
  assignUserFlair(
    boardId: $boardId
    templateId: $templateId
    customText: $customText
    customBackgroundColor: $customBackgroundColor
    emojiIds: $emojiIds
  ) {
    id
    userId
    customText
    isApproved
    template {
      templateName
    }
  }
}
```

**Approval Workflow:**
- If `template.requiresApproval = true`, flair is created with `isApproved = false`
- User won't see flair displayed until moderator approves
- If `requiresApproval = false`, flair is immediately active

#### Remove User Flair

```graphql
mutation RemoveUserFlair($boardId: Int!) {
  removeUserFlair(boardId: $boardId)
}
```

#### Approve User Flair (Moderators)

```graphql
mutation ApproveUserFlair($flairId: Int!, $approved: Boolean!) {
  approveUserFlair(flairId: $flairId, approved: $approved) {
    id
    isApproved
    approvedBy
  }
}
```

#### Bulk Moderate User Flairs (Moderators)

```graphql
mutation BulkModerateUserFlairs($approvals: [Int!]!, $rejections: [Int!]!) {
  bulkModerateUserFlairs(approvals: $approvals, rejections: $rejections) {
    approved
    rejected
  }
}
```

#### Update Flair Filters

```graphql
mutation UpdateFlairFilters($boardId: Int, $hideFlairs: [Int!]!, $showFlairs: [Int!]!) {
  updateFlairFilters(
    boardId: $boardId
    hideFlairs: $hideFlairs
    showFlairs: $showFlairs
  ) {
    id
    filterType
    templateId
  }
}
```

**Behavior:**
- `hideFlairs`: Posts with these flairs will be hidden from feed
- `showFlairs`: Only posts with these flairs will be shown (if any show filters exist)
- Flairs cannot be in both lists simultaneously
- If `boardId` is null, filters apply globally

#### Clear Flair Filters

```graphql
mutation ClearFlairFilters($boardId: Int) {
  clearFlairFilters(boardId: $boardId)
}
```

#### Hide Flair

```graphql
mutation HideFlair($templateId: Int!, $boardId: Int) {
  hideFlair(templateId: $templateId, boardId: $boardId) {
    id
    filterType
  }
}
```

#### Highlight Flair

```graphql
mutation HighlightFlair($templateId: Int!, $boardId: Int) {
  highlightFlair(templateId: $templateId, boardId: $boardId) {
    id
    filterType
  }
}
```

#### Unhide Flair

```graphql
mutation UnhideFlair($templateId: Int!, $boardId: Int) {
  unhideFlair(templateId: $templateId, boardId: $boardId)
}
```

---

## Permission System

### Flair Permissions

Flair management has been added to both admin and moderator permission systems:

```rust
// Admin permissions
pub enum AdminPerms {
    // ... existing perms
    Flair = 1 << 8,  // Manage site-wide flair settings
}

// Moderator permissions
pub enum ModPerms {
    // ... existing perms
    Flair = 1 << 8,  // Manage board flair templates and approvals
}
```

### Permission Matrix

| Action | Post Author | Moderator | Admin |
|--------|-------------|-----------|-------|
| Create flair template | ❌ | ✅ (board) | ✅ (any board) |
| Update flair template | ❌ | ✅ (board) | ✅ (any board) |
| Delete flair template | ❌ | ✅ (board) | ✅ (any board) |
| Assign post flair | ✅ (own post) | ✅ (any post) | ✅ (any post) |
| Remove post flair | ✅ (own post) | ✅ (any post) | ✅ (any post) |
| Assign user flair | ✅ (self) | ✅ (anyone) | ✅ (anyone) |
| Remove user flair | ✅ (self) | ✅ (anyone) | ✅ (anyone) |
| Approve user flair | ❌ | ✅ (board) | ✅ (any board) |
| Manage flair filters | ✅ (own) | ✅ (own) | ✅ (own) |

### Mod-Only Flairs

Templates with `mod_only = true`:
- Only moderators and admins can assign to posts/users
- Regular users cannot select these flairs
- Useful for marking official announcements, warnings, etc.

### Approval Workflow

Templates with `requires_approval = true`:
1. User assigns flair to themselves
2. Flair is created with `is_approved = false`
3. Flair is hidden until moderator approves
4. Moderator uses `approveUserFlair` or `bulkModerateUserFlairs`
5. Flair becomes visible with `is_approved = true`

---

## Frontend Integration

### Vue 3 Components

The frontend includes comprehensive components for flair management:

**Location**: `/home/kroner/Desktop/code/tinyboards-fe/components/flair/`

#### Key Components:

1. **FlairEditor.vue** - Complete flair template editor
   - Text and background color pickers
   - Gradient editor with presets
   - Text shadow customization
   - Animation selection
   - Emoji picker integration
   - Real-time preview

2. **FlairPicker.vue** - User-facing flair selection
   - Browse available templates
   - Search and filter
   - Apply custom text (if editable)
   - Preview before applying

3. **FlairBadge.vue** - Display component
   - Renders flair with all styling
   - Supports gradients, shadows, animations
   - Emoji rendering
   - Responsive sizing

4. **FlairCategoryManager.vue** - Organize templates
   - Drag-and-drop reordering
   - Category-based grouping
   - Bulk operations

5. **FlairFilterSettings.vue** - User filter preferences
   - Hide unwanted flairs
   - Highlight preferred flairs
   - Board-specific or global filters

### TypeScript Types

**Location**: `/home/kroner/Desktop/code/tinyboards-fe/types/flair.ts`

```typescript
export interface FlairStyle {
  backgroundColor?: string
  gradient?: FlairGradient
  textColor?: string
  borderColor?: string
  borderWidth?: number
  borderRadius?: number
  textShadow?: FlairTextShadow
  fontWeight?: 'normal' | 'medium' | 'semibold' | 'bold'
  animation?: 'pulse' | 'shimmer' | 'bounce' | 'none'
  glow?: boolean
  glowColor?: string
}

export interface FlairTemplate {
  id?: number
  boardId?: number
  flairType: FlairType
  text: string
  style: FlairStyle
  emoji?: FlairEmoji
  category?: FlairCategory
  isUserSelectable: boolean
  isModOnly: boolean
  createdAt: string
  updatedAt: string
}
```

### Usage Example

```vue
<template>
  <div>
    <!-- Display flair on post -->
    <FlairBadge
      v-if="post.flair"
      :flair="post.flair"
      size="md"
    />

    <!-- Flair picker for users -->
    <FlairPicker
      v-if="showPicker"
      :board-id="boardId"
      :flair-type="'user'"
      @select="handleFlairSelect"
    />
  </div>
</template>

<script setup lang="ts">
const handleFlairSelect = async (template: FlairTemplate, customText?: string) => {
  await $gql.default.mutation({
    mutation: AssignUserFlairDocument,
    variables: {
      boardId: boardId.value,
      templateId: template.id,
      customText
    }
  })
}
</script>
```

---

## Configuration & Limits

### Global Constants

**Location**: `crates/api/src/helpers/flair.rs`

```rust
pub const BOARD_FLAIR_LIMIT: usize = 50;  // Max templates per board per type
pub const MAX_FLAIR_TEXT_LENGTH: usize = 200;
pub const MAX_EMOJI_PER_FLAIR: usize = 5;
```

### Board-Level Configuration

Each board can configure:
- Maximum number of post flair templates (up to 50)
- Maximum number of user flair templates (up to 50)
- Whether to allow user-selectable post flairs
- Whether to require approval for user flairs
- Default flair display order

### Validation Rules

1. **Template Names**: 1-100 characters
2. **Flair Text**: 0-200 characters (configurable per template)
3. **Emoji Limit**: Maximum 5 emojis per flair
4. **Color Format**: Hex colors with or without '#'
5. **Gradient Stops**: Minimum 2 stops required
6. **Template Quota**: Maximum 50 templates per board per type

### Helper Functions

**Validate Emoji IDs** (`crates/api/src/helpers/flair.rs:61-86`)
```rust
pub async fn validate_emoji_ids(
    pool: &DbPool,
    emoji_ids: &[i32],
) -> Result<(), TinyBoardsError>
```
- Verifies all emoji IDs exist in database
- Checks that emojis are active
- Returns error if any emoji is invalid

**Check Flair Quota** (`crates/api/src/helpers/flair.rs:88-127`)
```rust
pub async fn check_flair_quota(
    pool: &DbPool,
    board_id: i32,
    flair_type: &str,
) -> Result<(), TinyBoardsError>
```
- Prevents boards from exceeding 50 template limit
- Counts only active templates
- Separate quotas for post and user flairs

---

## Best Practices

### For Moderators

1. **Organize with Categories**: Group related flairs together for easier navigation
2. **Use Display Order**: Set logical ordering for frequently used flairs
3. **Limit Choices**: Too many options overwhelm users; 10-20 templates is ideal
4. **Consistent Styling**: Use similar color schemes for related categories
5. **Clear Names**: Use descriptive template names ("Bug Report", "Feature Request")
6. **Editable Text**: Allow users to customize flair text when appropriate
7. **Approval Workflow**: Only require approval for sensitive user flairs

### For Developers

1. **Cache Templates**: Flair templates rarely change; cache aggressively
2. **Batch Operations**: Use `bulkModerateUserFlairs` for efficiency
3. **Validate Early**: Check permissions before database operations
4. **Use DataLoader**: Prevent N+1 queries when loading flairs for multiple posts
5. **Index Properly**: Ensure foreign keys have indexes for fast lookups
6. **Update Aggregates**: Increment usage_count when flairs are assigned

### For Users

1. **Filter Wisely**: Use flair filters to curate your feed
2. **Custom Text**: Personalize editable flairs with relevant information
3. **Respect Guidelines**: Follow board rules for appropriate flair usage
4. **Update Regularly**: Change user flair to reflect current interests

### Performance Considerations

1. **Aggregate Tables**: Pre-compute statistics to avoid counting at query time
2. **Partial Indexes**: Index only active flairs for faster queries
3. **JSONB Indexing**: Use GIN indexes on style_config for advanced queries
4. **Connection Pooling**: Flair operations can be high-volume; ensure adequate pool size

---

## Implementation Files

### Backend

**Models**: `crates/db/src/models/flair/`
- `flair_template.rs` - Template definitions
- `post_flair.rs` - Post flair assignments
- `user_flair.rs` - User flair assignments
- `flair_aggregates.rs` - Statistics and metrics
- `flair_filter.rs` - User filter preferences

**API Mutations**: `crates/api/src/mutations/flair/`
- `template.rs` - CRUD operations for templates
- `assignment.rs` - Assign/remove flairs
- `filter.rs` - User filter management

**API Queries**: `crates/api/src/queries/flair.rs`
- Get templates, assignments, filters
- Pending approval queries for moderators

**Helpers**: `crates/api/src/helpers/flair.rs`
- Validation functions
- Quota checking
- Permission verification

### Frontend

**Components**: `tinyboards-fe/components/flair/`
- `editor/FlairEditor.vue` - Complete template editor
- `picker/FlairPicker.vue` - User-facing selection
- `display/FlairBadge.vue` - Render component
- `settings/FlairFilterSettings.vue` - User preferences
- `manager/FlairCategoryManager.vue` - Category organization

**Types**: `tinyboards-fe/types/flair.ts`
- TypeScript interfaces
- Helper functions (gradientToCSS, flairStyleToCSS)
- Preset constants

**Pages**: `tinyboards-fe/pages/b/[name]/flair.vue`
- Board flair management interface

---

## Migration Files

**Location**: `/home/kroner/Desktop/code/tinyboards/migrations/`

1. `2025-10-22-205901_add_flair_core_tables/` - Core flair tables
2. `2025-10-22-211042_add_flair_aggregates/` - Statistics tables
3. `2025-10-22-211543_add_flair_filters/` - User filter system
4. `2025-10-22-212158_add_flair_permissions/` - Permission system updates

Each migration includes:
- `up.sql` - Schema changes to apply
- `down.sql` - Rollback procedures

---

## Future Enhancements

Potential improvements for future releases:

1. **Site-Wide Flairs**: Global flair templates available across all boards
2. **Flair Analytics**: Detailed usage statistics and trending flairs
3. **Flair Badges**: Award-style achievements displayed as special flairs
4. **Import/Export**: Share flair templates between boards or instances
5. **Advanced Animations**: More animation options (rainbow, pulse variants)
6. **Custom Fonts**: Allow font selection in flair styling
7. **Flair Presets**: Quick-apply style templates for common use cases
8. **Bulk Assignment**: Assign flairs to multiple posts at once
9. **Flair History**: Track changes to flair assignments over time
10. **API Rate Limiting**: Prevent flair spam or abuse

---

## Support

For issues, questions, or feature requests related to the flair system:

1. Check existing GitHub issues
2. Review this documentation thoroughly
3. Examine the source code in the implementation files listed above
4. Create a new issue with detailed reproduction steps

---

**Last Updated**: 2025-10-22
**Version**: 1.0.0
**Commits**: Backend 56aea91, Frontend 2b80769
