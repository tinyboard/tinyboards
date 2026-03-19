# Moderation

Board moderators manage content and users within their boards. This guide covers moderator tools and workflows.

## Table of Contents

- [Becoming a Moderator](#becoming-a-moderator)
- [Moderator Panel](#moderator-panel)
- [Content Moderation](#content-moderation)
- [User Moderation](#user-moderation)
- [Moderation Log](#moderation-log)
- [Reports](#reports)
- [Moderator Permissions](#moderator-permissions)

## Becoming a Moderator

- **Board creator** — Automatically becomes the first moderator with full permissions.
- **Invited** — Existing moderators can invite you. You'll receive an invitation to accept or decline.
- **Admin-assigned** — Site administrators can assign moderators to any board.

## Moderator Panel

Access the mod panel at `/b/boardname/mod`. It includes:

| Section | URL | Description |
|---------|-----|-------------|
| Overview | `/b/boardname/mod` | Quick stats and pending items |
| Appearance | `/b/boardname/mod/appearance` | Board icon, banner, colors |
| Sidebar | `/b/boardname/mod/sidebar` | Board sidebar content |
| Sections | `/b/boardname/mod/sections` | Feed/threads configuration |
| Moderators | `/b/boardname/mod/mods` | Add/remove mods, set permissions |
| Bans | `/b/boardname/mod/bans` | Board-level user bans |
| Emoji | `/b/boardname/mod/emojis` | Board custom emoji |
| Reactions | `/b/boardname/mod/reactions` | Reaction settings |

## Content Moderation

### Removing Content

Click the mod menu on any post or comment in your board to:

- **Remove** — Hides the content from normal users. Removed content is visible in the mod panel.
- **Approve** — Restores removed content.

### Locking

- **Lock Post** — Prevents new comments on a post.
- **Unlock Post** — Re-enables commenting.

### Featuring / Pinning

- **Feature (Board)** — Pins a post to the top of the board's feed.
- **Feature (Site)** — Pins a post site-wide (admin only).
- **Unfeature** — Removes the pin.

Each board can have multiple featured posts.

## User Moderation

### Board Bans

Ban a user from your board:

1. Go to `/b/boardname/mod/bans` or use the mod menu on a user's post/comment
2. Select the user
3. Optionally set a reason and expiry date
4. Confirm the ban

Banned users cannot post or comment in the board. Bans can be:
- **Permanent** — No expiry
- **Temporary** — Automatically lifted after the expiry date

### Viewing Banned Users

The bans page shows all currently banned users with:
- Ban reason
- Ban date
- Expiry date (if temporary)
- Who issued the ban

## Moderation Log

Every moderation action is recorded in the moderation log, accessible at the mod panel. The log shows:

- **Action** — What was done (remove, ban, lock, etc.)
- **Moderator** — Who performed the action
- **Target** — The affected post, comment, or user
- **Timestamp** — When it happened
- **Reason** — If one was provided

The site admin can configure whether moderator names are visible to regular users in the public moderation log.

## Reports

Users can report posts and comments. Reports appear in the moderation queue.

### Handling Reports

1. Go to `/b/boardname/mod` or the reports section
2. Review the reported content and the reporter's reason
3. Take action:
   - **Remove** the content if it violates rules
   - **Resolve** the report if no action is needed
   - **Dismiss** false reports

### Report Types

| Type | Location |
|------|----------|
| Post reports | `/b/boardname/mod` → Reports |
| Comment reports | `/b/boardname/mod` → Reports |

Site admins also see reports from all boards in the admin panel.

## Moderator Permissions

Board moderators have granular permissions:

| Permission | What It Allows |
|------------|----------------|
| Content | Remove/approve posts and comments, lock/unlock posts |
| Users | Ban/unban users from the board |
| Appearance | Change board icon, banner, colors, sidebar |
| Config | Modify board settings (sections, wiki, etc.) |
| Flair | Create and manage flair templates |
| Full | All permissions |

The board creator (or higher-ranked moderator) assigns permissions when adding new moderators. Moderators are ranked — higher-ranked mods can manage lower-ranked ones, but not the reverse.
