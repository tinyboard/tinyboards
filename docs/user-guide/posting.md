# Posting

TinyBoards has two types of content: **feed posts** and **threads**. Both are created within boards.

## Table of Contents

- [Feed Posts](#feed-posts)
- [Threads](#threads)
- [Creating a Post](#creating-a-post)
- [Post Types](#post-types)
- [Formatting](#formatting)
- [Editing and Deleting](#editing-and-deleting)
- [Voting](#voting)
- [Saving and Hiding](#saving-and-hiding)

## Feed Posts

Feed posts are the primary content type — similar to Reddit posts. They appear in a board's feed section and support:

- **Voting** — Upvote or downvote (if enabled by the site admin)
- **Sorting** — Hot, New, Top, Old, Most Comments, Controversial
- **Flairs** — Optional colored tags for categorization
- **Comments** — Threaded comment trees

Feed posts have URLs like `/b/boardname/feed/post-id/post-slug`.

## Threads

Threads are forum-style discussions. They differ from feed posts:

- **No voting** — Threads are sorted by most recent activity, not score
- **Discussion-focused** — Better suited for long-running conversations

Threads have URLs like `/b/boardname/threads/thread-id/thread-slug`.

## Creating a Post

1. Navigate to a board or click **Submit** in the navbar
2. Select the **board** (if not already in one)
3. Choose the **post type** (text, link, image, or video)
4. Fill in the details:
   - **Title** — Required, up to 200 characters
   - **Body** — The post content (optional for link/image posts)
   - **URL** — For link posts
   - **Image** — Upload an image
   - **Flair** — Select a flair tag (if the board has flairs)
   - **NSFW** — Mark as not safe for work
5. Click **Submit**

## Post Types

| Type | Description | Required Fields |
|------|-------------|-----------------|
| Text | Written content | Title, Body |
| Link | External URL | Title, URL |
| Image | Uploaded image | Title, Image |
| Video | Video embed | Title, URL |

Link posts automatically fetch a preview (title, description, thumbnail) from the linked URL.

## Formatting

The post body supports rich text editing with the TipTap editor. Available formatting:

### Text Formatting
- **Bold**, *italic*, ~~strikethrough~~
- Headings (H1 through H6)
- Blockquotes
- Inline code and code blocks (with syntax highlighting)
- Ordered and unordered lists
- Task lists

### Rich Content
- Links
- Images (inline)
- Tables
- Horizontal rules
- Text color

### Editor Modes

You can switch between editor modes in your settings:

| Mode | Description |
|------|-------------|
| Rich Text | Visual WYSIWYG editor (default) |
| Markdown | Write in Markdown syntax |
| Plain Text | No formatting |

### Mentions

Type `@username` to mention another user. They'll receive a notification.

### Custom Emoji

Type `:emojiname:` to insert custom emoji defined by the site or board.

## Editing and Deleting

### Editing

Click the **Edit** button on your own post to modify the title or body. Edited posts show an "edited" indicator with the edit timestamp.

### Deleting

Click **Delete** on your post. This is a soft delete — the post is marked as deleted but remains in the database. Moderators and admins can still see deleted posts.

## Voting

Feed posts support upvoting and downvoting (if enabled by the site admin):

- **Upvote** — Increases the post's score. Signals that you find the content valuable.
- **Downvote** — Decreases the post's score. Some instances disable downvoting.
- Click the same vote button again to remove your vote.
- You cannot vote on your own posts.

Post scores affect sorting — higher-scored posts appear higher in Hot and Top sorts.

## Saving and Hiding

### Saving

Click **Save** on any post to bookmark it. View your saved posts at `/@yourusername/saved` or in your settings.

### Hiding

Click **Hide** to remove a post from your feed. Hidden posts won't appear in listings for you. Manage hidden posts at `/settings/hidden`.
