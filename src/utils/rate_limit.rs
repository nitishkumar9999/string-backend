use crate::errors::{AppError, RateLimitError};
use sqlx::PgPool;
use std::net::IpAddr;
use time::OffsetDateTime;

/// Rate limit configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub posts_per_hour: i32,
    pub posts_per_day: i32,
    pub questions_per_hour: i32,
    pub questions_per_day: i32,
    pub comments_per_minute: i32,
    pub comments_per_hour: i32,
    pub comments_per_day: i32,
    pub answers_per_hour: i32,
    pub answers_per_day: i32,
    pub refracts_per_hour: i32,
    pub refracts_per_day: i32,
    pub tag_creation_per_hour: i32,
    pub tag_creation_per_day: i32,
    pub anonymous_per_minute: i32,
    pub anonymous_per_hour: i32,
    pub echo_per_minute: i32,
    pub update_profile_per_hour: i32,
    pub update_username_per_day: i32,
    pub max_requests: u32,
    pub window_secs: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            posts_per_hour: 10,
            posts_per_day: 50,
            questions_per_hour: 5,
            questions_per_day: 20,
            comments_per_minute: 5,
            comments_per_hour: 100,
            comments_per_day: 500,
            answers_per_hour: 20,
            answers_per_day: 100,
            refracts_per_hour: 15,
            refracts_per_day: 100,
            tag_creation_per_hour: 3,
            tag_creation_per_day: 10,
            anonymous_per_minute: 10,
            anonymous_per_hour: 100,
            echo_per_minute: 10,
            update_profile_per_hour: 5,
            update_username_per_day: 1,
            max_requests: 100,
            window_secs: 60,
        }
    }
}

/// Action types that can be rate limited
#[derive(Debug, Clone, Copy)]
pub enum RateLimitAction {
    Post,
    Question,
    Comment,
    Answer,
    Refract,
    TagCreation,
    AnonymousRequest,
    Echo,
    UpdateProfile,
    UpdateUsername,
}

impl RateLimitAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Post => "post",
            Self::Question => "question",
            Self::Comment => "comment",
            Self::Answer => "answer",
            Self::Refract => "refract",
            Self::TagCreation => "tag_creation",
            Self::AnonymousRequest => "anonymous_request",
            Self::Echo => "echo",
            Self::UpdateProfile => "update_profile",
            Self::UpdateUsername => "update_username",
        }
    }
}

/// Time window for rate limiting
#[derive(Debug, Clone, Copy)]
pub enum TimeWindow {
    Minute,
    Hour,
    Day,
}

impl TimeWindow {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Minute => "minute",
            Self::Hour => "hour",
            Self::Day => "day",
        }
    }

    /// Get the start of the current window
    pub fn window_start(&self, now: OffsetDateTime) -> OffsetDateTime {
        match self {
            Self::Minute => truncate_to_minute(now),
            Self::Hour => truncate_to_hour(now),
            Self::Day => truncate_to_day(now),
        }
    }

    /// Get seconds until window resets
    pub fn seconds_until_reset(&self, now: OffsetDateTime) -> i64 {
        let next_window = match self {
            Self::Minute => {
                let start = truncate_to_minute(now);
                start + time::Duration::minutes(1)
            }
            Self::Hour => {
                let start = truncate_to_hour(now);
                start + time::Duration::hours(1)
            }
            Self::Day => {
                let start = truncate_to_day(now);
                start + time::Duration::days(1)
            }
        };

        (next_window - now).whole_seconds()
    }
}

/// Rate limiter
pub struct RateLimiter {
    pool: PgPool,
    config: RateLimitConfig,
}

impl RateLimiter {
    pub fn new(pool: PgPool, config: RateLimitConfig) -> Self {
        Self { pool, config }
    }

