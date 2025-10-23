use regex::Regex;
use once_cell::sync::Lazy;

static SLUG_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[^a-z0-9]+").expect("compile slug regex")
});

static MULTIPLE_HYPHEN_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"-+").expect("compile multiple hyphen regex")
});

/// Generate a URL-friendly slug from a title
///
/// # Arguments
/// * `title` - The title to convert to a slug
/// * `max_length` - Maximum length of the slug (default: 60)
///
/// # Returns
/// A lowercase, URL-friendly slug with only alphanumeric characters and hyphens
///
/// # Example
/// ```
/// use tinyboards_utils::slug::generate_slug;
///
/// let slug = generate_slug("Hello World! This is a test.", Some(60));
/// assert_eq!(slug, "hello-world-this-is-a-test");
/// ```
pub fn generate_slug(title: &str, max_length: Option<usize>) -> String {
    let max_len = max_length.unwrap_or(60);

    // Convert to lowercase
    let mut slug = title.to_lowercase();

    // Replace all non-alphanumeric characters with hyphens
    slug = SLUG_REGEX.replace_all(&slug, "-").to_string();

    // Replace multiple consecutive hyphens with a single hyphen
    slug = MULTIPLE_HYPHEN_REGEX.replace_all(&slug, "-").to_string();

    // Truncate to max length
    if slug.len() > max_len {
        slug = slug.chars().take(max_len).collect();
    }

    // Remove leading and trailing hyphens
    slug = slug.trim_matches('-').to_string();

    // If slug is empty after processing, use a default
    if slug.is_empty() {
        slug = "post".to_string();
    }

    slug
}

/// Ensure a slug is unique within a given context by appending numbers
///
/// This is a helper function that takes a base slug and a callback
/// to check if the slug already exists. If it does, it appends a number
/// and tries again.
///
/// # Arguments
/// * `base_slug` - The base slug to make unique
/// * `check_exists` - A callback that returns true if the slug already exists
///
/// # Returns
/// A unique slug (may have a number appended like "my-slug-2")
///
/// # Example
/// ```
/// use tinyboards_utils::slug::ensure_unique_slug;
///
/// let mut existing_slugs = vec!["my-slug", "my-slug-2"];
/// let unique = ensure_unique_slug("my-slug", |slug| {
///     existing_slugs.contains(&slug)
/// });
/// assert_eq!(unique, "my-slug-3");
/// ```
pub fn ensure_unique_slug<F>(base_slug: &str, check_exists: F) -> String
where
    F: Fn(&str) -> bool,
{
    let mut final_slug = base_slug.to_string();
    let mut counter = 2;

    while check_exists(&final_slug) {
        final_slug = format!("{}-{}", base_slug, counter);
        counter += 1;

        // Safety check to prevent infinite loops
        if counter > 1000 {
            // Append a random number to ensure uniqueness
            use rand::{thread_rng, Rng};
            let random_suffix: u32 = thread_rng().gen();
            final_slug = format!("{}-{}", base_slug, random_suffix);
            break;
        }
    }

    final_slug
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_slug_basic() {
        assert_eq!(generate_slug("Hello World", None), "hello-world");
    }

    #[test]
    fn test_generate_slug_special_chars() {
        assert_eq!(
            generate_slug("Hello, World! How are you?", None),
            "hello-world-how-are-you"
        );
    }

    #[test]
    fn test_generate_slug_multiple_hyphens() {
        assert_eq!(generate_slug("Hello   World", None), "hello-world");
    }

    #[test]
    fn test_generate_slug_truncate() {
        let long_title = "This is a very long title that should be truncated to sixty characters maximum";
        let slug = generate_slug(long_title, Some(30));
        assert!(slug.len() <= 30);
        assert!(!slug.ends_with('-'));
    }

    #[test]
    fn test_generate_slug_empty() {
        assert_eq!(generate_slug("", None), "post");
        assert_eq!(generate_slug("!!!", None), "post");
    }

    #[test]
    fn test_ensure_unique_slug() {
        let existing = vec!["my-slug", "my-slug-2"];
        let unique = ensure_unique_slug("my-slug", |slug| existing.contains(&slug));
        assert_eq!(unique, "my-slug-3");
    }

    #[test]
    fn test_ensure_unique_slug_no_conflict() {
        let existing = vec!["other-slug"];
        let unique = ensure_unique_slug("my-slug", |slug| existing.contains(&slug));
        assert_eq!(unique, "my-slug");
    }
}
