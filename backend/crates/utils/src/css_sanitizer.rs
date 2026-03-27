use once_cell::sync::Lazy;
use regex::Regex;

/// Maximum allowed CSS length in bytes (50 KB for site, 25 KB for board).
pub const MAX_SITE_CSS_BYTES: usize = 50 * 1024;
pub const MAX_BOARD_CSS_BYTES: usize = 25 * 1024;

/// Patterns that are never allowed in custom CSS — these can be used for XSS,
/// data exfiltration, or to hijack the page layout in dangerous ways.
static DANGEROUS_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // Script injection / expression evaluation
        Regex::new(r"(?i)expression\s*\(").unwrap(),
        Regex::new(r"(?i)javascript\s*:").unwrap(),
        Regex::new(r"(?i)vbscript\s*:").unwrap(),
        // Legacy IE behaviors
        Regex::new(r"(?i)behavior\s*:").unwrap(),
        Regex::new(r"(?i)-moz-binding\s*:").unwrap(),
        // Data exfiltration via external resources
        Regex::new(r"(?i)@import").unwrap(),
        Regex::new(r"(?i)@charset").unwrap(),
        Regex::new(r"(?i)@namespace").unwrap(),
        // url() can load external resources — block it entirely
        Regex::new(r"(?i)url\s*\(").unwrap(),
        // Block HTML tags that may have been injected
        Regex::new(r"<\s*/?(?:script|style|link|meta|iframe|object|embed|applet|form|input|svg|math)").unwrap(),
        // Unicode escape sequences used to bypass keyword filters
        Regex::new(r"\\[0-9a-fA-F]{1,6}").unwrap(),
    ]
});

/// CSS properties that could be used to overlay content or break the admin UI.
static BLOCKED_PROPERTY_VALUES: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // Fixed/sticky positioning can overlay nav, modals, etc.
        Regex::new(r"(?i)position\s*:\s*(fixed|sticky)").unwrap(),
        // Very high z-index can break modals and overlays
        Regex::new(r"(?i)z-index\s*:\s*(\d{5,})").unwrap(),
    ]
});

/// Result of CSS sanitization.
#[derive(Debug)]
pub struct CssSanitizeResult {
    pub css: String,
    pub warnings: Vec<String>,
}

/// Sanitize a CSS string for safe injection into the page.
///
/// This validates the CSS and strips anything that could be used for XSS,
/// data exfiltration, or to break the site's UI. Returns the cleaned CSS
/// and any warnings about removed content.
pub fn sanitize_css(input: &str, max_bytes: usize) -> Result<CssSanitizeResult, String> {
    if input.trim().is_empty() {
        return Ok(CssSanitizeResult {
            css: String::new(),
            warnings: vec![],
        });
    }

    if input.len() > max_bytes {
        return Err(format!(
            "CSS exceeds maximum allowed size of {} KB",
            max_bytes / 1024
        ));
    }

    let mut warnings = Vec::new();
    let mut css = input.to_string();

    // Check for dangerous patterns
    for pattern in DANGEROUS_PATTERNS.iter() {
        if pattern.is_match(&css) {
            let matched = pattern
                .find(&css)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            warnings.push(format!("Blocked pattern removed: {}", matched.trim()));
            css = pattern.replace_all(&css, "/* blocked */").to_string();
        }
    }

    // Check for blocked property values
    for pattern in BLOCKED_PROPERTY_VALUES.iter() {
        if pattern.is_match(&css) {
            let matched = pattern
                .find(&css)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            warnings.push(format!("Blocked property value removed: {}", matched.trim()));
            css = pattern.replace_all(&css, "/* blocked */").to_string();
        }
    }

    // Strip any remaining HTML-like content
    let html_tag_re = Regex::new(r"<[^>]*>").unwrap();
    if html_tag_re.is_match(&css) {
        warnings.push("HTML tags removed from CSS".to_string());
        css = html_tag_re.replace_all(&css, "").to_string();
    }

    // Verify balanced braces — unbalanced CSS can break the page
    let open_braces = css.chars().filter(|c| *c == '{').count();
    let close_braces = css.chars().filter(|c| *c == '}').count();
    if open_braces != close_braces {
        return Err(format!(
            "Unbalanced braces: {} opening vs {} closing. Please check your CSS syntax.",
            open_braces, close_braces
        ));
    }

    Ok(CssSanitizeResult { css, warnings })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_css_passes() {
        let css = r#"
            .post-card { background: #f0f0f0; border-radius: 8px; }
            .header { color: #333; font-size: 1.2rem; }
            @media (max-width: 768px) { .sidebar { display: none; } }
        "#;
        let result = sanitize_css(css, MAX_SITE_CSS_BYTES).unwrap();
        assert!(result.warnings.is_empty());
        assert!(result.css.contains(".post-card"));
    }

    #[test]
    fn test_blocks_javascript() {
        let css = "div { background: javascript:alert(1); }";
        let result = sanitize_css(css, MAX_SITE_CSS_BYTES).unwrap();
        assert!(!result.warnings.is_empty());
        assert!(result.css.contains("/* blocked */"));
    }

    #[test]
    fn test_blocks_import() {
        let css = "@import url('evil.css'); .safe { color: red; }";
        let result = sanitize_css(css, MAX_SITE_CSS_BYTES).unwrap();
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn test_blocks_expression() {
        let css = "div { width: expression(document.body.clientWidth); }";
        let result = sanitize_css(css, MAX_SITE_CSS_BYTES).unwrap();
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn test_blocks_url() {
        let css = "div { background: url('https://evil.com/track.png'); }";
        let result = sanitize_css(css, MAX_SITE_CSS_BYTES).unwrap();
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn test_blocks_fixed_position() {
        let css = "div { position: fixed; top: 0; left: 0; }";
        let result = sanitize_css(css, MAX_SITE_CSS_BYTES).unwrap();
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn test_rejects_unbalanced_braces() {
        let css = ".test { color: red; ";
        let result = sanitize_css(css, MAX_SITE_CSS_BYTES);
        assert!(result.is_err());
    }

    #[test]
    fn test_rejects_oversized_css() {
        let css = "a".repeat(MAX_SITE_CSS_BYTES + 1);
        let result = sanitize_css(&css, MAX_SITE_CSS_BYTES);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_css() {
        let result = sanitize_css("", MAX_SITE_CSS_BYTES).unwrap();
        assert!(result.css.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_allows_media_queries() {
        let css = "@media (prefers-color-scheme: dark) { body { background: #1a1a1a; } }";
        let result = sanitize_css(css, MAX_SITE_CSS_BYTES).unwrap();
        assert!(result.warnings.is_empty());
        assert!(result.css.contains("@media"));
    }

    #[test]
    fn test_allows_keyframes() {
        let css = "@keyframes fade { from { opacity: 0; } to { opacity: 1; } }";
        let result = sanitize_css(css, MAX_SITE_CSS_BYTES).unwrap();
        assert!(result.warnings.is_empty());
    }
}
