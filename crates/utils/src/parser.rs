use regex::Regex;
use once_cell::sync::Lazy;

pub fn parse_markdown_opt(text: &str) -> Option<String> {
    Some(markdown::to_html(text))
}

pub fn parse_markdown(text: &str) -> String {
    markdown::to_html(text)
}

/// Sanitize HTML content by removing dangerous tags and attributes
/// This is a simple blocklist-based sanitizer that removes XSS vectors
pub fn sanitize_html(html: &str) -> String {
    static SCRIPT_TAG: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?is)<script\b[^>]*>.*?</script>").unwrap());
    static STYLE_TAG: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?is)<style\b[^>]*>.*?</style>").unwrap());
    static ON_EVENT: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(?i)\s+on\w+\s*="#).unwrap());
    static JAVASCRIPT_PROTOCOL: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)javascript:").unwrap());

    let mut clean = html.to_string();

    // Remove script tags (with content)
    clean = SCRIPT_TAG.replace_all(&clean, "").to_string();

    // Remove style tags (inline styles are ok)
    clean = STYLE_TAG.replace_all(&clean, "").to_string();

    // Remove event handler attributes (onclick, onload, etc.)
    clean = ON_EVENT.replace_all(&clean, " ").to_string();

    // Remove javascript: protocol from URLs
    clean = JAVASCRIPT_PROTOCOL.replace_all(&clean, "").to_string();

    clean
}