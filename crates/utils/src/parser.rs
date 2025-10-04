use ammonia::Builder;
use maplit::hashset;

pub fn parse_markdown_opt(text: &str) -> Option<String> {
    Some(markdown::to_html(text))
}

pub fn parse_markdown(text: &str) -> String {
    markdown::to_html(text)
}

/// Sanitize HTML content to allow rich text editor formatting while preventing XSS
pub fn sanitize_html(html: &str) -> String {
    Builder::default()
        // Allow all standard text formatting tags
        .add_tags(hashset![
            "h1", "h2", "h3", "h4", "h5", "h6",
            "p", "br", "hr",
            "strong", "b", "em", "i", "u", "s", "strike", "del",
            "ul", "ol", "li",
            "blockquote", "pre", "code",
            "a", "img",
            "span", "div",
            "mark",
        ])
        // Allow attributes for styling and functionality
        .add_generic_attributes(hashset!["class", "id"])
        .add_tag_attributes("a", hashset!["href", "title", "target", "rel"])
        .add_tag_attributes("img", hashset!["src", "alt", "title", "width", "height", "loading"])
        .add_tag_attributes("span", hashset!["style", "class"])
        .add_tag_attributes("div", hashset!["style", "class"])
        .add_tag_attributes("p", hashset!["style", "class"])
        .add_tag_attributes("h1", hashset!["style", "class"])
        .add_tag_attributes("h2", hashset!["style", "class"])
        .add_tag_attributes("h3", hashset!["style", "class"])
        .add_tag_attributes("h4", hashset!["style", "class"])
        .add_tag_attributes("h5", hashset!["style", "class"])
        .add_tag_attributes("h6", hashset!["style", "class"])
        .add_tag_attributes("mark", hashset!["style", "class", "data-color"])
        .add_tag_attributes("code", hashset!["class"])
        .add_tag_attributes("pre", hashset!["class"])
        // Allow specific style properties for text color, background, and alignment
        .add_allowed_classes("img", hashset!["img-expand"])
        .add_allowed_classes("a", hashset!["username-mention", "board-reference"])
        .add_allowed_classes("span", hashset!["lite-youtube", "mention"])
        // Allow URL schemes
        .url_schemes(hashset!["http", "https", "mailto"])
        .clean(html)
        .to_string()
}