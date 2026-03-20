use crate::errors::ValidationError;
use regex::Regex;
use sha2::{Digest, Sha256};
use unicode_segmentation::UnicodeSegmentation;
use url::Url;
use image::GenericImageView;

/// Validation limits and constants
pub struct Limits;

impl Limits {
    // Content limits
    pub const POST_MIN: usize = 10;
    pub const POST_MAX: usize = 30_000;
    pub const QUESTION_MIN: usize = 10;
    pub const QUESTION_MAX: usize = 15_000;
    pub const ANSWER_MIN: usize = 10;
    pub const ANSWER_MAX: usize = 30_000;
    pub const REFRACT_MIN: usize = 10;
    pub const REFRACT_MAX: usize = 1_500;
    
    // Comment limits by depth
    pub const COMMENT_DEPTH_0_MAX: usize = 1_500;
    pub const COMMENT_DEPTH_1_MAX: usize = 1_000;
    pub const COMMENT_DEPTH_2_MAX: usize = 500;
    pub const COMMENT_DEPTH_3_MAX: usize = 250;
    
    // Title limits
    pub const TITLE_MIN: usize = 15;
    pub const TITLE_MAX: usize = 300;
    
    // Username limits
    pub const USERNAME_MIN: usize = 3;
    pub const USERNAME_MAX: usize = 30;
    
    // Bio limits
    pub const BIO_MAX: usize = 1_000;
    
    // Tag limits
    pub const TAG_MIN_COUNT: usize = 1;
    pub const TAG_MAX_COUNT: usize = 5;
    pub const TAG_NAME_MAX: usize = 35;
    
    // Media limits
    pub const IMAGE_MAX_COUNT: usize = 5;
    pub const VIDEO_MAX_COUNT: usize = 2;
    pub const IMAGE_MAX_SIZE_MB: usize = 10;
    pub const VIDEO_MAX_SIZE_MB: usize = 100;
    pub const IMAGE_MAX_DIMENSION: u32 = 10_000;
    pub const VIDEO_MAX_MINUTES: usize = 10;
    
    // Spam detection
    pub const MAX_UPPERCASE_RATIO: f32 = 0.8;
    pub const MAX_EMOJI_RATIO: f32 = 0.3;
    pub const MAX_LINKS_IN_SHORT_CONTENT: usize = 5;
    pub const SHORT_CONTENT_THRESHOLD: usize = 500;

    pub const GIF_MAX_SIZE_MB: usize = 2;
}

/// Reserved usernames that cannot be registered
pub const RESERVED_USERNAMES: &[&str] = &[
    "admin", "administrator", "mod", "moderator", "support",
    "help", "api", "root", "system", "official", "staff",
    "about", "terms", "privacy", "contact", "settings",
    "login", "signup", "logout", "register", "auth",
    "apple", "microsoft", "nvidia", 
];

/// Blocked malicious domains
pub const BLOCKED_DOMAINS: &[&str] = &[
    // Add known malicious domains here
    "malware.com",
    "phishing.com",
];

pub const ALLOWED_IMAGE_TYPES: &[&str] = &[
    "image/jpeg",
    "image/png",
    "image/webp",
    "image/gif",
];

/// Allowed video MIME types
pub const ALLOWED_VIDEO_TYPES: &[&str] = &[
    "video/mp4",
    "video/webm",
    "video/quicktime",
];


/// Content validator
pub struct ContentValidator;

impl ContentValidator {
    /// Validate post content
    pub fn validate_post(content: &str) -> Result<(), ValidationError> {
        Self::validate_length(content, Limits::POST_MIN, Limits::POST_MAX)?;
        Self::check_spam_patterns(content)?;
        Self::validate_urls_in_content(content)?;
        Ok(())
    }

    /// Validate question content
    pub fn validate_question(content: &str, title: &str) -> Result<(), ValidationError> {
        Self::validate_length(content, Limits::QUESTION_MIN, Limits::QUESTION_MAX)?;
        Self::validate_title(title)?;
        Self::check_spam_patterns(content)?;
        Self::validate_urls_in_content(content)?;
        Ok(())
    }

    /// Validate answer content
    pub fn validate_answer(content: &str) -> Result<(), ValidationError> {
        Self::validate_length(content, Limits::ANSWER_MIN, Limits::ANSWER_MAX)?;
        Self::check_spam_patterns(content)?;
        Self::validate_urls_in_content(content)?;
        Ok(())
    }

    /// Validate refract content
    pub fn validate_refract(content: &str) -> Result<(), ValidationError> {
        Self::validate_length(content, Limits::REFRACT_MIN, Limits::REFRACT_MAX)?;
        Self::check_spam_patterns(content)?;
        Ok(())
    }

