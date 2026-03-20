use sha2::{Sha256, Digest};
use sqlx::PgPool;
use regex::Regex;
use lazy_static::lazy_static;
use std::collections::HashSet;
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct SpamScore {
    pub score: u8,
    pub is_spam: bool,
    pub reasons: Vec<String>,
}

impl SpamScore {
    pub fn new() -> Self {
        Self {
            score: 0,
            is_spam: false,
            reasons: Vec::new(),
        }
    }

    pub fn add_score(&mut self, points: u8, reason: String) {
        self.score = self.score.saturating_add(points);
        self.reasons.push(reason);

        if self.score >= 60 {
            self.is_spam = true;
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ContentType {
    Post,
    Commentdepth0,
    Commentdepth1,
    Commentdepth2,
    Commentdepth3,
    Answer,
    Question,
    Refract,
}

impl ContentType {
    fn min_length(&self) -> usize {
        match self {
            Self::Post => 10,
            Self::Question => 10,
            Self::Answer => 10,
            Self::Refract => 10,
            Self::Commentdepth0 => 10,
            Self::Commentdepth1 => 5,
            Self::Commentdepth2 => 5,
            Self::Commentdepth3 => 5,
        }
    }

    fn max_length(&self) -> usize {
        match self {
            Self::Post => 30_000,
            Self::Question => 15_000,
            Self::Answer => 30_000,
            Self::Commentdepth0 => 1_500,
            Self::Commentdepth1 => 1_000,
            Self::Commentdepth2 => 500,
            Self::Commentdepth3 => 250,
            Self::Refract => 1_500,
        }
    }
}

pub async fn detect_spam(
    content_raw: &str,
    user_id: i32,
    pool: &PgPool,
    content_type: ContentType,
) -> Result<SpamScore, sqlx::Error> {
    let mut score = SpamScore::new();

    check_content_quality(content_raw, content_type, &mut score);

    check_spam_patterns(content_raw, &mut score);

    check_user_behavior(user_id, content_raw, pool, content_type, &mut score).await?;

    check_duplicates(content_raw, user_id, pool, content_type, &mut score).await?;

    check_link_spam(content_raw, &mut score);

    Ok(score)
}

fn check_content_quality(content: &str, content_type: ContentType, score: &mut SpamScore) {
    let trimmed = content.trim();
    let length = trimmed.len();

    if matches!(content_type, ContentType::Post | ContentType::Question | ContentType::Answer) {
        if length < content_type.min_length() {
            score.add_score(40, format!("Content too short ({} chars)", length));
            return;
        }
    }

    if length > content_type.max_length() {
        score.add_score(20, format!("Content suspiciously long ({} chars)", length));
    }

    let alpha_chars: Vec<char> = trimmed.chars().filter(|c| c.is_alphabetic()).collect();
    if !alpha_chars.is_empty() {
        let uppercase_ratio = alpha_chars.iter().filter(|c| c.is_uppercase()).count() as f32 / alpha_chars.len() as f32;

        if uppercase_ratio > 0.7 && trimmed.len() > 20 {
            score.add_score(25, "Excessive uppercase (shouting)".to_string());
        }
    }

    if has_excessive_character_repetition(trimmed) {
        score.add_score(30, "Excessive character repetition".to_string());
    }

    let emoji_count = count_emojis(trimmed);
    if trimmed.chars().count() > 0 {
        let emoji_ration = emoji_count as f32 / trimmed.chars().count() as f32;

        if emoji_ratio > 0.3 {
            score.add_score(15, format!("Excessive emojis ({}%)", (emoji_ratio * 100.0) as u8));
        }
    }

    if is_gibberish(trimmed) {
        score.add_score(50, "Content appears to be gibberish".to_string());
    }

    let alpha_count = trimmed.chars().filter(|c| c.is_alphabetic()).count();
    if alpha_count < 3 && trimmed.len() > 10 {
        score.add_score(35, "No meaningful text content".to_string());
    }
}

fn check_spam_patterns(content: &str, score: &mut SpamScore) {
    lazy_static! {
        static ref SPAM_KEYWORDS: HashSet<&'static str> = {
            let mut set = HashSet::new();
            set.insert("free money");
            set.insert("make money fast");
            set.insert("get rich quick");
            set.insert("work from home");
            set.insert("click here now");
            set.insert("click here");
            set.insert("visit this site");
            set.insert("limited time offer");
            set.insert("act now");
            set.insert("buy now");
            set.insert("viagra");
            set.insert("cialis");
            set.insert("online casino");
            set.insert("bitcoin investment");
            set.insert("enlargement");
            set.insert("weight loss");
            set.insert("lose weight");
            set.insert("100% free");
            set.insert("no credit card");
            set.insert("special promotion");
            set.insert("urgent response needed");
            set.insert("call now");
            set.insert("free trial");
            set.insert("risk-free");
            set.insert("earn money");
            set.insert("casino online");
            set.insert("forex trading");
            set.insert("binary options");
            set.insert("crypto pump");
            set.insert("guaranteed profit");
            set.insert("mlm opportunity");
            set.insert("join now");
            set.insert("be your own boss");
            set.insert("passive income");

            set
        };

        static ref SPAM_PATTERNS: Vec<Regex> = vec![
            Regex::new(r"!{3,}").unwrap(),
            Regex::new(r"(\+?\d{1,3}[-.\s]?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4})").unwrap(),
            Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap(),
            Regex::new(r"\b[13][a-km-zA-HJ-NP-Z1-9]{25,34}\b").unwrap(),
            Regex::new(r"bit\.ly|tinyurl|goo\.gl|ow\.ly").unwrap(),
        ];
    }

    let lower_content = content.to_lowercase();

    for keyword in SPAM_KEYWORDS.iter() {
        if lower_content.contains(keyword) {
            keyword_matches += 1;
            score.add_score(20, format!("Spam keyword: '{}'", keyword));
        }
    }

    if keyword_matches >= 3 {
        score.add_score(30, "Multiple spam keywords detected".to_string());
    }

    for (idx, pattern) in SPAM_PATTERNS.iter().enumerate() {
        if pattern.is_match(content) {
            let points = match idx {
                0 => 10,
                1 => 15,
                2 => 15,
                3 => 25,
                4 => 20,
                _ => 5,
            };

            score.add_score(points, format!("Spam pattern detected"));
        }
    }
}

async fn check_user_behavior(
    user_id: i32,
    content: &str,
    pool: &PgPool,
    content_type: ContentType,
    score: &mut SpamScore,
) -> Result<(), sqlx::Error> {
    let user_stats = sqlx::query!(
        r#"
        SELECT 
            created_at,
            (SELECT COUNT(*) FROM posts WHERE user_id = $1 AND deleted_at IS NULL) as "post_count!",
            (SELECT COUNT(*) FROM questions WHERE user_id = $1 AND deleted_at IS NULL) as "question_count!",
            (SELECT COUNT(*) FROM comments WHERE user_id = $1 AND deleted_at IS NULL) as "comment_count!",
            (SELECT COUNT(*) FROM posts WHERE user_id = $1 AND deleted_at IS NOT NULL) as "deleted_count!"
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_optional(pool)
    .await?;

    if let Some(stats) = user_stats {
        let account_age_days = (OffsetDateTime::now_utc() - stats.created_at).whole_days();
        let total_content = stats.post_count + stats.question_count + stats.comment_count;

        if account_age_days < 1 {
            if total_content == 0 && content.trim().split_whitespace().count() < 10 {
                score.add_score(20, "New account with very short first post".to_string());
            }

            if total_content < 3 && has_external_links(content) {
                score.add_score(25, "New account posting links".to_string());
            }
        }

        if total_content > 5 && stats.deleted_count > total_content / 2 {
            score.add_score(15, "High content deletion rate".to_string());
        }

        let recent_posts = sqlx::query!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM (
                SELECT created_at FROM posts WHERE user_id = $1 AND created_at > NOW() - INTERVAL '10 minutes'
                UNION ALL
                SELECT created_at FROM questions WHERE user_id = $1 AND created_at > NOW() - INTERVAL '10 minutes'
                UNION ALL
                SELECT created_at FROM comments WHERE user_id = $1 AND created_at > NOW() - INTERVAL '10 minutes'
            ) combined
            "#
            user_id
        )
        .fetch_one(pool)
        .await?;

        if recent_posts > 5 {
            score.add_score(30, format!("Rapid posting: {} posts in 10 minutes", recent_posts));
        }
    }

    Ok(())
}

async fn check_duplicates(
    content: &str,
    user_id: i32,
    pool: &PgPool,
    content_type: ContentType,
    score: &mut SpamScore,
) -> Result<(), sqlx::Error> {
    let content_hash = format!("{:x}", Sha256::digest(content.as_bytes()));
    
    let user_duplicates = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM (
            SELECT content_hash FROM posts WHERE user_id = $1 AND content_hash = $2 AND created_at > NOW() - INTERVAL '24 hours'
            UNION ALL
            SELECT content_hash FROM questions WHERE user_id = $1 AND content_hash = $2 AND created_at > NOW() - INTERVAL '24 hours'
            UNION ALL
            SELECT content_hash FROM comments WHERE user_id = $1 AND content_hash = $2 AND created_at > NOW() - INTERVAL '24 hours'
        ) combined
        "#,
        user_id,
        content_hash
    )
    .fetch_one(pool)
    .await?;

    if user_duplicates > 0 {
        score.add_score(50, "Duplicate content posted by same user in 24h".to_string());
    }

    let global_duplicates = sqlx::query_scalar!(
        r#" 
        SELECT COUNT(DISTINCT user_id) as "count!"
        FROM (
            SELECT user_id FROM posts WHERE content_hash = $1 AND created_at > NOW() - INTERVAL '7 days'
            UNION ALL
            SELECT user_id FROM questions WHERE content_hash = $1 AND created_at > NOW() - INTERVAL '7 days'
        ) combined
        "#, 
        content_hash
    )
    .fetch_one(pool)
    .await?;

    if global_duplicates >= 3 {
        score.add_score(40, format!("Same content posted by {} different users", global_duplicates));
    }

    Ok(())
}

fn check_link_spam(content: &str, score: &mut SpamScore) {
    lazy_static! {
        static ref URL_REGEX: Regex = Regex::new(r"https?://[^\s<>]+").unwrap();
        static ref SUSPICIOUS_DOMAINS: HashSet<&'static str> = {
            let mut set = HashSet::new();
            set.insert(".tk");
            set.insert(".ml");
            set.insert(".ga");
            set.insert(".cf");
            set.insert(".gq");
            set.insert(".ru");
            set.insert(".club");
            set.insert(".xyz");
            set.insert(".top");
            set.insert(".ru");
            set.insert(".dev");
            set
        };
    }

