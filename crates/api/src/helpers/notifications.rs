use tinyboards_db::{
    models::notification::notifications::{Notification, NotificationForm},
    utils::DbPool,
};
use chrono::Utc;

/// Create a notification for a comment reply
pub async fn create_reply_notification(
    pool: &DbPool,
    recipient_user_id: i32,
    comment_id: i32,
    post_id: Option<i32>,
) -> Result<Notification, diesel::result::Error> {
    let form = NotificationForm {
        kind: "reply".to_string(),
        recipient_user_id,
        comment_id: Some(comment_id),
        post_id,
        message_id: None,
        is_read: Some(false),
        created: Some(Utc::now().naive_utc()),
    };

    Notification::create(pool, &form).await
}

/// Create a notification for a mention in a comment
pub async fn create_mention_notification(
    pool: &DbPool,
    recipient_user_id: i32,
    comment_id: i32,
    post_id: Option<i32>,
) -> Result<Notification, diesel::result::Error> {
    let form = NotificationForm {
        kind: "mention".to_string(),
        recipient_user_id,
        comment_id: Some(comment_id),
        post_id,
        message_id: None,
        is_read: Some(false),
        created: Some(Utc::now().naive_utc()),
    };

    Notification::create(pool, &form).await
}

/// Create a notification for a mention in a post
pub async fn create_post_mention_notification(
    pool: &DbPool,
    recipient_user_id: i32,
    post_id: i32,
) -> Result<Notification, diesel::result::Error> {
    let form = NotificationForm {
        kind: "mention".to_string(),
        recipient_user_id,
        comment_id: None,
        post_id: Some(post_id),
        message_id: None,
        is_read: Some(false),
        created: Some(Utc::now().naive_utc()),
    };

    Notification::create(pool, &form).await
}

/// Create a notification for a post reply
pub async fn create_post_reply_notification(
    pool: &DbPool,
    recipient_user_id: i32,
    post_id: i32,
    comment_id: i32,
) -> Result<Notification, diesel::result::Error> {
    let form = NotificationForm {
        kind: "post_reply".to_string(),
        recipient_user_id,
        comment_id: Some(comment_id),
        post_id: Some(post_id),
        message_id: None,
        is_read: Some(false),
        created: Some(Utc::now().naive_utc()),
    };

    Notification::create(pool, &form).await
}

/// Extract @mentions from text
/// Returns list of unique usernames mentioned (without the @ symbol)
pub fn extract_mentions(text: &str) -> Vec<String> {
    use std::collections::HashSet;
    let mut mentions = HashSet::new();

    // Find @username patterns
    // Matches @username where username is alphanumeric + underscore
    for word in text.split_whitespace() {
        if word.starts_with('@') {
            let username = word.trim_start_matches('@')
                .trim_end_matches(|c: char| !c.is_alphanumeric() && c != '_');
            if !username.is_empty() {
                mentions.insert(username.to_string());
            }
        }
    }

    mentions.into_iter().collect()
}

/// Convert @mentions in text to HTML links
/// Example: "@alice check this out" -> "<a href=\"/@alice\">@alice</a> check this out"
pub fn convert_mentions_to_links(text: &str) -> String {
    let mut result = String::new();
    let mut last_end = 0;

    for word_match in text.match_indices(|c: char| c.is_whitespace() || c == '@') {
        let (idx, _) = word_match;

        // Check if this is the start of a mention
        if idx < text.len() && text[idx..].starts_with('@') {
            // Add everything up to this point
            result.push_str(&text[last_end..idx]);

            // Find the end of the username (alphanumeric + underscore)
            let rest = &text[idx+1..];
            let username_end = rest.find(|c: char| !c.is_alphanumeric() && c != '_')
                .unwrap_or(rest.len());

            if username_end > 0 {
                let username = &rest[..username_end];
                // Create the link
                result.push_str(&format!("<a href=\"/@{}\">@{}</a>", username, username));
                last_end = idx + 1 + username_end;
            } else {
                // Just an @ with no username, keep it
                result.push('@');
                last_end = idx + 1;
            }
        }
    }

    // Add any remaining text
    if last_end < text.len() {
        result.push_str(&text[last_end..]);
    }

    // Handle case where text starts with @
    if text.starts_with('@') && result.is_empty() {
        let username_end = text[1..].find(|c: char| !c.is_alphanumeric() && c != '_')
            .unwrap_or(text.len() - 1);
        if username_end > 0 {
            let username = &text[1..=username_end];
            result = format!("<a href=\"/@{}\">@{}</a>{}", username, username, &text[username_end+1..]);
        } else {
            result = text.to_string();
        }
    }

    if result.is_empty() {
        text.to_string()
    } else {
        result
    }
}

/// Get user IDs for mentioned usernames
pub async fn get_user_ids_for_mentions(
    pool: &DbPool,
    usernames: Vec<String>,
) -> Result<Vec<i32>, diesel::result::Error> {
    use tinyboards_db::{schema::users, utils::get_conn};
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    let conn = &mut get_conn(pool).await?;

    users::table
        .filter(users::name.eq_any(usernames))
        .filter(users::is_deleted.eq(false))
        .select(users::id)
        .load::<i32>(conn)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_mentions() {
        let text = "Hey @alice and @bob, check this out! @charlie.";
        let mut mentions = extract_mentions(text);
        mentions.sort(); // Sort for consistent comparison
        assert_eq!(mentions, vec!["alice", "bob", "charlie"]);
    }

    #[test]
    fn test_extract_mentions_empty() {
        let text = "No mentions here";
        let mentions = extract_mentions(text);
        assert_eq!(mentions.len(), 0);
    }

    #[test]
    fn test_extract_mentions_with_underscore() {
        let text = "Hello @user_name";
        let mentions = extract_mentions(text);
        assert_eq!(mentions, vec!["user_name"]);
    }

    #[test]
    fn test_extract_mentions_deduplication() {
        let text = "@alice @alice @bob @alice";
        let mut mentions = extract_mentions(text);
        mentions.sort();
        assert_eq!(mentions, vec!["alice", "bob"]);
    }

    #[test]
    fn test_convert_mentions_to_links() {
        let text = "Hey @alice check this out";
        let result = convert_mentions_to_links(text);
        assert!(result.contains("<a href=\"/@alice\">@alice</a>"));
        assert!(result.contains("check this out"));
    }

    #[test]
    fn test_convert_mentions_multiple() {
        let text = "@alice and @bob are here";
        let result = convert_mentions_to_links(text);
        assert!(result.contains("<a href=\"/@alice\">@alice</a>"));
        assert!(result.contains("<a href=\"/@bob\">@bob</a>"));
    }

    #[test]
    fn test_convert_mentions_no_mentions() {
        let text = "No mentions here";
        let result = convert_mentions_to_links(text);
        assert_eq!(result, text);
    }
}