    /// Validate comment content based on depth
    pub fn validate_comment(content: &str, depth: i32) -> Result<(), ValidationError> {
        let max_length = match depth {
            0 => Limits::COMMENT_DEPTH_0_MAX,
            1 => Limits::COMMENT_DEPTH_1_MAX,
            2 => Limits::COMMENT_DEPTH_2_MAX,
            3 => Limits::COMMENT_DEPTH_3_MAX,
            _ => return Err(ValidationError::MissingField("Invalid depth".to_string())),
        };

        Self::validate_length(content, 1, max_length)?;
        Ok(())
    }

    /// Validate title
    pub fn validate_title(title: &str) -> Result<(), ValidationError> {
        let len = title.trim().len();
        
        if len < Limits::TITLE_MIN {
            return Err(ValidationError::TitleTooShort {
                min: Limits::TITLE_MIN,
                actual: len,
            });
        }

        if len > Limits::TITLE_MAX {
            return Err(ValidationError::TitleTooLong {
                max: Limits::TITLE_MAX,
                actual: len,
            });
        }

        Ok(())
    }

    /// Validate content length
    fn validate_length(content: &str, min: usize, max: usize) -> Result<(), ValidationError> {
        let len = content.trim().len();

        if len < min {
            return Err(ValidationError::ContentTooShort {
                min,
                actual: len,
            });
        }

        if len > max {
            return Err(ValidationError::ContentTooLong {
                max,
                actual: len,
            });
        }

        Ok(())
    }

    /// Check for spam patterns
    fn check_spam_patterns(content: &str) -> Result<(), ValidationError> {
        // Check excessive uppercase
        if content.len() > 50 {
            let uppercase_count = content.chars().filter(|c| c.is_uppercase()).count();
            let uppercase_ratio = uppercase_count as f32 / content.len() as f32;

            if uppercase_ratio > Limits::MAX_UPPERCASE_RATIO {
                return Err(ValidationError::ExcessiveCaps);
            }
        }

        // Check excessive emoji
        let graphemes: Vec<&str> = content.graphemes(true).collect();
        let emoji_count = graphemes.iter().filter(|g| is_emoji(g)).count();
        let emoji_ratio = emoji_count as f32 / graphemes.len() as f32;

        if emoji_ratio > Limits::MAX_EMOJI_RATIO {
            return Err(ValidationError::ExcessiveEmoji);
        }

        // Check excessive repetition
        if has_excessive_repetition(content) {
            return Err(ValidationError::ExcessiveRepetition);
        }

        // Check too many links in short content
        let urls = extract_urls(content);
        if urls.len() > Limits::MAX_LINKS_IN_SHORT_CONTENT 
            && content.len() < Limits::SHORT_CONTENT_THRESHOLD {
            return Err(ValidationError::TooManyLinks {
                max: Limits::MAX_LINKS_IN_SHORT_CONTENT,
            });
        }

        Ok(())
    }

    /// Validate URLs in content
    fn validate_urls_in_content(content: &str) -> Result<(), ValidationError> {
        let urls = extract_urls(content);

        for url_str in urls {
            // Parse URL
            let url = Url::parse(&url_str)
                .map_err(|_| ValidationError::InvalidUrl(url_str.clone()))?;

            // Check for blocked domains
            if let Some(domain) = url.domain() {
                for blocked in BLOCKED_DOMAINS {
                    if domain.contains(*blocked) {
                        return Err(ValidationError::BlockedDomain(domain.to_string()));
                    }
                }
            }
        }

        Ok(())
    }
}

/// Username validator
pub struct UsernameValidator;

impl UsernameValidator {
    /// Validate username
    pub fn validate(username: &str) -> Result<(), ValidationError> {
        let len = username.len();

        // Check length
        if len < Limits::USERNAME_MIN {
            return Err(ValidationError::UsernameTooShort {
                min: Limits::USERNAME_MIN,
            });
        }

        if len > Limits::USERNAME_MAX {
            return Err(ValidationError::UsernameTooLong {
                max: Limits::USERNAME_MAX,
            });
        }

        // Check for consecutive dots
        if username.contains("..") {
            return Err(ValidationError::UsernameConsecutiveDots);
        }

        // Check format: alphanumeric start/end, can contain _ and .
        let re = Regex::new(r"^[a-zA-Z0-9][a-zA-Z0-9_.]*[a-zA-Z0-9]$").expect("USERNAME_REGEX compilation failed");
        if !re.is_match(username) {
            return Err(ValidationError::InvalidUsernameFormat);
        }

        // Check reserved usernames
        if RESERVED_USERNAMES.contains(&username.to_lowercase().as_str()) {
            return Err(ValidationError::UsernameReserved(username.to_string()));
        }

        Ok(())
    }
}

/// Bio validator
pub struct BioValidator;

