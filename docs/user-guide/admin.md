# Instance Administration

Site administrators manage the TinyBoards instance through the admin panel at `/admin`.

## Table of Contents

- [Admin Panel Overview](#admin-panel-overview)
- [Site Settings](#site-settings)
- [User Management](#user-management)
- [Content Management](#content-management)
- [Security](#security)
- [Appearance](#appearance)
- [Registration](#registration)

## Admin Panel Overview

The admin panel is accessible at `/admin` and is restricted to users with the admin flag. It provides control over the entire instance.

| Section | URL | Description |
|---------|-----|-------------|
| Dashboard | `/admin` | Overview and statistics |
| Settings | `/admin/settings` | General site settings |
| Appearance | `/admin/appearance` | Site colors, banner, theme |
| Content | `/admin/content` | Content policy, voting, NSFW |
| Board Settings | `/admin/board_settings` | Board creation rules |
| Users | `/admin/users` | User management |
| Admins | `/admin/admins` | Admin user management |
| Bans | `/admin/bans` | Site-wide bans |
| Invites | `/admin/invites` | Invite code management |
| Security | `/admin/security` | Security and trust settings |
| Queue | `/admin/queue` | Moderation queue (all boards) |
| Reports (Posts) | `/admin/reports/posts` | Post reports from all boards |
| Reports (Comments) | `/admin/reports/comments` | Comment reports from all boards |
| Removed Posts | `/admin/removed/posts` | All removed posts |
| Removed Comments | `/admin/removed/comments` | All removed comments |
| Emojis | `/admin/emojis` | Site-wide custom emoji |
| Flairs | `/admin/flairs` | Site-wide flair management |
| Custom CSS | `/admin/css` | Custom site-wide CSS |

## Site Settings

Configure the core settings for your instance:

- **Site Name** — The display name shown in the navbar and page titles
- **Description** — A short description of your instance
- **Welcome Message** — Shown to new users
- **Legal Information** — Terms of service, privacy policy links
- **Default Feed** — Which listing type new users see by default (All, Subscribed, Local)

## User Management

### Viewing Users

Browse all users at `/admin/users`. Sort by registration date, post count, or activity.

### Admin Actions

| Action | Description |
|--------|-------------|
| **Set Admin Level** | Grant or revoke admin privileges |
| **Ban User** | Site-wide ban (with optional expiry) |
| **Unban User** | Lift a site-wide ban |
| **Purge User** | Permanently remove a user and all their content |
| **Approve Board Creation** | Allow a user to create boards (when board creation requires approval) |

### Site-Wide Bans

Site bans are more severe than board bans — the user cannot access any part of the site. To ban a user:

1. Go to `/admin/bans` or find the user in `/admin/users`
2. Enter a ban reason and optional expiry date
3. Confirm the ban

Temporary bans are automatically lifted when they expire (checked every 5 minutes by a background task).

## Content Management

### Content Policy Settings

At `/admin/content`:

- **Enable Downvotes** — Whether users can downvote posts and comments
- **Enable NSFW** — Whether NSFW content is allowed at all
- **NSFW Tagging** — Require NSFW posts to be tagged
- **Hide Modlog Mod Names** — Whether moderator names are visible in the public moderation log
- **Reports Email Admins** — Whether admins receive email notifications for new reports

### Content Filtering

- **Word Filter** — Block posts/comments containing specific words
- **Domain Filter** — Block links to specific domains
- **Allowed Post Types** — Restrict which post types (text, link, image, video) are allowed
- **Approved Image Hosts** — Only allow images from specific hosts

### Moderation Queue

The admin queue at `/admin/queue` shows reported and pending content from all boards, giving site-wide oversight.

## Security

### Trust System

Configure automatic trust levels at `/admin/security`:

- **Minimum Reputation** — Score required for trusted user status
- **Minimum Account Age** — Days before an account is considered trusted
- **Minimum Posts** — Post count required for trusted status
- **Manual Approval** — Require admin approval for trusted status

### Email Verification

Toggle email verification for new accounts. Requires SMTP to be configured (see [env-vars.md](../self-hosting/env-vars.md)).

### CAPTCHA

Enable CAPTCHA on the registration page with configurable difficulty (easy, medium, hard).

## Appearance

At `/admin/appearance`:

- **Site Icon** — The favicon and logo
- **Homepage Banner** — Banner image on the home page
- **Primary/Secondary/Hover Colors** — Site-wide color theme
- **Default Theme** — The default theme for new users
- **Default Avatar** — Avatar assigned to new users
- **Custom CSS** — Write custom CSS at `/admin/css` to further customize the site appearance

## Registration

Control how new users join at `/admin/board_settings` and `/admin/settings`:

### Registration Modes

| Mode | Behavior |
|------|----------|
| **Open** | Anyone can register |
| **Invite Only** | Users need an invite code |
| **Application Required** | Users submit an application for admin review |
| **Closed** | No new registrations |

### Invite Management

At `/admin/invites`:
- Generate invite codes
- View active invites
- Delete unused invites
- See who used each invite

### Registration Applications

When using the "Application Required" mode, review applications at the admin panel:
- View the applicant's answer to the application question
- **Approve** to create their account
- **Deny** with an optional reason
