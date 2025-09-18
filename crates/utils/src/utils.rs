use crate::{settings::structs::Settings, IpAddr};
use actix_web::dev::ConnectionInfo;
use itertools::Itertools;
use once_cell::sync::Lazy;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use regex::Regex;

pub fn get_ip(conn_info: &ConnectionInfo) -> IpAddr {
    IpAddr(
        conn_info
            .realip_remote_addr()
            .unwrap_or("127.0.0.1:12345")
            .split(':')
            .next()
            .unwrap_or("127.0.0.1")
            .to_string(),
    )
}

static MENTIONS_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"@(?P<name>[\w.]+)").expect("compile username mention regex"));

static BOARDS_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\+(?P<name>[\w.]+)").expect("compile board mention regex"));

static IMG_TAG_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"<img src=").expect("compile img tag regex"));

static YT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
    r#"(?P<a>https?://|http://)(?P<b>www\.)?(?P<c>youtube\.com/watch\?v=|youtube\.com/user/[a-zA-Z0-9_]+#p/a/@[a-zA-Z0-9_]+/|youtube\.com/v/|youtube\.com/watch\?v=|youtube\.com/embed/|youtu\.be/|youtube\.com/shorts/)(?P<yt_code>[a-zA-Z0-9_]+)(?P<end>[^\s]*)"#)
    .expect("compile yt link regex")
});

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct MentionData {
    pub name: String,
    pub domain: String,
}

impl MentionData {
    pub fn is_local(&self, hostname: &str) -> bool {
        hostname.eq(&self.domain)
    }
    pub fn full_name(&self) -> String {
        format!("@{}@{}", &self.name, &self.domain)
    }
}

pub fn scrape_text_for_mentions(text: &str) -> Vec<MentionData> {
    let mut out: Vec<MentionData> = Vec::new();
    for caps in MENTIONS_REGEX.captures_iter(text) {
        if let Some(name) = caps.name("name").map(|c| c.as_str().to_string()) {
            if let Some(domain) = caps.name("domain").map(|c| c.as_str().to_string()) {
                out.push(MentionData { name, domain });
            }
        }
    }
    out.into_iter().unique().collect()
}

pub fn custom_body_parsing(body: &str, settings: &Settings) -> String {
    let base_url = settings.get_protocol_and_hostname();
    let mut result = IMG_TAG_REGEX
        .replace_all(body, "<img loading='lazy' class='img-expand' src=")
        .to_string();
    let rcopy = result.clone();

    for cap in MENTIONS_REGEX.captures_iter(rcopy.as_str()) {
        let uname = cap["name"].to_string();
        let profile_link = format!("{}/@{}", base_url, uname);
        let profile_ref = format!(
            "<a id=\"mention-{}\" class=\"username-mention\" href='{}'>@{}</a>",
            uname, profile_link, uname
        );
        //let nuxt_ref = format!("<NuxtLink to='/user/{}'>@{}</NuxtLink>", uname, uname);
        result = result.replace(&format!("@{}", uname), &profile_ref);
    }

    let rcopy = result.clone();

    for cap in BOARDS_REGEX.captures_iter(rcopy.as_str()) {
        let board_name = cap["name"].to_string();
        let board_link = format!("{}/b/{}", base_url, board_name);
        let board_ref = format!(
            "<a id=\"ref-board-{}\" class=\"board-reference\" href='{}'>+{}</a>",
            board_name, board_link, board_name
        );
        //let nuxt_ref = format!("<NuxtLink to='/user/{}'>@{}</NuxtLink>", uname, uname);
        result = result.replace(&format!("+{}", board_name), &board_ref);
    }

    let rcopy = result.clone();

    for cap in YT_REGEX.captures_iter(rcopy.as_str()) {
        let yt_code = cap["yt_code"].to_string();
        let yt_tag = format!(
            "<span class='lite-youtube'><lite-youtube videoid='{}'></lite-youtube></span>",
            yt_code
        );

        let mut yt_vec: Vec<&str> = Vec::new();

        if let Some(a) = cap.name("a") {
            yt_vec.push(a.as_str());
        }

        if let Some(b) = cap.name("b") {
            yt_vec.push(b.as_str());
        }

        if let Some(c) = cap.name("c") {
            yt_vec.push(c.as_str());
        }

        if let Some(d) = cap.name("yt_code") {
            yt_vec.push(d.as_str());
        }

        if let Some(e) = cap.name("e") {
            yt_vec.push(e.as_str());
        }

        let yt_link = yt_vec.concat();

        result = result.replace(&yt_link, &yt_tag);
    }
    result
}

