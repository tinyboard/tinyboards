use regex::Regex;
use std::collections::HashMap;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    enums::DbEmojiScope,
    models::emoji::Emoji,
    schema::emoji,
    utils::{get_conn, DbPool},
};
use tinyboards_utils::{TinyBoardsError, parser::parse_markdown_opt, utils::custom_body_parsing};
use uuid::Uuid;

lazy_static::lazy_static! {
    static ref EMOJI_REGEX: Regex = Regex::new(r":([a-zA-Z0-9_-]+):").unwrap();
}

#[derive(Debug, Clone)]
pub struct EmojiReplacement {
    pub shortcode: String,
    pub image_url: String,
    pub alt_text_display: String,
}

pub struct EmojiParser {
    emoji_map: HashMap<String, EmojiReplacement>,
}

impl EmojiParser {
    pub async fn new(pool: &DbPool, board_id: Option<Uuid>) -> Result<Self, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;

        let mut query = emoji::table
            .filter(emoji::is_active.eq(true))
            .into_boxed();

        if let Some(bid) = board_id {
            query = query.filter(
                emoji::scope.eq(DbEmojiScope::Global)
                    .or(emoji::board_id.eq(bid))
            );
        } else {
            query = query.filter(emoji::scope.eq(DbEmojiScope::Global));
        }

        let emojis: Vec<Emoji> = query
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let mut emoji_map = HashMap::new();

        for e in emojis {
            emoji_map.insert(
                e.shortcode.clone(),
                EmojiReplacement {
                    shortcode: e.shortcode,
                    image_url: e.image_url.to_string(),
                    alt_text_display: e.alt_text,
                },
            );
        }

        Ok(Self { emoji_map })
    }

    pub fn parse_emojis_to_html(&self, text: &str) -> String {
        EMOJI_REGEX
            .replace_all(text, |caps: &regex::Captures| {
                let shortcode = &caps[1];
                if let Some(emoji) = self.emoji_map.get(shortcode) {
                    format!(
                        r#"<img class="emoji" src="{}" alt=":{shortcode}:" title=":{shortcode}:" loading="lazy" />"#,
                        emoji.image_url,
                        shortcode = emoji.shortcode
                    )
                } else {
                    caps[0].to_string()
                }
            })
            .to_string()
    }

    pub fn parse_emojis_to_markdown(&self, text: &str) -> String {
        EMOJI_REGEX
            .replace_all(text, |caps: &regex::Captures| {
                let shortcode = &caps[1];
                if let Some(emoji) = self.emoji_map.get(shortcode) {
                    format!("![:{shortcode}:]({url})", shortcode = emoji.shortcode, url = emoji.image_url)
                } else {
                    caps[0].to_string()
                }
            })
            .to_string()
    }

    pub fn extract_emoji_shortcodes(&self, text: &str) -> Vec<String> {
        EMOJI_REGEX
            .captures_iter(text)
            .map(|caps| caps[1].to_string())
            .collect()
    }

    pub fn increment_emoji_usage(&self, text: &str, pool: &DbPool) -> tokio::task::JoinHandle<()> {
        let shortcodes = self.extract_emoji_shortcodes(text);
        let pool = pool.clone();

        tokio::spawn(async move {
            if let Ok(conn) = &mut get_conn(&pool).await {
                for shortcode in shortcodes {
                    let _ = diesel::update(
                        emoji::table.filter(emoji::shortcode.eq(&shortcode))
                    )
                    .set(emoji::usage_count.eq(emoji::usage_count + 1))
                    .execute(conn)
                    .await;
                }
            }
        })
    }

    pub fn validate_emoji_usage(&self, text: &str, max_emojis_per_content: Option<usize>) -> Result<(), TinyBoardsError> {
        if let Some(max_emojis) = max_emojis_per_content {
            let emoji_count = self.extract_emoji_shortcodes(text).len();

            if emoji_count > max_emojis {
                return Err(TinyBoardsError::from_message(
                    400,
                    &format!("Too many emojis in content. Maximum allowed: {}, found: {}",
                             max_emojis, emoji_count)
                ));
            }
        }

        Ok(())
    }
}

// Utility functions for common emoji operations
pub async fn parse_content_with_emojis(
    content: &str,
    pool: &DbPool,
    board_id: Option<Uuid>,
    output_format: EmojiOutputFormat,
) -> Result<String, TinyBoardsError> {
    let parser = EmojiParser::new(pool, board_id).await?;

    match output_format {
        EmojiOutputFormat::Html => Ok(parser.parse_emojis_to_html(content)),
        EmojiOutputFormat::Markdown => Ok(parser.parse_emojis_to_markdown(content)),
        EmojiOutputFormat::Plain => Ok(content.to_string()),
    }
}

