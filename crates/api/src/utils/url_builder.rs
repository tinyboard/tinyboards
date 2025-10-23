use tinyboards_db::{models::site::site::Site, schema::site, utils::DbPool};
use tinyboards_utils::TinyBoardsError;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::sync::{Arc, RwLock};
use once_cell::sync::Lazy;

/// Global cache for boards_enabled setting
static BOARDS_ENABLED_CACHE: Lazy<Arc<RwLock<Option<bool>>>> =
    Lazy::new(|| Arc::new(RwLock::new(None)));

/// URL Builder for generating consistent URLs across the application
///
/// This builder checks the site's `boards_enabled` setting to determine
/// whether to use multi-board mode (/b/:boardSlug/:section/:id/:slug) or
/// single-board mode (/:section/:id/:slug).
pub struct UrlBuilder {
    boards_enabled: bool,
}

impl UrlBuilder {
    /// Create a new URL builder with the given boards_enabled setting
    pub fn new(boards_enabled: bool) -> Self {
        Self { boards_enabled }
    }

    /// Create a URL builder from database pool (queries site settings)
    pub async fn from_pool(pool: &DbPool) -> Result<Self, TinyBoardsError> {
        let boards_enabled = get_boards_enabled(pool).await?;
        Ok(Self { boards_enabled })
    }

    /// Build a URL for a post/thread
    ///
    /// # Arguments
    /// * `post_id` - The post ID
    /// * `slug` - The post slug (URL-friendly title)
    /// * `board_slug` - Optional board slug (board name)
    ///
    /// # Returns
    /// With boards enabled: `/b/{board_slug}/threads/{post_id}/{slug}`
    /// Without boards: `/threads/{post_id}/{slug}`
    pub fn build_thread_url(
        &self,
        post_id: i32,
        slug: &str,
        board_slug: Option<&str>,
    ) -> String {
        if self.boards_enabled {
            if let Some(board) = board_slug {
                format!("/b/{}/threads/{}/{}", board, post_id, slug)
            } else {
                // Fallback if board_slug is missing
                format!("/threads/{}/{}", post_id, slug)
            }
        } else {
            format!("/threads/{}/{}", post_id, slug)
        }
    }

    /// Build a URL for a feed post
    ///
    /// # Arguments
    /// * `post_id` - The post ID
    /// * `slug` - The post slug (URL-friendly title)
    /// * `board_slug` - Optional board slug (board name)
    ///
    /// # Returns
    /// With boards enabled: `/b/{board_slug}/feed/{post_id}/{slug}`
    /// Without boards: `/feed/{post_id}/{slug}`
    pub fn build_feed_url(
        &self,
        post_id: i32,
        slug: &str,
        board_slug: Option<&str>,
    ) -> String {
        if self.boards_enabled {
            if let Some(board) = board_slug {
                format!("/b/{}/feed/{}/{}", board, post_id, slug)
            } else {
                format!("/feed/{}/{}", post_id, slug)
            }
        } else {
            format!("/feed/{}/{}", post_id, slug)
        }
    }

    /// Build a URL for a wiki page
    ///
    /// # Arguments
    /// * `page_slug` - The wiki page slug
    /// * `board_slug` - Optional board slug (board name)
    ///
    /// # Returns
    /// With boards enabled: `/b/{board_slug}/wiki/{page_slug}`
    /// Without boards: `/wiki/{page_slug}`
    pub fn build_wiki_url(
        &self,
        page_slug: &str,
        board_slug: Option<&str>,
    ) -> String {
        if self.boards_enabled {
            if let Some(board) = board_slug {
                format!("/b/{}/wiki/{}", board, page_slug)
            } else {
                format!("/wiki/{}", page_slug)
            }
        } else {
            format!("/wiki/{}", page_slug)
        }
    }

    /// Build a URL for a board section listing
    ///
    /// # Arguments
    /// * `section` - The section name (threads, feed, wiki)
    /// * `board_slug` - Optional board slug (board name)
    ///
    /// # Returns
    /// With boards enabled: `/b/{board_slug}/{section}`
    /// Without boards: `/{section}`
    pub fn build_section_url(
        &self,
        section: &str,
        board_slug: Option<&str>,
    ) -> String {
        if self.boards_enabled {
            if let Some(board) = board_slug {
                format!("/b/{}/{}", board, section)
            } else {
                format!("/{}", section)
            }
        } else {
            format!("/{}", section)
        }
    }

    /// Build a URL for a board homepage
    ///
    /// # Arguments
    /// * `board_slug` - The board slug (board name)
    ///
    /// # Returns
    /// `/b/{board_slug}`
    pub fn build_board_url(&self, board_slug: &str) -> String {
        format!("/b/{}", board_slug)
    }

    /// Get whether boards are enabled
    pub fn boards_enabled(&self) -> bool {
        self.boards_enabled
    }
}

/// Query the database for the boards_enabled setting with caching
pub async fn get_boards_enabled(pool: &DbPool) -> Result<bool, TinyBoardsError> {
    // Check cache first
    {
        let cache = BOARDS_ENABLED_CACHE.read()
            .map_err(|_| TinyBoardsError::from_message(500, "Cache lock error"))?;
        if let Some(enabled) = *cache {
            return Ok(enabled);
        }
    }

    // Query database
    let conn = &mut tinyboards_db::utils::get_conn(pool).await?;

    let site: Site = site::table
        .select(site::all_columns)
        .first(conn)
        .await
        .map_err(|e| TinyBoardsError::from_message(500, &format!("Failed to get site settings: {}", e)))?;

    let boards_enabled = site.boards_enabled;

    // Update cache
    {
        let mut cache = BOARDS_ENABLED_CACHE.write()
            .map_err(|_| TinyBoardsError::from_message(500, "Cache lock error"))?;
        *cache = Some(boards_enabled);
    }

    Ok(boards_enabled)
}

/// Clear the boards_enabled cache (useful after updating site settings)
pub fn clear_boards_enabled_cache() -> Result<(), TinyBoardsError> {
    let mut cache = BOARDS_ENABLED_CACHE.write()
        .map_err(|_| TinyBoardsError::from_message(500, "Cache lock error"))?;
    *cache = None;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_thread_url_with_boards() {
        let builder = UrlBuilder::new(true);
        let url = builder.build_thread_url(123, "hello-world", Some("general"));
        assert_eq!(url, "/b/general/threads/123/hello-world");
    }

    #[test]
    fn test_build_thread_url_without_boards() {
        let builder = UrlBuilder::new(false);
        let url = builder.build_thread_url(123, "hello-world", None);
        assert_eq!(url, "/threads/123/hello-world");
    }

    #[test]
    fn test_build_feed_url_with_boards() {
        let builder = UrlBuilder::new(true);
        let url = builder.build_feed_url(456, "my-post", Some("tech"));
        assert_eq!(url, "/b/tech/feed/456/my-post");
    }

    #[test]
    fn test_build_wiki_url() {
        let builder = UrlBuilder::new(true);
        let url = builder.build_wiki_url("getting-started", Some("docs"));
        assert_eq!(url, "/b/docs/wiki/getting-started");
    }

    #[test]
    fn test_build_section_url() {
        let builder = UrlBuilder::new(true);
        let url = builder.build_section_url("threads", Some("general"));
        assert_eq!(url, "/b/general/threads");
    }

    #[test]
    fn test_build_board_url() {
        let builder = UrlBuilder::new(true);
        let url = builder.build_board_url("general");
        assert_eq!(url, "/b/general");
    }
}
