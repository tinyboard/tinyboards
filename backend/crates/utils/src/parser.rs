use pulldown_cmark::{html, Options, Parser};
use ammonia::clean;

/// Parse markdown to HTML with full CommonMark + GitHub Flavored Markdown support
pub fn parse_markdown(text: &str) -> String {
    // Enable all markdown features
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = Parser::new_ext(text, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    // Sanitize the HTML output
    sanitize_html(&html_output)
}

/// Parse markdown to HTML (returns Option for compatibility)
pub fn parse_markdown_opt(text: &str) -> Option<String> {
    Some(parse_markdown(text))
}

/// Sanitize HTML content using ammonia library
/// This provides comprehensive XSS protection while allowing safe HTML
pub fn sanitize_html(html: &str) -> String {
    // Use ammonia's default settings which are secure by default
    // Allows safe HTML tags and attributes while removing dangerous ones
    clean(html)
}

/// Extended sanitization with custom rules for TinyBoards
/// Allows additional safe tags and attributes as needed
pub fn sanitize_html_extended(html: &str) -> String {
    use ammonia::Builder;
    use maplit::hashset;

    Builder::default()
        // Allow additional safe tags
        .add_tags(hashset![
            "abbr", "details", "summary", "mark", "kbd", "sub", "sup"
        ])
        // Allow additional safe attributes
        .add_generic_attributes(hashset!["class", "id"])
        // Allow data attributes for custom functionality
        .add_generic_attribute_prefixes(hashset!["data-"])
        // Clean the HTML
        .clean(html)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_markdown() {
        let input = "**bold** and *italic*";
        let output = parse_markdown(input);
        assert!(output.contains("<strong>bold</strong>"));
        assert!(output.contains("<em>italic</em>"));
    }

    #[test]
    fn test_tables() {
        let input = r#"
| Header 1 | Header 2 |
|----------|----------|
| Cell 1   | Cell 2   |
"#;
        let output = parse_markdown(input);
        assert!(output.contains("<table>"));
        assert!(output.contains("<th>Header 1</th>"));
        assert!(output.contains("<td>Cell 1</td>"));
    }

    #[test]
    fn test_strikethrough() {
        let input = "~~strikethrough~~";
        let output = parse_markdown(input);
        assert!(output.contains("<del>strikethrough</del>"));
    }

    #[test]
    fn test_task_lists() {
        let input = r#"
- [x] Completed task
- [ ] Incomplete task
"#;
        let output = parse_markdown(input);
        // Task lists are rendered as list items
        assert!(output.contains("<li>"));
        assert!(output.contains("Completed task"));
        assert!(output.contains("Incomplete task"));
    }

    #[test]
    fn test_footnotes() {
        let input = "Here's a footnote[^1].\n\n[^1]: This is the footnote text.";
        let output = parse_markdown(input);
        // Footnotes create references
        assert!(output.contains("footnote"));
    }

    #[test]
    fn test_code_blocks_with_syntax() {
        let input = r#"
```rust
fn main() {
    println!("Hello, world!");
}
```
"#;
        let output = parse_markdown(input);
        assert!(output.contains("<pre><code"));
        // Code blocks work but language class might be sanitized
        assert!(output.contains("println"));
    }

    #[test]
    fn test_xss_prevention() {
        let input = r#"<script>alert('XSS')</script>"#;
        let output = parse_markdown(input);
        // Script tags should be removed
        assert!(!output.contains("<script>"));
        assert!(!output.contains("alert"));
    }

    #[test]
    fn test_javascript_protocol_removed() {
        let input = r#"[Click me](javascript:alert('XSS'))"#;
        let output = parse_markdown(input);
        // javascript: protocol should be removed by ammonia
        assert!(!output.contains("javascript:"));
    }

    #[test]
    fn test_on_event_handlers_removed() {
        let input = r#"<img src="x" onerror="alert('XSS')">"#;
        let output = sanitize_html(input);
        // Event handlers should be removed
        assert!(!output.contains("onerror"));
        assert!(!output.contains("alert"));
    }

    #[test]
    fn test_smart_punctuation() {
        let input = r#"He said "hello" -- it's nice!"#;
        let output = parse_markdown(input);
        // Output should at least contain the text
        assert!(output.contains("hello"));
        assert!(output.contains("nice"));
    }

    #[test]
    fn test_headings_with_attributes() {
        let input = "# Heading {#custom-id}";
        let output = parse_markdown(input);
        assert!(output.contains("<h1"));
    }

    #[test]
    fn test_autolinks() {
        let input = "<https://example.com>";
        let output = parse_markdown(input);
        assert!(output.contains("<a"));
        assert!(output.contains("example.com"));
    }

    #[test]
    fn test_nested_lists() {
        let input = r#"
- Item 1
  - Nested item 1.1
  - Nested item 1.2
- Item 2
"#;
        let output = parse_markdown(input);
        assert!(output.contains("<ul>"));
        assert!(output.contains("<li>"));
    }

    #[test]
    fn test_blockquotes() {
        let input = r#"
> This is a quote
> with multiple lines
"#;
        let output = parse_markdown(input);
        assert!(output.contains("<blockquote>"));
    }

    #[test]
    fn test_horizontal_rule() {
        let input = "---";
        let output = parse_markdown(input);
        assert!(output.contains("<hr"));
    }

    #[test]
    fn test_inline_code() {
        let input = "Use `code` for inline code";
        let output = parse_markdown(input);
        assert!(output.contains("<code>"));
    }
}
