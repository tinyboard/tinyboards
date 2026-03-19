# Notifications

TinyBoards sends notifications for activity related to your account.

## Table of Contents

- [Notification Types](#notification-types)
- [Viewing Notifications](#viewing-notifications)
- [Notification Settings](#notification-settings)
- [Email Notifications](#email-notifications)

## Notification Types

| Type | Trigger |
|------|---------|
| **Comment Reply** | Someone replies to your comment |
| **Post Reply** | Someone comments on your post |
| **Mention** | Someone @mentions your username |
| **Private Message** | You receive a new private message |
| **Mod Action** | A moderation action affects your content |
| **System** | Site-wide announcements or account-related events |

## Viewing Notifications

Notifications appear in your inbox at `/inbox`. You can:

- **Filter** by type (replies, mentions, messages, all)
- **Mark as read** individually or all at once
- **Delete** notifications you no longer need

The navbar shows an unread count badge when you have new notifications.

## Notification Settings

Customize which notifications you receive at `/settings/notifications`:

| Setting | Description |
|---------|-------------|
| Comment replies | Notify when someone replies to your comments |
| Post replies | Notify when someone comments on your posts |
| Mentions | Notify when someone @mentions you |
| Private messages | Notify for new private messages |
| Board invites | Notify when invited to moderate a board |
| Mod actions | Notify when a mod action affects your content |

Each type can be toggled on or off independently.

## Email Notifications

If the site has SMTP configured and you've verified your email address, you can opt into email notifications at `/settings/notifications`.

When enabled, you'll receive email summaries for notifications you haven't seen in the web UI. Email notifications respect your notification type settings — if you've disabled mention notifications, you won't receive email for mentions either.

### Privacy

- Your email address is never visible to other users
- Email notifications are opt-in
- You can unsubscribe from emails at any time through settings
- Notification content (the actual message text) is included in emails so you don't have to visit the site for every notification
