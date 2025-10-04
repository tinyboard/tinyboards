use ammonia;

pub fn parse_markdown_opt(text: &str) -> Option<String> {
    Some(markdown::to_html(text))
}

pub fn parse_markdown(text: &str) -> String {
    markdown::to_html(text)
}

/// Sanitize HTML content to allow rich text editor formatting while preventing XSS
/// Using ammonia::clean() with default settings which is safer than custom Builder
pub fn sanitize_html(html: &str) -> String {
    // Use ammonia's default clean() which has sensible defaults
    // It allows: a, abbr, acronym, b, blockquote, br, code, dd, del, dfn, div, dl, dt, em, h1-h6, hr, i, img, ins, kbd, li, ol, p, pre, s, samp, span, strike, strong, sub, sup, u, ul, var
    // And common safe attributes
    ammonia::clean(html)
}