pub fn parse_markdown_opt(text: &str) -> Option<String> {
    Some(markdown::to_html(text))
}

pub fn parse_markdown(text: &str) -> String {
    markdown::to_html(text)
}