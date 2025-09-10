use serde_json;
use std::collections::HashSet;
use url::Url;
use glob::Pattern;

use crate::TinyBoardsError;

/// Content filtering utilities for enforcing site policies
pub struct ContentFilter;

impl ContentFilter {
    /// Check if a post type is allowed based on site configuration
    pub fn is_post_type_allowed(allowed_types_json: &Option<String>, post_type: &str) -> Result<bool, TinyBoardsError> {
        let allowed_types = match allowed_types_json {
            Some(json_str) => {
                let allowed: Vec<String> = serde_json::from_str(json_str)
                    .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Invalid allowed_post_types configuration"))?;
                allowed
            }
            None => return Ok(true), // If not configured, allow all types
        };

        Ok(allowed_types.contains(&post_type.to_string()))
    }

    /// Check if text contains filtered words using glob patterns
    /// Supports patterns like "badword", "prefix*", "*suffix", "*word*", etc.
    pub fn contains_filtered_words(filtered_words_json: &Option<String>, text: &str) -> Result<bool, TinyBoardsError> {
        let filtered_patterns = match filtered_words_json {
            Some(json_str) => {
                let words: Vec<String> = serde_json::from_str(json_str)
                    .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Invalid filtered_words configuration"))?;
                words
            }
            None => return Ok(false), // If not configured, no words are filtered
        };

        let text_lower = text.to_lowercase();
        
        for pattern_str in filtered_patterns {
            // Create glob pattern (case-insensitive)
            let pattern = Pattern::new(&pattern_str.to_lowercase())
                .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Invalid glob pattern in filtered words"))?;
            
            // Split text into words and check each word
            // Also check the full text for patterns that might span word boundaries
            let words: Vec<&str> = text_lower.split_whitespace().collect();
            
            // Check individual words
            for word in &words {
                // Remove common punctuation from word boundaries for matching
                let clean_word = word.trim_matches(|c: char| c.is_ascii_punctuation());
                if pattern.matches(clean_word) {
                    return Ok(true);
                }
            }
            
            // Check the full text (useful for patterns that might span words)
            if pattern.matches(&text_lower) {
                return Ok(true);
            }
            
            // Check for patterns within the continuous text (no word boundaries)
            // This catches cases where filtered content might be part of larger words
            let no_spaces = text_lower.replace(' ', "");
            if pattern.matches(&no_spaces) {
                return Ok(true);
            }
        }
        
        Ok(false)
    }

    /// Check if a domain is banned
    pub fn is_domain_banned(banned_domains_json: &Option<String>, url: &str) -> Result<bool, TinyBoardsError> {
        let banned_domains = match banned_domains_json {
            Some(json_str) => {
                let domains: Vec<String> = serde_json::from_str(json_str)
                    .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Invalid banned_domains configuration"))?;
                domains.into_iter().collect::<HashSet<_>>()
            }
            None => return Ok(false), // If not configured, no domains are banned
        };

        let parsed_url = Url::parse(url)
            .map_err(|_| TinyBoardsError::from_message(400, "Invalid URL provided"))?;
        
        let domain = parsed_url.domain()
            .ok_or_else(|| TinyBoardsError::from_message(400, "URL has no domain"))?;

        Ok(banned_domains.contains(domain))
    }

    /// Check if an image URL is from an approved host (for image embeds)
    pub fn is_image_host_approved(
        approved_hosts_json: &Option<String>, 
        image_embed_hosts_only: &Option<bool>,
        url: &str
    ) -> Result<bool, TinyBoardsError> {
        // If image host restriction is not enabled, allow all
        if !image_embed_hosts_only.unwrap_or(false) {
            return Ok(true);
        }

        let approved_hosts = match approved_hosts_json {
            Some(json_str) => {
                let hosts: Vec<String> = serde_json::from_str(json_str)
                    .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Invalid approved_image_hosts configuration"))?;
                hosts.into_iter().collect::<HashSet<_>>()
            }
            None => return Ok(false), // If hosts-only mode is enabled but no hosts configured, deny all
        };

        let parsed_url = Url::parse(url)
            .map_err(|_| TinyBoardsError::from_message(400, "Invalid image URL provided"))?;
        
        let domain = parsed_url.domain()
            .ok_or_else(|| TinyBoardsError::from_message(400, "Image URL has no domain"))?;

        Ok(approved_hosts.contains(domain))
    }