pub enum EmojiOutputFormat {
    Html,
    Markdown,
    Plain,
}

/// Detect if content is primarily HTML or markdown
fn is_html_content(content: &str) -> bool {
    let html_indicators = [
        "<p>", "<div>", "<span>", "<br>", "<strong>", "<em>",
        "<h1>", "<h2>", "<h3>", "<h4>", "<h5>", "<h6>",
        "<ul>", "<ol>", "<li>", "<blockquote>",
        "<pre>", "<code>", "<img", "<a href=",
        "<table>", "<tr>", "<td>", "<th>",
    ];

    html_indicators.iter().any(|tag| content.contains(tag))
}

/// Enhanced content processing that handles markdown, HTML, or mixed content
pub async fn process_content_with_emojis(
    content: &str,
    pool: &DbPool,
    board_id: Option<Uuid>,
    settings: &tinyboards_utils::settings::structs::Settings,
    max_emojis: Option<usize>,
) -> Result<String, TinyBoardsError> {
    use tinyboards_utils::parser::sanitize_html;
    use crate::helpers::notifications::convert_mentions_to_links;

    let parser = EmojiParser::new(pool, board_id).await?;

    let html = if is_html_content(content) {
        sanitize_html(content)
    } else {
        let markdown_html = parse_markdown_opt(content);
        markdown_html.unwrap_or_default()
    };

    parser.validate_emoji_usage(&html, max_emojis)?;

    let html_with_emojis = parser.parse_emojis_to_html(&html);
    let processed_html = custom_body_parsing(&html_with_emojis, settings);
    let with_mentions = convert_mentions_to_links(&processed_html);
    let final_html = sanitize_html(&with_mentions);

    let _usage_task = parser.increment_emoji_usage(&final_html, pool);

    Ok(final_html)
}

/// Utility for re-processing existing content with emoji support
pub async fn reprocess_content_with_emojis(
    content_html: &str,
    pool: &DbPool,
    board_id: Option<Uuid>,
) -> Result<String, TinyBoardsError> {
    let parser = EmojiParser::new(pool, board_id).await?;
    let processed_html = parser.parse_emojis_to_html(content_html);
    Ok(processed_html)
}

/// Background task to update emoji usage from existing content
pub async fn update_emoji_usage_from_content(
    content: &str,
    pool: &DbPool,
    board_id: Option<Uuid>,
) -> Result<(), TinyBoardsError> {
    let parser = EmojiParser::new(pool, board_id).await?;
    let shortcodes = parser.extract_emoji_shortcodes(content);

    if let Ok(conn) = &mut get_conn(pool).await {
        for shortcode in shortcodes {
            let _ = diesel::update(
                emoji::table.filter(emoji::shortcode.eq(&shortcode))
            )
            .set(emoji::usage_count.eq(emoji::usage_count + 1))
            .execute(conn)
            .await;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_parser() -> EmojiParser {
        let mut emoji_map = HashMap::new();
        emoji_map.insert(
            "test".to_string(),
            EmojiReplacement {
                shortcode: "test".to_string(),
                image_url: "https://example.com/test.png".to_string(),
                alt_text_display: "Test emoji".to_string(),
            },
        );
        emoji_map.insert(
            "smile".to_string(),
            EmojiReplacement {
                shortcode: "smile".to_string(),
                image_url: "https://example.com/smile.png".to_string(),
                alt_text_display: "Smile emoji".to_string(),
            },
        );

        EmojiParser { emoji_map }
    }

    #[test]
    fn test_parse_emojis_to_html() {
        let parser = create_test_parser();
        let input = "Hello :test: world :smile:!";
        let result = parser.parse_emojis_to_html(input);

        assert!(result.contains(r#"<img class="emoji" src="https://example.com/test.png""#));
        assert!(result.contains(r#"alt=":test:""#));
        assert!(result.contains(r#"<img class="emoji" src="https://example.com/smile.png""#));
    }

    #[test]
    fn test_parse_emojis_to_markdown() {
        let parser = create_test_parser();
        let input = "Hello :test: world!";
        let result = parser.parse_emojis_to_markdown(input);

        assert!(result.contains("!["));
        assert!(result.contains("https://example.com/test.png"));
    }

    #[test]
    fn test_extract_emoji_shortcodes() {
        let parser = create_test_parser();
        let input = "Hello :test: world :smile: and :unknown:!";
        let shortcodes = parser.extract_emoji_shortcodes(input);

        assert_eq!(shortcodes, vec!["test", "smile", "unknown"]);
    }

    #[test]
    fn test_unknown_emoji_unchanged() {
        let parser = create_test_parser();
        let input = "Hello :unknown: world!";
        let result = parser.parse_emojis_to_html(input);

        assert_eq!(result, "Hello :unknown: world!");
    }
}
