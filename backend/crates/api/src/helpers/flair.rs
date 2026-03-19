use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    enums::DbFlairType,
    schema::{emoji, flair_templates},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

/// Validate flair template input
pub fn validate_flair_template_input(
    text_display: &str,
    _is_editable: bool,
    emoji_ids: &Option<Vec<i32>>,
    max_emoji_count: i32,
) -> Result<()> {
    // Validate text length
    if text_display.is_empty() {
        return Err(TinyBoardsError::from_message(400, "Flair text cannot be empty").into());
    }

    if text_display.len() > 150 {
        return Err(TinyBoardsError::from_message(
            400,
            "Flair text cannot exceed 150 characters",
        )
        .into());
    }

    // Validate max_emoji_count
    if max_emoji_count < 0 || max_emoji_count > 30 {
        return Err(TinyBoardsError::from_message(
            400,
            "max_emoji_count must be between 0 and 30",
        )
        .into());
    }

    // Validate emoji count
    if let Some(ref emoji_ids) = emoji_ids {
        if emoji_ids.len() > max_emoji_count as usize {
            return Err(TinyBoardsError::from_message(
                400,
                &format!("Too many emojis specified (max: {})", max_emoji_count),
            )
            .into());
        }

        // Check for duplicates
        let mut seen = std::collections::HashSet::new();
        for &id in emoji_ids {
            if !seen.insert(id) {
                return Err(TinyBoardsError::from_message(
                    400,
                    "Duplicate emoji IDs are not allowed",
                )
                .into());
            }
        }
    }

    Ok(())
}

/// Validate that emoji IDs exist and are accessible
pub async fn validate_emoji_ids(pool: &DbPool, emoji_ids: &[Uuid]) -> Result<()> {
    let conn = &mut get_conn(pool).await
        .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Database connection error"))?;

    for &emoji_id in emoji_ids {
        let result: Option<(Uuid, bool)> = emoji::table
            .find(emoji_id)
            .select((emoji::id, emoji::is_active))
            .first(conn)
            .await
            .optional()
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        match result {
            Some((_id, is_active)) => {
                if !is_active {
                    return Err(TinyBoardsError::from_message(
                        400,
                        &format!("Emoji {} is not active", emoji_id),
                    )
                    .into());
                }
            }
            None => {
                return Err(TinyBoardsError::from_message(
                    404,
                    &format!("Emoji {} not found", emoji_id),
                )
                .into());
            }
        }
    }

    Ok(())
}

/// Check flair quotas for a board
pub async fn check_flair_quota(
    pool: &DbPool,
    board_id: Uuid,
    flair_type: DbFlairType,
) -> Result<()> {
    use diesel::dsl::count;

    const BOARD_FLAIR_LIMIT: i64 = 50;

    let conn = &mut get_conn(pool).await.map_err(|e| {
        TinyBoardsError::from_error_message(e, 500, "Database connection error")
    })?;

    let existing_count: i64 = flair_templates::table
        .filter(flair_templates::board_id.eq(board_id))
        .filter(flair_templates::flair_type.eq(flair_type))
        .filter(flair_templates::is_active.eq(true))
        .select(count(flair_templates::id))
        .first(conn)
        .await
        .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Database query error"))?;

    if existing_count >= BOARD_FLAIR_LIMIT {
        return Err(TinyBoardsError::from_message(
            400,
            &format!("Flair template limit reached ({}/{})", existing_count, BOARD_FLAIR_LIMIT),
        )
        .into());
    }

    Ok(())
}

/// Generate CSS class name from flair text
pub fn generate_css_class(text_display: &str) -> String {
    let class = text_display
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c.is_whitespace() {
                '-'
            } else {
                '_'
            }
        })
        .collect::<String>();

    let mut result = format!("flair-{}", class);
    if result.len() > 64 {
        result.truncate(64);
    }

    result
}

