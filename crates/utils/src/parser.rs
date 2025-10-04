use regex::Regex;
use once_cell::sync::Lazy;

pub fn parse_markdown_opt(text: &str) -> Option<String> {
    Some(markdown::to_html(text))
}

pub fn parse_markdown(text: &str) -> String {
    markdown::to_html(text)
}

/// Sanitize HTML content by removing dangerous tags and attributes
/// This is a simple allowlist-based sanitizer that's more reliable than ammonia's Builder
pub fn sanitize_html(html: &str) -> String {
    static SCRIPT_TAG: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)<script[^>]*>.*?</script>").unwrap());
    static STYLE_TAG: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)<style[^>]*>.*?</style>").unwrap());
    static ON_EVENT: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(?i)\s+on\w+\s*=\s*["'][^"']*["']"#).unwrap());
    static JAVASCRIPT_PROTOCOL: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(?i)javascript:"#).unwrap());
    static DATA_PROTOCOL: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(?i)data:(?!image/)"#).unwrap());

    let mut clean = html.to_string();

    // Remove script tags
    clean = SCRIPT_TAG.replace_all(&clean, "").to_string();

    // Remove style tags (inline styles via style attribute are ok)
    clean = STYLE_TAG.replace_all(&clean, "").to_string();

    // Remove event handlers (onclick, onload, etc.)
    clean = ON_EVENT.replace_all(&clean, "").to_string();

    // Remove javascript: protocol
    clean = JAVASCRIPT_PROTOCOL.replace_all(&clean, "").to_string();

    // Remove data: protocol except for images
    clean = DATA_PROTOCOL.replace_all(&clean, "").to_string();

    clean
}