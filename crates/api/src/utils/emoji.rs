use regex::Regex;
use std::collections::HashMap;
use tinyboards_db::{
    models::emoji::emoji::Emoji,
    utils::DbPool,
};
use tinyboards_utils::{TinyBoardsError, parser::parse_markdown_opt, utils::custom_body_parsing};

lazy_static::lazy_static! {
    static ref EMOJI_REGEX: Regex = Regex::new(r":([a-zA-Z0-9_-]+):").unwrap();
}

#[derive(Debug, Clone)]
pub struct EmojiReplacement {
    pub shortcode: String,
    pub image_url: String,
    pub alt_text: String,
}

pub struct EmojiParser {
    emoji_map: HashMap<String, EmojiReplacement>,
}

impl EmojiParser {
    pub async fn new(pool: &DbPool, board_id: Option<i32>) -> Result<Self, TinyBoardsError> {
        let emojis = Emoji::list_all_available_emojis(pool, board_id).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to load emojis"))?;

        let mut emoji_map = HashMap::new();

        for emoji in emojis {
            emoji_map.insert(
                emoji.shortcode.clone(),
                EmojiReplacement {
                    shortcode: emoji.shortcode,
                    image_url: emoji.image_url.to_string(),
                    alt_text: emoji.alt_text,
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
                    // Return original if emoji not found
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
                    // Return original if emoji not found
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
            for shortcode in shortcodes {
                // Find emoji by shortcode and increment usage
                if let Ok(emojis) = Emoji::list_all_available_emojis(&pool, None).await {
                    if let Some(emoji) = emojis.iter().find(|e| e.shortcode == shortcode) {
                        if let Err(e) = Emoji::increment_usage(&pool, emoji.id).await {
                            eprintln!("Failed to increment usage for emoji {}: {}", shortcode, e);
                        }
                    }
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
        // If no limit specified, always pass validation

        Ok(())
    }
}

// Utility functions for common emoji operations
pub async fn parse_content_with_emojis(
    content: &str,
    pool: &DbPool,
    board_id: Option<i32>,
    output_format: EmojiOutputFormat,
) -> Result<String, TinyBoardsError> {
    let parser = EmojiParser::new(pool, board_id).await?;

    match output_format {
        EmojiOutputFormat::Html => Ok(parser.parse_emojis_to_html(content)),
        EmojiOutputFormat::Markdown => Ok(parser.parse_emojis_to_markdown(content)),
        EmojiOutputFormat::Plain => Ok(content.to_string()), // Return as-is
    }
}

pub enum EmojiOutputFormat {
    Html,
    Markdown,
    Plain,
}

// Enhanced content processing that combines markdown and emoji parsing
pub async fn process_content_with_emojis(
    content: &str,
    pool: &DbPool,
    board_id: Option<i32>,
    settings: &tinyboards_utils::settings::structs::Settings,
    max_emojis: Option<usize>,
) -> Result<String, TinyBoardsError> {
    let parser = EmojiParser::new(pool, board_id).await?;

    // Validate emoji usage if limit specified
    parser.validate_emoji_usage(content, max_emojis)?;

    // First, parse markdown to HTML
    let markdown_html = parse_markdown_opt(content);
    let processed_html = custom_body_parsing(&markdown_html.unwrap_or_default(), settings);

    // Then parse emojis in the resulting HTML
    let final_html = parser.parse_emojis_to_html(&processed_html);

    // Increment emoji usage in background
    let _usage_task = parser.increment_emoji_usage(&final_html, pool);

    Ok(final_html)
}

// Utility for re-processing existing content with emoji support
pub async fn reprocess_content_with_emojis(
    content_html: &str,
    pool: &DbPool,
    board_id: Option<i32>,
) -> Result<String, TinyBoardsError> {
    let parser = EmojiParser::new(pool, board_id).await?;

    // Simply parse emojis in the existing HTML
    // This is useful for content that was created before emoji support
    let processed_html = parser.parse_emojis_to_html(content_html);

    Ok(processed_html)
}

// Background task to update emoji usage from existing content
pub async fn update_emoji_usage_from_content(
    content: &str,
    pool: &DbPool,
    board_id: Option<i32>,
) -> Result<(), TinyBoardsError> {
    let parser = EmojiParser::new(pool, board_id).await?;

    // Extract and increment usage for emojis found in content
    let shortcodes = parser.extract_emoji_shortcodes(content);

    for shortcode in shortcodes {
        // Find emoji by shortcode and increment usage
        let emojis = Emoji::list_all_available_emojis(pool, board_id).await?;
        if let Some(emoji) = emojis.iter().find(|e| e.shortcode == shortcode) {
            if let Err(e) = Emoji::increment_usage(pool, emoji.id).await {
                eprintln!("Failed to increment usage for emoji {}: {}", shortcode, e);
            }
        }
    }

    Ok(())
}

// Comprehensive reprocessing function for all content
pub async fn reprocess_all_content_with_emojis(
    pool: &DbPool,
    board_id: Option<i32>,
) -> Result<(), TinyBoardsError> {
    use tinyboards_db::models::post::posts::Post;
    use tinyboards_db::models::comment::comments::Comment;

    let parser = EmojiParser::new(pool, board_id).await?;
    let mut processed_posts = 0;
    let mut processed_comments = 0;

    // Process posts
    match board_id {
        Some(board_id) => {
            // Process posts for specific board
            if let Ok(posts) = Post::list_posts_for_board(pool, board_id).await {
                for post in posts {
                    let body_html = &post.body_html;
                    let new_html = parser.parse_emojis_to_html(body_html);
                    if new_html != *body_html {
                        if let Err(e) = Post::update_body_html(pool, post.id, &new_html).await {
                            eprintln!("Failed to update post {} body_html: {}", post.id, e);
                        } else {
                            processed_posts += 1;
                        }
                    }
                }
            }
        }
        None => {
            // Process all posts site-wide
            if let Ok(posts) = Post::list_all_posts(pool).await {
                for post in posts {
                    let body_html = &post.body_html;
                    let new_html = parser.parse_emojis_to_html(body_html);
                    if new_html != *body_html {
                        if let Err(e) = Post::update_body_html(pool, post.id, &new_html).await {
                            eprintln!("Failed to update post {} body_html: {}", post.id, e);
                        } else {
                            processed_posts += 1;
                        }
                    }
                }
            }
        }
    }

    // Process comments
    match board_id {
        Some(board_id) => {
            // Process comments for specific board
            if let Ok(comments) = Comment::list_comments_for_board(pool, board_id).await {
                for comment in comments {
                    let body_html = &comment.body_html;
                    let new_html = parser.parse_emojis_to_html(body_html);
                    if new_html != *body_html {
                        if let Err(e) = Comment::update_body_html(pool, comment.id, &new_html).await {
                            eprintln!("Failed to update comment {} body_html: {}", comment.id, e);
                        } else {
                            processed_comments += 1;
                        }
                    }
                }
            }
        }
        None => {
            // Process all comments site-wide
            if let Ok(comments) = Comment::list_all_comments(pool).await {
                for comment in comments {
                    let body_html = &comment.body_html;
                    let new_html = parser.parse_emojis_to_html(body_html);
                    if new_html != *body_html {
                        if let Err(e) = Comment::update_body_html(pool, comment.id, &new_html).await {
                            eprintln!("Failed to update comment {} body_html: {}", comment.id, e);
                        } else {
                            processed_comments += 1;
                        }
                    }
                }
            }
        }
    }

    println!(
        "Emoji reprocessing completed: {} posts and {} comments updated",
        processed_posts, processed_comments
    );

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
                alt_text: "Test emoji".to_string(),
            },
        );
        emoji_map.insert(
            "smile".to_string(),
            EmojiReplacement {
                shortcode: "smile".to_string(),
                image_url: "https://example.com/smile.png".to_string(),
                alt_text: "Smile emoji".to_string(),
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

        assert_eq!(result, "Hello ![⦂test:](https://example.com/test.png) world!");
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