    /// Check and increment rate limit for authenticated user
    pub async fn check_user_limit(
        &self,
        user_id: i32,
        action: RateLimitAction,
    ) -> Result<(), AppError> {
        let now = OffsetDateTime::now_utc();

        // Check all relevant windows for this action
        match action {
            RateLimitAction::Post => {
                self.check_limit(
                    user_id,
                    None,
                    action,
                    TimeWindow::Hour,
                    self.config.posts_per_hour,
                    now,
                )
                .await?;

                self.check_limit(
                    user_id,
                    None,
                    action,
                    TimeWindow::Day,
                    self.config.posts_per_day,
                    now,
                )
                .await?;
            }

            RateLimitAction::Question => {
                self.check_limit(
                    user_id,
                    None,
                    action,
                    TimeWindow::Hour,
                    self.config.questions_per_hour,
                    now,
                )
                .await?;

                self.check_limit(
                    user_id,
                    None,
                    action,
                    TimeWindow::Day,
                    self.config.questions_per_day,
                    now,
                )
                .await?;
            }

            RateLimitAction::Comment => {
                self.check_limit(
                    user_id,
                    None,
                    action,
                    TimeWindow::Minute,
                    self.config.comments_per_minute,
                    now,
                )
                .await?;

                self.check_limit(
                    user_id,
                    None,
                    action,
                    TimeWindow::Hour,
                    self.config.comments_per_hour,
                    now,
                )
                .await?;

                self.check_limit(
                    user_id,
                    None,
                    action,
                    TimeWindow::Day,
                    self.config.comments_per_day,
                    now,
                )
                .await?;
            }

            RateLimitAction::Answer => {
                self.check_limit(
                    user_id,
                    None,
                    action,
                    TimeWindow::Hour,
                    self.config.answers_per_hour,
                    now,
                )
                .await?;

                self.check_limit(
                    user_id,
                    None,
                    action,
                    TimeWindow::Day,
                    self.config.answers_per_day,
                    now,
                )
                .await?;
            }

            RateLimitAction::Refract => {
                self.check_limit(
                    user_id,
                    None,
                    action,
                    TimeWindow::Hour,
                    self.config.refracts_per_hour,
                    now,
                )
                .await?;

                self.check_limit(
                    user_id,
                    None,
                    action,
                    TimeWindow::Day,
                    self.config.refracts_per_day,
                    now,
                )
                .await?;
            }

            RateLimitAction::TagCreation => {
                self.check_limit(
                    user_id,
                    None,
                    action,
                    TimeWindow::Hour,
                    self.config.tag_creation_per_hour,
                    now,
                )
                .await?;

                self.check_limit(
                    user_id,
                    None,
                    action,
                    TimeWindow::Day,
                    self.config.tag_creation_per_day,
                    now,
                )
                .await?;
            }

            RateLimitAction::AnonymousRequest => {
                // Should use IP-based check instead
                return Ok(());
            }

            RateLimitAction::Echo => {
                self.check_limit(
                    user_id,
                    None,
                    action,
                    TimeWindow::Minute,
                    self.config.echo_per_minute,
                    now,
                )
                .await?;
            }

            RateLimitAction::UpdateProfile => {
                self.check_limit(
                    user_id,
                    None,
                    action,
                    TimeWindow::Hour,
                    self.config.update_profile_per_hour,
                    now,
                )
                .await?;
            }

            RateLimitAction::UpdateUsername => {
                self.check_limit(
                    user_id,
                    None,
                    action,
                    TimeWindow::Day,
                    self.config.update_username_per_day,
                    now,
                )
                .await?;
            }

            
        }

        Ok(())
    }

    /// Check and increment rate limit for anonymous users (by IP)
    pub async fn check_ip_limit(&self, ip: IpAddr) -> Result<(), AppError> {
        let now = OffsetDateTime::now_utc();

        // Check minute window
        let count = self.check_limit(
            0, // Use 0 for anonymous
            Some(ip),
            RateLimitAction::AnonymousRequest,
            TimeWindow::Minute,
            self.config.anonymous_per_minute,
            now,
        )
        .await?;
        
        let limit = self.config.anonymous_per_minute;

        // Check hour window
        self.check_limit(
            0,
            Some(ip),
            RateLimitAction::AnonymousRequest,
            TimeWindow::Hour,
            self.config.anonymous_per_hour,
            now,
        )
        .await?;



        Ok(())
    }