pub fn generate_rand_string() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .map(char::from)
        .take(18)
        .collect()
}

pub fn get_file_type(content_type: &str) -> &str {
    get_file_type_extended(content_type)
}

pub fn get_file_type_extended(content_type: &str) -> &str {
    match content_type {
        // Images
        "image/gif" => "gif",
        "image/jpeg" | "image/jpg" => "jpg",
        "image/png" => "png",
        "image/webp" => "webp",
        "image/bmp" => "bmp",
        "image/svg+xml" => "svg",
        // Videos
        "video/mp4" => "mp4",
        "video/webm" => "webm",
        "video/avi" => "avi",
        "video/mov" => "mov",
        "video/mkv" => "mkv",
        "video/flv" => "flv",
        "video/wmv" => "wmv",
        // Audio
        "audio/mp3" => "mp3",
        "audio/wav" => "wav",
        "audio/ogg" => "ogg",
        "audio/flac" => "flac",
        // Documents
        "application/pdf" => "pdf",
        "text/plain" => "txt",
        "application/msword" => "doc",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => "docx",
        // Archives
        "application/zip" => "zip",
        "application/x-rar-compressed" => "rar",
        "application/x-7z-compressed" => "7z",
        _ => "bin",
    }
}

pub fn is_acceptable_file_type(content_type: &str) -> bool {
    let acceptable_types = [
        // Images
        "image/gif",
        "image/jpeg",
        "image/jpg",
        "image/webp",
        "image/png",
        "image/bmp",
        "image/svg+xml",
        // Videos
        "video/mp4",
        "video/webm",
        "video/avi",
        "video/mov",
        "video/mkv",
        // Audio
        "audio/mp3",
        "audio/wav",
        "audio/ogg",
        // Documents
        "application/pdf",
        "text/plain",
    ];

    acceptable_types.contains(&content_type)
}

pub fn is_video_type(content_type: &str) -> bool {
    content_type.starts_with("video/")
}

pub fn is_image_type(content_type: &str) -> bool {
    content_type.starts_with("image/")
}

pub fn is_audio_type(content_type: &str) -> bool {
    content_type.starts_with("audio/")
}

pub fn is_document_type(content_type: &str) -> bool {
    matches!(
        content_type,
        "application/pdf" | "text/plain" | "application/msword" |
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
    )
}