impl BioValidator {
    pub fn validate(bio: &str) -> Result<(), ValidationError> {
        if bio.len() > Limits::BIO_MAX {
            return Err(ValidationError::BioTooLong {
                max: Limits::BIO_MAX,
            });
        }
        Ok(())
    }
}

/// Tag validator
pub struct TagValidator;

impl TagValidator {
    /// Validate tag count
    pub fn validate_count(count: usize) -> Result<(), ValidationError> {
        if count < Limits::TAG_MIN_COUNT {
            return Err(ValidationError::NotEnoughTags {
                min: Limits::TAG_MIN_COUNT,
                actual: count,
            });
        }

        if count > Limits::TAG_MAX_COUNT {
            return Err(ValidationError::TooManyTags {
                max: Limits::TAG_MAX_COUNT,
                actual: count,
            });
        }

        Ok(())
    }

    /// Validate tag name format
    pub fn validate_name(name: &str) -> Result<String, ValidationError> {
        // Convert to lowercase
        let tag = name.to_lowercase();

        // Check length
        if tag.is_empty() || tag.len() > Limits::TAG_NAME_MAX {
            return Err(ValidationError::InvalidTagFormat(
                format!("Tag must be 1-{} characters", Limits::TAG_NAME_MAX),
            ));
        }

        // Check format: lowercase letters, numbers, +, #, ., -, _
        let re = Regex::new(r"^[a-z0-9+#.\-_]+$").expect("TAG_NAME_REGEX compilation failed");
        if !re.is_match(&tag) {
            return Err(ValidationError::InvalidTagFormat(
                "Tag can only contain: a-z, 0-9, +, #, ., -, _".to_string(),
            ));
        }

        Ok(tag)
    }

    /// Create slug from tag name
    pub fn create_slug(name: &str) -> String {
        name.to_lowercase()
    }
}

/// Media validator
pub struct MediaValidator;

impl Limits {

}

impl MediaValidator {
    /// Validate image
    pub fn validate_image(data: &[u8]) -> Result<(), ValidationError> {
        // Check file size
        let size_mb = data.len() / (1024 * 1024);
        if size_mb > Limits::IMAGE_MAX_SIZE_MB {
            return Err(ValidationError::FileSizeTooLarge {
                max_mb: Limits::IMAGE_MAX_SIZE_MB,
                actual_mb: size_mb,
            });
        }

        // Check if valid image
        let img = image::load_from_memory(data)
            .map_err(|_| ValidationError::InvalidImageFormat)?;

        // Check dimensions
        let (width, height) = img.dimensions();
        if width > Limits::IMAGE_MAX_DIMENSION || height > Limits::IMAGE_MAX_DIMENSION {
            return Err(ValidationError::ImageDimensionsTooLarge {
                max: Limits::IMAGE_MAX_DIMENSION,
                width,
                height,
            });
        }

        Ok(())
    }

    /// Validate image count
    pub fn validate_image_count(count: usize) -> Result<(), ValidationError> {
        if count > Limits::IMAGE_MAX_COUNT {
            return Err(ValidationError::TooManyImages {
                max: Limits::IMAGE_MAX_COUNT,
                actual: count,
            });
        }
        Ok(())
    }

    /// Validate video count
    pub fn validate_video_count(count: usize) -> Result<(), ValidationError> {
        if count > Limits::VIDEO_MAX_COUNT {
            return Err(ValidationError::TooManyVideos {
                max: Limits::VIDEO_MAX_COUNT,
                actual: count,
            });
        }
        Ok(())
    }

    /// Validate video file size
    pub fn validate_video_size(data: &[u8]) -> Result<(), ValidationError> {
        let size_mb = data.len() / (1024 * 1024);
        if size_mb > Limits::VIDEO_MAX_SIZE_MB {
            return Err(ValidationError::FileSizeTooLarge {
                max_mb: Limits::VIDEO_MAX_SIZE_MB,
                actual_mb: size_mb,
            });
        }
        Ok(())
    }

    pub fn validate_image_with_type(data: &[u8], mime_type: &str) -> Result<(), ValidationError> {
        // Check MIME type
        if !ALLOWED_IMAGE_TYPES.contains(&mime_type) {
            return Err(ValidationError::InvalidImageFormat);
        }

        // Special handling for GIF size limit
        if mime_type == "image/gif" {
            let size_mb = data.len() / (1024 * 1024);
            if size_mb > Limits::GIF_MAX_SIZE_MB {
                return Err(ValidationError::FileSizeTooLarge {
                    max_mb: Limits::GIF_MAX_SIZE_MB,
                    actual_mb: size_mb,
                });
            }
        }

        // Use existing validation
        Self::validate_image(data)
    }

