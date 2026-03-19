# Boards

Boards are the core organizational unit of TinyBoards — each board is a community focused on a topic.

## Table of Contents

- [Finding Boards](#finding-boards)
- [Subscribing](#subscribing)
- [Board Sections](#board-sections)
- [Creating a Board](#creating-a-board)
- [Board Settings](#board-settings)

## Finding Boards

### Board Directory

Visit `/boards` to browse all boards on the instance. You can:
- **Search** by name or description
- **Sort** by subscribers, post count, or creation date
- **Filter** to show or hide NSFW boards

### Direct URL

Every board has a URL at `/b/boardname`. For example, `/b/gaming`.

## Subscribing

Click the **Subscribe** button on any board page to add it to your home feed. Subscribed boards appear in:
- Your **Home** feed (`/home`) — shows posts from all your subscriptions
- The board navigator dropdown in the navbar

To unsubscribe, click the button again.

### Home vs All

- **Home** (`/home`) — Posts from boards you've subscribed to
- **All** (`/all`) — Posts from every board on the instance (except those excluded by admins)

## Board Sections

Boards can have one or both of these content sections:

### Feed

The feed section contains **feed posts** — links, images, text posts, and videos. Feed posts have:
- Upvoting and downvoting
- Sorting by Hot, New, Top, and Controversial
- Flair tags

Access via: `/b/boardname` or `/b/boardname/feed`

### Threads

The threads section contains **forum-style threads** — longer-form discussions. Threads have:
- No voting (sorted by activity instead)
- A more traditional forum feel

Access via: `/b/boardname/threads`

Board moderators configure which sections are available.

## Creating a Board

If the site allows board creation (check with the site admin), you can create a new board:

1. Click **Create Board** or visit `/createBoard`
2. Follow the wizard:
   - **Name** — The URL-friendly board name (lowercase, no spaces)
   - **Title** — The display title
   - **Description** — What the board is about
   - **Colors** — Primary, secondary, and hover colors
   - **Images** — Icon and banner
3. Submit to create the board

You become the board's first moderator automatically.

### Board Creation Modes

The site admin controls who can create boards:

| Mode | Who Can Create |
|------|----------------|
| Open | Any registered user |
| Admin Only | Only site administrators |
| Trusted Users | Users who meet reputation/age requirements |
| Disabled | No new boards can be created |

## Board Settings

Board moderators can configure these settings at `/b/boardname/mod`:

### Appearance

- Board icon and banner
- Primary, secondary, and hover colors
- Sidebar content (supports rich text)

### Content

- Which sections are enabled (feed, threads, or both)
- Whether posting is restricted to moderators
- NSFW flag
- Default content section

### Wiki

- Enable/disable the board wiki
- Set default view and edit permissions
- Require approval for wiki edits

### Moderation

- Add or remove moderators
- Set moderator permissions
- Manage board bans
- Configure flair templates
- Manage custom emoji
- Configure reaction settings
