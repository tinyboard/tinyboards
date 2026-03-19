# Frequently Asked Questions

## Account

### How do I change my password?

Go to **Settings** → **Account** → **Change Password**. You'll need your current password.

### How do I delete my account?

Go to **Settings** → **Account** → **Delete Account**. Enter your password to confirm. This is a soft delete — your username will no longer be available and your profile will be hidden, but your posts and comments remain attributed to a deleted user.

### I forgot my password. How do I reset it?

If the site has email configured and you verified your email address, use the "Forgot Password" link on the login page. You'll receive a reset link via email.

### Can I change my username?

Usernames cannot be changed after registration. If you need a different username, create a new account.

### Why can't I register?

The site administrator controls registration. Possible reasons:
- Registration is closed
- Registration requires an invite code
- Registration requires an admin-approved application
- Your CAPTCHA answer was incorrect

Contact the site administrator if you need an invite or have questions about registration.

## Content

### What's the difference between feed posts and threads?

**Feed posts** are like Reddit posts — they support voting, have score-based sorting, and are best for sharing links, images, or short discussions.

**Threads** are like forum threads — no voting, sorted by activity, and better for ongoing discussions.

### Can I edit my posts after submitting?

Yes. Click the edit button on your post. The post will show an "edited" indicator.

### Can I recover a deleted post?

No. Once you delete a post, it cannot be recovered by you. Moderators and admins may still be able to see deleted content.

### Why was my post removed?

A moderator removed it for violating board rules or site rules. Check the board's sidebar for rules. You can view removal reasons in your notifications if the moderator provided one.

### What does NSFW mean?

"Not Safe For Work." Posts marked NSFW contain content that may be inappropriate for some settings. NSFW content is hidden by default — you can enable it in your settings. Some instances disable NSFW entirely.

## Boards

### How do I create a board?

Visit `/createBoard` to use the board creation wizard. Board creation may be restricted — check with the site admin.

### Can I transfer ownership of a board?

The top-ranked moderator effectively "owns" the board. Admin users can transfer board ownership. Contact a site admin for ownership transfers.

### How do I report a board?

Contact a site administrator. Boards can be banned site-wide if they violate instance rules.

## Moderation

### How do I become a moderator?

A board's existing moderators can invite you. You'll see the invitation in your notifications. Alternatively, create your own board.

### Can moderators see my private messages?

No. Moderators can only see public content in their boards. Only site administrators have broader access, and even then, private messages are not accessible through the admin panel.

### How do I report a post or comment?

Click the report button (flag icon) on any post or comment. Provide a reason for the report. Reports are reviewed by board moderators and site administrators.

## Streams

### What are streams?

Streams are custom feeds that combine posts from multiple boards. You can also filter by flair to create very specific feeds.

### Are my private streams visible to anyone?

No. Private streams are only visible to you. Unlisted streams are visible to anyone with the direct link or share token. Public streams appear in the Discover page.

## Technical

### What browsers are supported?

TinyBoards works in all modern browsers: Chrome, Firefox, Safari, and Edge. Internet Explorer is not supported.

### Can I use TinyBoards on mobile?

Yes. The interface is responsive and works on mobile devices. The rich text editor has a mobile-optimized mode.

### Is there an API?

Yes. TinyBoards has a GraphQL API at `/api/v2/graphql`. See the [API documentation](../api/README.md) for details.

### Is TinyBoards federated?

Federation support is not currently implemented. Each TinyBoards instance is standalone.