    let urls: Vec<_> = URL_REGEX.find_iter(content).collect();
    let url_count = urls.len();

    if url_count > 5 {
        score.add_score(20, format!("Excessive links({})", url_count));
    }

    for url_match in urls {
        let url = url_match.as_str();

        for domain in SUSPICIOUS_DOMAINS.iter() {
            if url.contains(domain) {
                score.add_score(30, format!("Suspicious domain: {}", domain));
                break;
            }
        }
    }

    let text_without_links = URL_REGEX.replace_all(content, "").to_string();
    let text_length = text_without_links.trim().len();
    let links_length = content.len() - text_length;

    if content.len() > 50 && links_length as f32 / content.len() as f32 > 0.5 {
        score.add_score(25, "Content is mostly links".to_string());
    }
}

fn has_excessive_character_repetition(content: &str) -> bool {
    let chars: Vec<char> = content.chars().collect();
    let mut max_repeat = 0;
    let mut current_repeat = 1;

    for i in 1..chars.len() {
        if chars[i] == chars[i - 1] {
            current_repeat += 1;
            max_repeat = max_repeat.max(current_repeat);
        } else {
            current_repeat = 1;
        }
    }

    max_repeat >= 10
}

fn count_emojis(content: &str) -> usize {
    content.chars().filter(|c| {
        matches!(*c as u32,
            0x1F600..=0x1F64F |
            0x1F300..=0x1F5FF |
            0x1F680..=0x1F6FF |
            0x2600..=0x26FF |
            0x2700..=0x27BF |
            0x1F900..=0x1F9FF |
            0x1F1E0..=0x1F1FF
        )
    }).count()
}

fn is_gibberish(content: &str) -> bool {
    let words: Vec<&str> = content.split_whitespace().collect();
    if words.is_empty() {
        return false;
    }

    let gibberish_words = words.iter().filter(|word| {
        let has_vowel = word.chars().any(|c| matches!(c.to_lowercase().next(), Some('a' | 'e' | 'i' | 'o' | 'u')));
        !has_vowel && word.len() > 3
    }).count();

    gibberish_words as f32 / words.len() as f32 > 0.6
}

fn has_external_links(content: &str) -> bool {
    lazy_static! {
        static ref URL_REGEX: Regex = Regex::new(r"https?://").unwrap();
    }
    URL_REGEX.is_match(content)
}