// File size helpers
pub fn format_file_size(bytes: i64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

pub fn get_max_file_size_for_type(content_type: &str) -> i64 {
    // Default sizes in MB, converted to bytes
    let size_mb = if is_image_type(content_type) {
        10 // 10MB for images
    } else if is_video_type(content_type) {
        100 // 100MB for videos
    } else if is_audio_type(content_type) {
        50 // 50MB for audio
    } else if is_document_type(content_type) {
        25 // 25MB for documents
    } else {
        10 // 10MB for other files
    };

    size_mb * 1024 * 1024
}

pub fn validate_file_size(content_type: &str, size: i64) -> Result<(), String> {
    let max_size = get_max_file_size_for_type(content_type);

    if size > max_size {
        Err(format!(
            "File exceeds maximum allowed size of {} for {}",
            format_file_size(max_size),
            content_type
        ))
    } else {
        Ok(())
    }
}

// Secure filename generation
pub fn generate_secure_filename(original_name: Option<String>, content_type: &str) -> String {
    let extension = get_file_type_extended(content_type);
    let timestamp = chrono::Utc::now().timestamp();
    let random = generate_rand_string();

    // Optionally preserve original name (sanitized)
    if let Some(orig) = original_name {
        let sanitized = sanitize_filename(&orig);
        if !sanitized.is_empty() {
            format!("{}_{}_{}.{}", sanitized, timestamp, random, extension)
        } else {
            format!("upload_{}_{}.{}", timestamp, random, extension)
        }
    } else {
        format!("upload_{}_{}.{}", timestamp, random, extension)
    }
}

pub fn sanitize_filename(filename: &str) -> String {
    // Remove extension from original filename for sanitization
    let stem = if let Some(dot_pos) = filename.rfind('.') {
        &filename[..dot_pos]
    } else {
        filename
    };

    stem.chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect::<String>()
        .trim_matches('_')
        .trim_matches('-')
        .to_lowercase()
        .chars()
        .take(50) // Limit length
        .collect()
}

// File validation helpers
pub fn detect_file_type_from_bytes(bytes: &[u8]) -> Option<&'static str> {
    if bytes.len() < 8 {
        return None;
    }

    // Magic number detection
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some("image/jpeg")
    } else if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        Some("image/png")
    } else if bytes.starts_with(&[0x47, 0x49, 0x46, 0x38]) {
        Some("image/gif")
    } else if bytes.starts_with(&[0x52, 0x49, 0x46, 0x46]) && bytes.len() > 11 && &bytes[8..12] == b"WEBP" {
        Some("image/webp")
    } else if bytes.starts_with(&[0x00, 0x00, 0x00]) && bytes.len() > 7 {
        // MP4 files have various ftypXXXX signatures
        if &bytes[4..8] == b"ftyp" {
            Some("video/mp4")
        } else {
            None
        }
    } else if bytes.starts_with(&[0x1A, 0x45, 0xDF, 0xA3]) {
        Some("video/webm") // Also MKV
    } else if bytes.starts_with(&[0x41, 0x56, 0x49, 0x20]) {
        Some("video/avi")
    } else if bytes.starts_with(&[0x25, 0x50, 0x44, 0x46]) {
        Some("application/pdf")
    } else if bytes.starts_with(&[0x50, 0x4B, 0x03, 0x04]) || bytes.starts_with(&[0x50, 0x4B, 0x05, 0x06]) {
        Some("application/zip")
    } else {
        None
    }
}

pub fn validate_file_content(bytes: &[u8], declared_content_type: &str) -> Result<(), String> {
    // Detect actual file type from magic numbers
    if let Some(detected_type) = detect_file_type_from_bytes(bytes) {
        // Allow some flexibility in MIME type matching
        let declared_base = declared_content_type.split('/').next().unwrap_or("");
        let detected_base = detected_type.split('/').next().unwrap_or("");

        if declared_base != detected_base {
            return Err(format!(
                "File content does not match declared type. Expected: {}, Detected: {}",
                declared_content_type, detected_type
            ));
        }
    }

    // Basic size validation
    if bytes.len() < 10 {
        return Err("File appears to be corrupted or too small".to_string());
    }

    Ok(())
}

// Directory management
pub async fn ensure_upload_directories(media_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let directories = [
        media_path,
        &format!("{}/emojis", media_path),
        &format!("{}/avatars", media_path),
        &format!("{}/videos", media_path),
        &format!("{}/audio", media_path),
        &format!("{}/documents", media_path),
        &format!("{}/temp", media_path),
    ];

    for dir in directories {
        tokio::fs::create_dir_all(dir).await?;
    }

    Ok(())
}

pub fn get_file_url(filename: &str, protocol_and_hostname: &str) -> String {
    format!("{}/media/{}", protocol_and_hostname, filename)
}

pub fn get_file_path_for_type(media_path: &str, filename: &str, content_type: &str) -> String {
    let subdir = if is_video_type(content_type) {
        "videos"
    } else if is_audio_type(content_type) {
        "audio"
    } else if is_document_type(content_type) {
        "documents"
    } else if content_type.starts_with("image/") && filename.starts_with("emoji_") {
        "emojis"
    } else if content_type.starts_with("image/") && (filename.contains("avatar") || filename.contains("pfp")) {
        "avatars"
    } else {
        return format!("{}/{}", media_path, filename);
    };

    format!("{}/{}/{}", media_path, subdir, filename)
}

pub fn extract_img_file_name(i_url: &str) -> Option<String> {
    // find last "/" position on the string
    let last_slash_pos = i_url.rfind('/')?;
    // find the file name based on this
    let img_file_name = i_url.get((last_slash_pos + 1)..)?;

    Some(img_file_name.to_string())
}