    /// Extract URLs from text content to check for domain filtering
    pub fn extract_urls_from_text(text: &str) -> Vec<String> {
        let url_regex = regex::Regex::new(r"https?://[^\s]+").unwrap();
        url_regex.find_iter(text)
            .map(|m| m.as_str().to_string())
            .collect()
    }

    /// Validate content against all applicable filters
    pub fn validate_post_content(
        allowed_post_types: &Option<String>,
        word_filter_enabled: &Option<bool>,
        word_filter_applies_to_posts: &Option<bool>,
        filtered_words: &Option<String>,
        link_filter_enabled: &Option<bool>,
        banned_domains: &Option<String>,
        post_type: &str,
        title: &str,
        body: &Option<String>,
        url: &Option<String>,
    ) -> Result<(), TinyBoardsError> {
        // Check post type
        if !Self::is_post_type_allowed(allowed_post_types, post_type)? {
            return Err(TinyBoardsError::from_message(
                403,
                &format!("Post type '{}' is not allowed on this site", post_type)
            ));
        }

        // Check word filtering if enabled
        if word_filter_enabled.unwrap_or(false) 
            && word_filter_applies_to_posts.unwrap_or(true) {
            
            if Self::contains_filtered_words(filtered_words, title)? {
                return Err(TinyBoardsError::from_message(403, "Post title contains filtered content"));
            }

            if let Some(body_text) = body {
                if Self::contains_filtered_words(filtered_words, body_text)? {
                    return Err(TinyBoardsError::from_message(403, "Post body contains filtered content"));
                }
            }
        }

        // Check link filtering if enabled
        if link_filter_enabled.unwrap_or(false) {
            // Check main URL
            if let Some(url_str) = url {
                if Self::is_domain_banned(banned_domains, url_str)? {
                    return Err(TinyBoardsError::from_message(403, "Link to banned domain is not allowed"));
                }
            }

            // Check URLs in body text
            if let Some(body_text) = body {
                let urls = Self::extract_urls_from_text(body_text);
                for url in urls {
                    if Self::is_domain_banned(banned_domains, &url)? {
                        return Err(TinyBoardsError::from_message(403, "Post contains links to banned domains"));
                    }
                }
            }
        }

        Ok(())
    }

    /// Validate comment content against applicable filters
    pub fn validate_comment_content(
        word_filter_enabled: &Option<bool>,
        word_filter_applies_to_comments: &Option<bool>,
        filtered_words: &Option<String>,
        link_filter_enabled: &Option<bool>,
        banned_domains: &Option<String>,
        body: &str,
    ) -> Result<(), TinyBoardsError> {
        // Check word filtering if enabled
        if word_filter_enabled.unwrap_or(false) 
            && word_filter_applies_to_comments.unwrap_or(true) {
            
            if Self::contains_filtered_words(filtered_words, body)? {
                return Err(TinyBoardsError::from_message(403, "Comment contains filtered content"));
            }
        }

        // Check link filtering if enabled
        if link_filter_enabled.unwrap_or(false) {
            let urls = Self::extract_urls_from_text(body);
            for url in urls {
                if Self::is_domain_banned(banned_domains, &url)? {
                    return Err(TinyBoardsError::from_message(403, "Comment contains links to banned domains"));
                }
            }
        }

        Ok(())
    }

    /// Validate username against applicable filters
    pub fn validate_username(
        word_filter_enabled: &Option<bool>,
        word_filter_applies_to_usernames: &Option<bool>,
        filtered_words: &Option<String>,
        username: &str,
    ) -> Result<(), TinyBoardsError> {
        // Check word filtering if enabled for usernames
        if word_filter_enabled.unwrap_or(false) 
            && word_filter_applies_to_usernames.unwrap_or(false) {
            
            if Self::contains_filtered_words(filtered_words, username)? {
                return Err(TinyBoardsError::from_message(403, "Username contains filtered content"));
            }
        }

        Ok(())
    }
}