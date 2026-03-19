# Streams

Streams are custom feeds that let you curate content from multiple boards and flair filters into a single view.

## Table of Contents

- [What Are Streams?](#what-are-streams)
- [Creating a Stream](#creating-a-stream)
- [Stream Subscriptions](#stream-subscriptions)
- [Sharing Streams](#sharing-streams)
- [Following Streams](#following-streams)
- [Discovering Streams](#discovering-streams)

## What Are Streams?

A stream is a personalized feed that pulls posts from selected boards and flair filters. Instead of switching between individual boards, you can create a stream that combines them.

Examples:
- A "Gaming News" stream that pulls posts tagged with "News" flair from `/b/gaming`, `/b/pcgaming`, and `/b/console`
- A "Creative" stream that combines `/b/art`, `/b/writing`, and `/b/music`
- A "Tech" stream with only "Discussion" and "Tutorial" flair types from tech boards

## Creating a Stream

1. Go to `/streams/create`
2. Fill in the stream details:
   - **Name** — Display name for your stream
   - **Description** — What the stream is about
   - **Slug** — URL-friendly identifier (e.g., `gaming-news`)
   - **Visibility** — Public, unlisted, or private
3. Add board subscriptions (select which boards to include)
4. Optionally add flair filters (only include posts with specific flairs)
5. Save the stream

Your stream is accessible at `/streams/@yourusername/stream-slug`.

## Stream Subscriptions

Streams pull content based on two types of subscriptions:

### Board Subscriptions

Select which boards to include in your stream. Posts from all selected boards appear in the stream feed.

### Flair Subscriptions

Optionally filter posts by flair. When flair subscriptions are set, only posts with matching flairs from the subscribed boards appear.

You can add or remove subscriptions at any time by editing the stream.

## Sharing Streams

### Visibility Levels

| Level | Who Can See It |
|-------|----------------|
| **Public** | Anyone, appears in Discover |
| **Unlisted** | Anyone with the direct link |
| **Private** | Only you |

### Share Tokens

For unlisted or private streams, you can generate a **share token** — a special link that lets anyone with the token view the stream:

- URL format: `/s/token-here`
- Regenerate the token at any time to revoke previous shares
- Edit your stream and click "Regenerate Share Token"

## Following Streams

You can follow other users' public streams:

1. Find a stream via Discover or a direct link
2. Click **Follow**
3. The stream appears in your streams list at `/streams`

### Navbar Pinning

Pin frequently used streams to your navigation bar for quick access. Reorder pinned streams from the streams management page.

## Discovering Streams

Browse public streams at `/streams/discover`:

- Sort by followers, recent activity, or creation date
- Search by name or description
- Browse streams from users you follow
