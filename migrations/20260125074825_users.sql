
-- =====================================================
-- CORE USER TABLES
-- =====================================================

-- Users table with OAuth support (GitHub/Twitter)
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    
    -- OAuth identifiers (nullable to support both providers)
    github_id BIGINT UNIQUE,
    github_username VARCHAR(255),
    twitter_id BIGINT UNIQUE,
    twitter_username VARCHAR(255),
    
    -- Core user info (manually entered in platform)
    username VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    
    -- Profile
    avatar_url TEXT,
    bio_raw TEXT,  -- Markdown source
    bio_rendered_html TEXT,  -- Rendered HTML for display
    bio_content_hash VARCHAR(64),  -- Hash to check if re-rendering needed
    
    -- Metadata
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMP WITH TIME ZONE,  -- Soft delete
    last_username_changed_at TIMESTAMP WITH TIME ZONE,
    
    -- At least one OAuth provider required
    CONSTRAINT at_least_one_oauth CHECK (github_id IS NOT NULL OR twitter_id IS NOT NULL)
);

CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_github_id ON users(github_id);
CREATE INDEX idx_users_twitter_id ON users(twitter_id);
CREATE INDEX idx_users_created_at ON users(created_at DESC);

-- User social links (Twitter, GitHub, YouTube, Website, etc.)
CREATE TABLE user_links (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    platform VARCHAR(50) NOT NULL,  -- 'twitter', 'github', 'email', 'youtube', 'website', 'linkedin', 'other'
    url VARCHAR(500) NOT NULL,
    display_text VARCHAR(100),  -- Custom text like "My Portfolio"
    display_order INTEGER DEFAULT 0,  -- For ordering links in profile
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    CONSTRAINT unique_user_platform UNIQUE(user_id, platform)
);

CREATE INDEX idx_user_links_user_id ON user_links(user_id);