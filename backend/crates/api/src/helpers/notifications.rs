use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use lazy_static::lazy_static;
use regex::Regex;
use tinyboards_db::{
    enums::DbNotificationKind,
    models::notification::notifications::NotificationInsertForm,
    schema::{notifications, users},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

/// Create a notification for a comment reply
pub async fn create_comment_reply_notification(
    pool: &DbPool,
    recipient_user_id: Uuid,
    comment_id: Uuid,
    actor_user_id: Uuid,
) -> Result<(), TinyBoardsError> {
    let conn = &mut get_conn(pool).await?;

    let form = NotificationInsertForm {
        kind: DbNotificationKind::CommentReply,
        recipient_user_id,
        comment_id: Some(comment_id),
        post_id: None,
        message_id: None,
        is_read: false,
        actor_user_id: Some(actor_user_id),
    };

    diesel::insert_into(notifications::table)
        .values(&form)
        .execute(conn)
        .await?;

    Ok(())
}

/// Create a notification for a mention in a comment
pub async fn create_comment_mention_notification(
    pool: &DbPool,
    recipient_user_id: Uuid,
    comment_id: Uuid,
    actor_user_id: Uuid,
) -> Result<(), TinyBoardsError> {
    let conn = &mut get_conn(pool).await?;

    let form = NotificationInsertForm {
        kind: DbNotificationKind::Mention,
        recipient_user_id,
        comment_id: Some(comment_id),
        post_id: None,
        message_id: None,
        is_read: false,
        actor_user_id: Some(actor_user_id),
    };

    diesel::insert_into(notifications::table)
        .values(&form)
        .execute(conn)
        .await?;

    Ok(())
}

/// Create a notification for a mention in a post
pub async fn create_post_mention_notification(
    pool: &DbPool,
    recipient_user_id: Uuid,
    post_id: Uuid,
    actor_user_id: Uuid,
) -> Result<(), TinyBoardsError> {
    let conn = &mut get_conn(pool).await?;

    let form = NotificationInsertForm {
        kind: DbNotificationKind::Mention,
        recipient_user_id,
        comment_id: None,
        post_id: Some(post_id),
        message_id: None,
        is_read: false,
        actor_user_id: Some(actor_user_id),
    };

    diesel::insert_into(notifications::table)
        .values(&form)
        .execute(conn)
        .await?;

    Ok(())
}

/// Create a notification for a post reply (comment on post)
pub async fn create_post_reply_notification(
    pool: &DbPool,
    recipient_user_id: Uuid,
    post_id: Uuid,
    comment_id: Uuid,
    actor_user_id: Uuid,
) -> Result<(), TinyBoardsError> {
    let conn = &mut get_conn(pool).await?;

    let form = NotificationInsertForm {
        kind: DbNotificationKind::PostReply,
        recipient_user_id,
        comment_id: Some(comment_id),
        post_id: Some(post_id),
        message_id: None,
        is_read: false,
        actor_user_id: Some(actor_user_id),
    };

    diesel::insert_into(notifications::table)
        .values(&form)
        .execute(conn)
        .await?;

    Ok(())
}

/// Extract @mentions from text
/// Returns list of unique usernames mentioned (without the @ symbol)
pub fn extract_mentions(text: &str) -> Vec<String> {
    use std::collections::HashSet;
    let mut mentions = HashSet::new();

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

        if idx < text.len() && text[idx..].starts_with('@') {
            result.push_str(&text[last_end..idx]);

            let rest = &text[idx+1..];
            let username_end = rest.find(|c: char| !c.is_alphanumeric() && c != '_')
                .unwrap_or(rest.len());

            if username_end > 0 {
                let username = &rest[..username_end];
                result.push_str(&format!("<a href=\"/@{}\">@{}</a>", username, username));
                last_end = idx + 1 + username_end;
            } else {
                result.push('@');
                last_end = idx + 1;
            }
        }
    }

    if last_end < text.len() {
        result.push_str(&text[last_end..]);
    }

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

/// Convert b/boardname references in text to HTML links.
/// Matches `b/<name>` only when it appears at the start of the string or
/// immediately after whitespace, an opening tag boundary (`>`), or common
/// punctuation that would precede a fresh token (`(`, `[`). This keeps URLs
/// like https://example.com/b/foo from being rewritten.
pub fn convert_board_mentions_to_links(text: &str) -> String {
    lazy_static! {
        static ref BOARD_REF_RE: Regex =
            Regex::new(r"(^|[\s>(\[])b/([A-Za-z0-9_][A-Za-z0-9_.-]{0,29})\b")
                .expect("compile board mention regex");
    }

    BOARD_REF_RE
        .replace_all(text, |caps: &regex::Captures| {
            let prefix = &caps[1];
            let name = &caps[2];
            format!(
                "{prefix}<a href=\"/b/{name}\" class=\"board-mention\">b/{name}</a>"
            )
        })
        .to_string()
}

/// Get user IDs for mentioned usernames
pub async fn get_user_ids_for_mentions(
    pool: &DbPool,
    usernames: Vec<String>,
) -> Result<Vec<Uuid>, TinyBoardsError> {
    let conn = &mut get_conn(pool).await?;

    Ok(users::table
        .filter(users::name.eq_any(usernames))
        .filter(users::deleted_at.is_null())
        .select(users::id)
        .load::<Uuid>(conn)
        .await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_mentions() {
        let text = "Hey @alice and @bob, check this out! @charlie.";
        let mut mentions = extract_mentions(text);
        mentions.sort();
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

    #[test]
    fn test_convert_board_mentions_basic() {
        let text = "Check out b/general for updates";
        let result = convert_board_mentions_to_links(text);
        assert!(result.contains("<a href=\"/b/general\" class=\"board-mention\">b/general</a>"));
        assert!(result.contains("for updates"));
    }

    #[test]
    fn test_convert_board_mentions_at_start() {
        let text = "b/rust is great";
        let result = convert_board_mentions_to_links(text);
        assert!(result.starts_with("<a href=\"/b/rust\""));
    }

    #[test]
    fn test_convert_board_mentions_does_not_match_inside_url() {
        let text = "<a href=\"https://example.com/b/foo\">link</a>";
        let result = convert_board_mentions_to_links(text);
        assert_eq!(result, text);
    }

    #[test]
    fn test_convert_board_mentions_after_html_tag() {
        let text = "<p>b/news is open</p>";
        let result = convert_board_mentions_to_links(text);
        assert!(result.contains("<a href=\"/b/news\""));
    }

    #[test]
    fn test_convert_board_mentions_does_not_match_letter_prefix() {
        let text = "tab/foo bar";
        let result = convert_board_mentions_to_links(text);
        assert_eq!(result, text);
    }

    #[test]
    fn test_convert_board_mentions_strips_trailing_punct() {
        let text = "see b/news, it rules";
        let result = convert_board_mentions_to_links(text);
        assert!(result.contains("<a href=\"/b/news\" class=\"board-mention\">b/news</a>,"));
    }
}