    /// Validate video with MIME type check
    pub fn validate_video_with_type(data: &[u8], mime_type: &str) -> Result<(), ValidationError> {
        // Check MIME type
        if !ALLOWED_VIDEO_TYPES.contains(&mime_type) {
            return Err(ValidationError::InvalidVideoFormat);
        }

        // Use existing size validation
        Self::validate_video_size(data)
    }

    /// Detect MIME type from file data
    pub fn detect_mime_type(data: &[u8]) -> Result<String, ValidationError> {
        let kind = infer::get(data)
            .ok_or(ValidationError::UnknownFileType)?;

        let mime = kind.mime_type();
        
        // Verify it's an allowed type
        if ALLOWED_IMAGE_TYPES.contains(&mime) || ALLOWED_VIDEO_TYPES.contains(&mime) {
            Ok(mime.to_string())
        } else {
            Err(ValidationError::UnSupportedMediaType(mime.to_string()))
        }
    }
}

/// Helper: Generate content hash
pub fn generate_content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

/// Helper: Check if string is emoji
fn is_emoji(s: &str) -> bool {
    s.chars().any(|c| {
        matches!(c as u32,
            0x1F600..=0x1F64F | // Emoticons
            0x1F300..=0x1F5FF | // Misc Symbols and Pictographs
            0x1F680..=0x1F6FF | // Transport and Map
            0x2600..=0x26FF |   // Misc symbols
            0x2700..=0x27BF |   // Dingbats
            0xFE00..=0xFE0F |   // Variation Selectors
            0x1F900..=0x1F9FF | // Supplemental Symbols and Pictographs
            0x1F1E0..=0x1F1FF   // Flags
        )
    })
}

/// Helper: Check for excessive repetition
fn has_excessive_repetition(content: &str) -> bool {

    let code_block_regex = Regex::new(r"``` [\s\S]*?```|`[^`]*`").unwrap();
    let without_code = code_block_regex.replace_all(content, "");

    let chars: Vec<char> = without_code.as_ref().chars().collect();
    let mut max_repetition = 0;
    let mut current_repetition = 1;

    for i in 1..chars.len() {
        if chars[i] == chars[i - 1] && chars[1] != ' ' {
            current_repetition += 1;
            max_repetition = max_repetition.max(current_repetition);
        } else {
            current_repetition = 1;
        }
    }

    max_repetition >= 10 // 10+ same characters in a row
}

/// Helper: Extract URLs from content
fn extract_urls(content: &str) -> Vec<String> {
    let url_regex = Regex::new(
        r"https?://[^\s<>]+|www\.[^\s<>]+"
    ).expect("URL_REGEX compilation failed");

    url_regex
        .find_iter(content)
        .map(|m| m.as_str().to_string())
        .collect()
}

pub fn generate_post_slug(title: Option<&str>, first_tag: Option<&str>) -> String {
    let mut parts = vec![];
    
    if let Some(title) = title {
        let words: Vec<_> = title
            .unicode_words()
            .take(3)
            .map(|w| w.to_lowercase())
            .collect();
        parts.extend(words);
    }
    
    if let Some(tag) = first_tag {
        parts.push(tag.to_lowercase());
    }
    
    // Return without ID (ID added later)
    if parts.is_empty() {
        "post".to_string()  // Fallback
    } else {
        parts.join("-")
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_username_validation() {
        assert!(UsernameValidator::validate("john_doe").is_ok());
        assert!(UsernameValidator::validate("user.name").is_ok());
        assert!(UsernameValidator::validate("user123").is_ok());
        
        assert!(UsernameValidator::validate("ab").is_err()); // Too short
        assert!(UsernameValidator::validate("user..name").is_err()); // Consecutive dots
        assert!(UsernameValidator::validate("admin").is_err()); // Reserved
        assert!(UsernameValidator::validate(".username").is_err()); // Starts with dot
        assert!(UsernameValidator::validate("username_").is_err()); // Ends with underscore
    }

    #[test]
    fn test_tag_validation() {
        assert!(TagValidator::validate_name("rust").is_ok());
        assert!(TagValidator::validate_name("c++").is_ok());
        assert!(TagValidator::validate_name("c#").is_ok());
        assert!(TagValidator::validate_name("node.js").is_ok());
        
        assert!(TagValidator::validate_name("Rust Programming").is_err()); // Space
        assert!(TagValidator::validate_name("🔥rust").is_err()); // Emoji
    }

    #[test]
    fn test_content_validation() {
        let valid_content = "This is a valid post with enough content to pass validation.";
        assert!(ContentValidator::validate_post(valid_content).is_ok());
        
        let too_short = "Short";
        assert!(ContentValidator::validate_post(too_short).is_err());
        
        let spam = "HELLO EVERYONE THIS IS SPAM CONTENT ALL IN CAPS!!!!!";
        assert!(ContentValidator::validate_post(spam).is_err());
    }
}