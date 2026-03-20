mod test_helpers;
mod unit {
    mod validation_tests;
    mod markdown_tests;
    mod slug_tests;
    mod rate_limiter_tests;
    mod pagination_tests;
}

#[cfg(test)]
mod integration {
    mod auth_tests;
    mod post_tests;
    mod question_tests;
    mod answer_tests;
    mod comment_tests;
    mod echo_tests;
    mod refract_tests;
    mod feed_tests;
    mod user_tests;
}