/// Generate inline CSS from flair style config
pub fn generate_flair_css(
    background_color: Option<&str>,
    text_color: Option<&str>,
    style_config: Option<&str>,
) -> String {
    let mut css_parts = Vec::new();

    if let Some(bg) = background_color {
        css_parts.push(format!("background-color: {}", bg));
    }

    if let Some(tc) = text_color {
        css_parts.push(format!("color: {}", tc));
    }

    if let Some(config_str) = style_config {
        if let Ok(config) = serde_json::from_str::<serde_json::Value>(config_str) {
            if let Some(border_color) = config.get("border_color").and_then(|v| v.as_str()) {
                css_parts.push(format!("border-color: {}", border_color));
            }
            if let Some(border_width) = config.get("border_width").and_then(|v| v.as_i64()) {
                css_parts.push(format!("border-width: {}px", border_width));
            }
            if let Some(border_radius) = config.get("border_radius").and_then(|v| v.as_i64()) {
                css_parts.push(format!("border-radius: {}px", border_radius));
            }
            if let Some(font_weight) = config.get("font_weight").and_then(|v| v.as_str()) {
                css_parts.push(format!("font-weight: {}", font_weight));
            }
            if let Some(font_size) = config.get("font_size").and_then(|v| v.as_str()) {
                css_parts.push(format!("font-size: {}", font_size));
            }
            if let Some(padding) = config.get("padding").and_then(|v| v.as_str()) {
                css_parts.push(format!("padding: {}", padding));
            }
            if let Some(margin) = config.get("margin").and_then(|v| v.as_str()) {
                css_parts.push(format!("margin: {}", margin));
            }
            if let Some(custom_css) = config.get("custom_css").and_then(|v| v.as_str()) {
                css_parts.push(custom_css.to_string());
            }
        }
    }

    css_parts.join("; ")
}

/// Validate color hex code
pub fn validate_color(color: &str) -> Result<()> {
    if !color.starts_with('#') {
        return Err(TinyBoardsError::from_message(400, "Color must start with #").into());
    }

    let hex = &color[1..];
    if hex.len() != 3 && hex.len() != 6 && hex.len() != 8 {
        return Err(TinyBoardsError::from_message(
            400,
            "Color must be in #RGB, #RRGGBB, or #RRGGBBAA format",
        )
        .into());
    }

    if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(TinyBoardsError::from_message(
            400,
            "Color contains invalid hex characters",
        )
        .into());
    }

    Ok(())
}

/// Sanitize CSS class name
pub fn sanitize_css_class(class: &str) -> Result<String> {
    if class.is_empty() {
        return Err(TinyBoardsError::from_message(400, "CSS class cannot be empty").into());
    }

    if class.len() > 64 {
        return Err(TinyBoardsError::from_message(
            400,
            "CSS class cannot exceed 64 characters",
        )
        .into());
    }

    if !class.chars().next().unwrap().is_alphabetic() {
        return Err(TinyBoardsError::from_message(
            400,
            "CSS class must start with a letter",
        )
        .into());
    }

    if !class
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(TinyBoardsError::from_message(
            400,
            "CSS class can only contain letters, numbers, hyphens, and underscores",
        )
        .into());
    }

    Ok(class.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_flair_template_input() {
        assert!(validate_flair_template_input("Test", false, &None, 5).is_ok());
        assert!(validate_flair_template_input("", false, &None, 5).is_err());
        let long_text = "a".repeat(151);
        assert!(validate_flair_template_input(&long_text, false, &None, 5).is_err());
    }

    #[test]
    fn test_generate_css_class() {
        assert_eq!(generate_css_class("Bug Fix"), "flair-bug-fix");
        assert_eq!(generate_css_class("Feature!"), "flair-feature_");
        assert_eq!(generate_css_class("Test 123"), "flair-test-123");
    }

    #[test]
    fn test_validate_color() {
        assert!(validate_color("#FFF").is_ok());
        assert!(validate_color("#FFFFFF").is_ok());
        assert!(validate_color("#FFFFFFFF").is_ok());
        assert!(validate_color("FFF").is_err());
        assert!(validate_color("#GGG").is_err());
        assert!(validate_color("#FF").is_err());
    }

    #[test]
    fn test_sanitize_css_class() {
        assert!(sanitize_css_class("valid-class").is_ok());
        assert!(sanitize_css_class("valid_class_123").is_ok());
        assert!(sanitize_css_class("123invalid").is_err());
        assert!(sanitize_css_class("invalid class").is_err());
        assert!(sanitize_css_class("").is_err());
    }
}
