# Flairs

Flairs are colored tags that can be applied to posts and users within a board. They help categorize content and add visual identity.

## Table of Contents

- [Post Flairs](#post-flairs)
- [User Flairs](#user-flairs)
- [Flair Filtering](#flair-filtering)
- [Managing Flairs (Moderators)](#managing-flairs-moderators)
- [Flair Categories](#flair-categories)
- [Flair Styling](#flair-styling)

## Post Flairs

Post flairs are tags attached to posts. They appear next to the post title in listings and on the post page.

### Assigning a Post Flair

When creating or editing a post, you can select a flair from the board's available flair templates. Some boards may require a flair on every post.

### Filtering by Flair

Click on a post flair to view all posts in that board with the same flair. The URL format is `/b/boardname/flair/flairId`.

## User Flairs

User flairs appear next to your username within a specific board. They're like a badge showing your role or identity in that community.

### Selecting a User Flair

1. Visit the board's flair selection page
2. Choose from available user flair templates
3. Some boards require moderator approval for user flair changes

### Approval Workflow

If the board requires flair approval:
1. You select your desired flair
2. The selection is marked as "pending"
3. A board moderator approves or rejects the request

## Flair Filtering

You can configure how flairs affect your feed view:

### Filter Modes

| Mode | Behavior |
|------|----------|
| Include | Only show posts with selected flairs |
| Exclude | Hide posts with selected flairs |

Configure your flair filters per board through the board's flair filter settings.

## Managing Flairs (Moderators)

Board moderators manage flairs at `/b/boardname/flairs`.

### Creating a Flair

1. Go to `/b/boardname/flairs/create`
2. Set the flair properties:
   - **Name** — Display text
   - **Type** — Post flair or user flair
   - **Style** — Colors, borders, gradients, etc.
   - **Category** — Optional category grouping
   - **Active** — Whether the flair is available for selection

### Editing and Deleting

- Edit: `/b/boardname/flairs/[id]/edit`
- Deactivate a flair to hide it from selection without removing it from existing posts
- Delete to permanently remove a flair template

### Pending User Flairs

If your board requires approval for user flairs, review pending requests at the mod panel.

## Flair Categories

Flairs can be organized into categories for easier browsing. Manage categories at `/b/boardname/flairs/categories`.

Categories help when a board has many flairs — users see them grouped by category in the selection UI.

## Flair Styling

Flair templates support rich visual customization:

| Property | Description |
|----------|-------------|
| Background color | Solid color or gradient |
| Text color | Flair label color |
| Border | Width, color, style, radius |
| Shadow | Box shadow effects |
| Animation | Subtle animations (shimmer, pulse, etc.) |
| Gradient | Linear gradient backgrounds |

Moderators use the flair editor at `/b/boardname/flairs/create` or `/b/boardname/flairs/[id]/edit` to design flairs visually. A live preview shows how the flair will look.
