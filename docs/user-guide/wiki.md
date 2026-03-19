# Wiki

Boards can have wiki pages for collaborative documentation, guides, and reference material.

## Table of Contents

- [Viewing Wiki Pages](#viewing-wiki-pages)
- [Creating a Page](#creating-a-page)
- [Editing a Page](#editing-a-page)
- [Revision History](#revision-history)
- [Permissions](#permissions)

## Viewing Wiki Pages

If a board has the wiki enabled, you can access it at `/b/boardname/wiki`. The wiki index shows all available pages.

Each page has a URL like `/b/boardname/wiki/page-slug`.

## Creating a Page

If you have edit permissions:

1. Go to `/b/boardname/wiki/new`
2. Set the page title and URL slug
3. Write the page content using the rich text editor
4. Save the page

## Editing a Page

1. Open the wiki page
2. Click **Edit** (or go to `/b/boardname/wiki/page-slug/edit`)
3. Make your changes
4. Save — the previous version is preserved in the revision history

## Revision History

Every edit creates a new revision. View the history at the wiki page's history link:

- See who made each change and when
- View the content of any previous revision
- Revert to a previous revision (moderator action)

## Permissions

Board moderators control wiki permissions:

| Permission Level | Who Can View | Who Can Edit |
|-----------------|--------------|--------------|
| Public | Everyone | Depends on edit permission |
| Members | Board subscribers | Board subscribers |
| Mods Only | Board moderators | Board moderators |
| Locked | Everyone (read-only) | No one (until unlocked) |

View and edit permissions are configured independently. For example, a board could allow everyone to view pages but restrict editing to subscribers.

Moderators can also require approval for wiki edits — changes are reviewed before going live.