    /// Internal check and increment logic
    async fn check_limit(
        &self,
        user_id: i32,
        ip: Option<IpAddr>,
        action: RateLimitAction,
        window: TimeWindow,
        limit: i32,
        now: OffsetDateTime,
    ) -> Result<(), AppError> {
        let window_start = window.window_start(now);
        let action_str = action.as_str();
        let window_str = window.as_str();

        // Upsert the rate limit counter
        let count = if let Some(ip_addr) = ip {
            // IP-based rate limit
            let ip_str = ip_addr.to_string();
            let result = sqlx::query!(
                r#"
                INSERT INTO rate_limits (user_id, ip_address, action_type, window_type, window_start, count)
                VALUES (0, $1::text::inet, $2, $3, $4, 1)
                ON CONFLICT (ip_address, action_type, window_type, window_start)
                   WHERE ip_address IS NOT NULL AND user_id = 0
                DO UPDATE SET count = rate_limits.count + 1
                RETURNING count
                "#,
                ip_str,
                action_str,
                window_str,
                window_start
            )
            .fetch_one(&self.pool)
            .await?;

            result.count
        } else {
            // User-based rate limit
            let result = sqlx::query!(
                r#"
                INSERT INTO rate_limits (user_id, action_type, window_type, window_start, count)
                VALUES ($1, $2, $3, $4, 1)
                ON CONFLICT (user_id, action_type, window_type, window_start)
                WHERE user_id > 0 AND ip_address IS NULL
                DO UPDATE SET count = rate_limits.count + 1
                RETURNING count
                "#,
                user_id,
                action_str,
                window_str,
                window_start
            )
            .fetch_one(&self.pool)
            .await?;

            result.count
        };

        // Check if limit exceeded
        if count > limit {
            let retry_after = window.seconds_until_reset(now);

            return Err(AppError::RateLimit(RateLimitError {
                action: action_str.to_string(),
                limit: format!("{} per {}", limit, window_str),
                current_count: count,
                retry_after_seconds: retry_after,
                window_resets_at: window_start + match window {
                    TimeWindow::Minute => time::Duration::minutes(1),
                    TimeWindow::Hour => time::Duration::hours(1),
                    TimeWindow::Day => time::Duration::days(1),
                },
            }));
        }

        Ok(())
    }

    /// Cleanup old rate limit records (run periodically)
    pub async fn cleanup_old_records(&self) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            DELETE FROM rate_limits
            WHERE window_start < NOW() - INTERVAL '24 hours'
            "#
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}

/// Helper: Truncate to minute
fn truncate_to_minute(dt: OffsetDateTime) -> OffsetDateTime {
    let (year, month, day) = dt.to_calendar_date();
    let (hour, minute, _) = dt.to_hms();
    
    time::Date::from_calendar_date(year, month, day)
        .unwrap()
        .with_hms(hour, minute, 0)
        .unwrap()
        .assume_utc()
}

/// Helper: Truncate to hour
fn truncate_to_hour(dt: OffsetDateTime) -> OffsetDateTime {
    let (year, month, day) = dt.to_calendar_date();
    let hour = dt.hour();
    
    time::Date::from_calendar_date(year, month, day)
        .unwrap()
        .with_hms(hour, 0, 0)
        .unwrap()
        .assume_utc()
}

/// Helper: Truncate to day
fn truncate_to_day(dt: OffsetDateTime) -> OffsetDateTime {
    let (year, month, day) = dt.to_calendar_date();
    
    time::Date::from_calendar_date(year, month, day)
        .unwrap()
        .with_hms(0, 0, 0)
        .unwrap()
        .assume_utc()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_to_hour() {
        let dt = OffsetDateTime::parse(
            "2025-01-26T15:45:30Z",
            &time::format_description::well_known::Rfc3339,
        )
        .unwrap();

        let truncated = truncate_to_hour(dt);
        assert_eq!(truncated.hour(), 15);
        assert_eq!(truncated.minute(), 0);
        assert_eq!(truncated.second(), 0);
    }

    #[test]
    fn test_time_window_seconds_until_reset() {
        let now = OffsetDateTime::parse(
            "2025-01-26T15:45:30Z",
            &time::format_description::well_known::Rfc3339,
        )
        .unwrap();

        let window = TimeWindow::Hour;
        let seconds = window.seconds_until_reset(now);

        // Should be 14 minutes 30 seconds = 870 seconds
        assert_eq!(seconds, 870);
    }
}

