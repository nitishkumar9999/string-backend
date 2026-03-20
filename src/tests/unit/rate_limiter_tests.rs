
#[cfg(test)]
mod rate_limiter_tests {
    use crate::utils::rate_limit::*;
    use std::net::{IpAddr, Ipv4Addr};
    
    #[tokio::test]
    async fn test_rate_limiter_allows_within_limit() {
        let limiter = RateLimiter::new();
        let user_id = 1;
        
        // Should allow first request
        let result = limiter.check_user_limit(user_id, RateLimitAction::Post).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_rate_limiter_blocks_over_limit() {
        let limiter = RateLimiter::new_with_config(RateLimitConfig {
            post_per_hour: 1,
            ..Default::default()
        });
        
        let user_id = 1;
        
        // First request should succeed
        limiter.check_user_limit(user_id, RateLimitAction::Post).await.unwrap();
        
        // Second request should fail
        let result = limiter.check_user_limit(user_id, RateLimitAction::Post).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_rate_limiter_ip_limit() {
        let limiter = RateLimiter::new();
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        
        // Should allow requests within limit
        let result = limiter.check_ip_limit(ip).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_rate_limiter_different_actions() {
        let limiter = RateLimiter::new();
        let user_id = 1;
        
        // Different actions have separate limits
        assert!(limiter.check_user_limit(user_id, RateLimitAction::Post).await.is_ok());
        assert!(limiter.check_user_limit(user_id, RateLimitAction::Question).await.is_ok());
        assert!(limiter.check_user_limit(user_id, RateLimitAction::Comment).await.is_ok());
    }
}
