
-- =====================================================
-- TAGS SYSTEM
-- =====================================================

-- Core tags for categorization
CREATE TABLE tags (
    id SERIAL PRIMARY KEY,
    name VARCHAR(35) UNIQUE NOT NULL,  -- Lowercase, allows hyphens and special chars (c++, c#, .net)
    slug VARCHAR(35) UNIQUE NOT NULL,  -- URL-friendly version
    
    -- Analytics
    usage_count INTEGER DEFAULT 0,  -- Total posts + questions using this tag
    follower_count INTEGER DEFAULT 0,  -- Users explicitly following
    
    -- Metadata
    created_by_user_id INTEGER REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    is_active BOOLEAN DEFAULT TRUE  -- Admins can deactivate spam tags
);

CREATE INDEX idx_tags_name ON tags(name);
CREATE INDEX idx_tags_slug ON tags(slug);
CREATE INDEX idx_tags_usage_count ON tags(usage_count DESC);
CREATE INDEX idx_tags_created_at ON tags(created_at DESC);

-- User explicitly follows tags (for personalized feed)
CREATE TABLE user_tag_follows (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    CONSTRAINT unique_user_tag_follow UNIQUE(user_id, tag_id)
);

CREATE INDEX idx_user_tag_follows_user_id ON user_tag_follows(user_id);
CREATE INDEX idx_user_tag_follows_tag_id ON user_tag_follows(tag_id);

-- =====================================================
-- POSTS SYSTEM
-- =====================================================

-- Main posts table (Twitter-like posts with optional title)
CREATE TABLE posts (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Content (supports markdown)
    title VARCHAR(300),  -- Optional title
    content_raw TEXT NOT NULL CHECK (char_length(content_raw) <= 30000),
    content_rendered_html TEXT NOT NULL,
    content_hash VARCHAR(64),
    
    -- Metadata
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMP WITH TIME ZONE,  -- Soft delete
    edited_at TIMESTAMP WITH TIME ZONE,  -- Track if edited
    
    -- Engagement metrics
    view_count INTEGER DEFAULT 0,
    echo_count INTEGER DEFAULT 0,  -- Promotion count
    refract_count INTEGER DEFAULT 0,  -- Modified repost count
    comment_count INTEGER DEFAULT 0
);

CREATE INDEX idx_posts_user_id ON posts(user_id);
CREATE INDEX idx_posts_created_at ON posts(created_at DESC);
CREATE INDEX idx_posts_echo_count ON posts(echo_count DESC);

-- Post media attachments (images, videos, files)
CREATE TABLE post_media (
    id SERIAL PRIMARY KEY,
    post_id INTEGER NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    media_type VARCHAR(20) NOT NULL CHECK (media_type IN ('image', 'video', 'file')),
    
    -- File storage (local or CDN URL)
    file_path TEXT NOT NULL,  -- '/media/posts/123/image.jpg' or CDN URL
    original_filename VARCHAR(255),
    mime_type VARCHAR(100),
    file_size BIGINT,  -- Bytes
    
    -- Image/Video metadata
    width INTEGER,
    height INTEGER,
    duration INTEGER,  -- For videos (seconds)
    thumbnail_path TEXT,  -- For videos
    
    -- Display
    display_order INTEGER DEFAULT 0,
    uploaded_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_post_media_post_id ON post_media(post_id);

-- Post-Tag relationship (many-to-many, 1-5 tags per post)
CREATE TABLE post_tags (
    id SERIAL PRIMARY KEY,
    post_id INTEGER NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    CONSTRAINT unique_post_tag UNIQUE(post_id, tag_id)
);

CREATE INDEX idx_post_tags_post_id ON post_tags(post_id);
CREATE INDEX idx_post_tags_tag_id ON post_tags(tag_id);

-- =====================================================
-- QUESTIONS & ANSWERS SYSTEM
-- =====================================================

-- Questions table (Stack Overflow-like Q&A)
CREATE TABLE questions (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Content
    title VARCHAR(300) NOT NULL,  -- Required for questions
    content_raw TEXT NOT NULL CHECK (char_length(content_raw) <= 15000),
    content_rendered_html TEXT NOT NULL,
    content_hash VARCHAR(64),
    
    -- Answer management (author-only features)
    is_solved BOOLEAN DEFAULT FALSE,  -- Author marks as solved, stops notifications
    
    -- Metadata
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMP WITH TIME ZONE,
    edited_at TIMESTAMP WITH TIME ZONE,
    
    -- Engagement metrics
    view_count INTEGER DEFAULT 0,
    echo_count INTEGER DEFAULT 0,  -- Others can promote, hidden in UI
    answer_count INTEGER DEFAULT 0,
    comment_count INTEGER DEFAULT 0
);

CREATE INDEX idx_questions_user_id ON questions(user_id);
CREATE INDEX idx_questions_is_solved ON questions(is_solved);
CREATE INDEX idx_questions_created_at ON questions(created_at DESC);
CREATE INDEX idx_questions_answer_count ON questions(answer_count DESC);

-- Question media attachments
CREATE TABLE question_media (
    id SERIAL PRIMARY KEY,
    question_id INTEGER NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
    media_type VARCHAR(20) NOT NULL CHECK (media_type IN ('image', 'video', 'file')),
    
    file_path TEXT NOT NULL,
    original_filename VARCHAR(255),
    mime_type VARCHAR(100),
    file_size BIGINT,
    
    width INTEGER,
    height INTEGER,
    duration INTEGER,
    thumbnail_path TEXT,
    
    display_order INTEGER DEFAULT 0,
    uploaded_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_question_media_question_id ON question_media(question_id);

-- Question-Tag relationship (many-to-many, 1-5 tags per question)
CREATE TABLE question_tags (
    id SERIAL PRIMARY KEY,
    question_id INTEGER NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    CONSTRAINT unique_question_tag UNIQUE(question_id, tag_id)
);

CREATE INDEX idx_question_tags_question_id ON question_tags(question_id);
CREATE INDEX idx_question_tags_tag_id ON question_tags(tag_id);

-- Answers to questions
CREATE TABLE answers (
    id SERIAL PRIMARY KEY,
    question_id INTEGER NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Content
    content_raw TEXT NOT NULL CHECK (char_length(content_raw) <= 30000),
    content_rendered_html TEXT NOT NULL,
    content_hash VARCHAR(64),
    
    -- Metadata
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMP WITH TIME ZONE,
    edited_at TIMESTAMP WITH TIME ZONE,
    
    -- Engagement (ranked by echo_count)
    echo_count INTEGER DEFAULT 0,
    comment_count INTEGER DEFAULT 0
);

CREATE INDEX idx_answers_question_id ON answers(question_id);
CREATE INDEX idx_answers_user_id ON answers(user_id);
CREATE INDEX idx_answers_echo_count ON answers(echo_count DESC);
CREATE INDEX idx_answers_created_at ON answers(created_at DESC);

-- Answer media attachments
CREATE TABLE answer_media (
    id SERIAL PRIMARY KEY,
    answer_id INTEGER NOT NULL REFERENCES answers(id) ON DELETE CASCADE,
    media_type VARCHAR(20) NOT NULL CHECK (media_type IN ('image', 'video', 'file')),
    
    file_path TEXT NOT NULL,
    original_filename VARCHAR(255),
    mime_type VARCHAR(100),
    file_size BIGINT,
    
    width INTEGER,
    height INTEGER,
    duration INTEGER,
    thumbnail_path TEXT,
    
    display_order INTEGER DEFAULT 0,
    uploaded_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_answer_media_answer_id ON answer_media(answer_id);

-- =====================================================
-- REFRACTS SYSTEM (Quote Tweets - Simplified)
-- =====================================================

-- Refracts: User's commentary on a post (no media, no tags, no comments)
CREATE TABLE refracts (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    original_post_id INTEGER NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    
    -- User's commentary only (1500 char limit, no media, no tags)
    content_raw TEXT NOT NULL CHECK (char_length(content_raw) <= 1500),
    content_rendered_html TEXT NOT NULL,
    content_hash VARCHAR(64),
    
    -- Metadata
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMP WITH TIME ZONE,
    edited_at TIMESTAMP WITH TIME ZONE,
    
    -- Engagement (only echoes, no comments)
    echo_count INTEGER DEFAULT 0,
    
    -- User can only refract a post once
    CONSTRAINT unique_user_post_refract UNIQUE(user_id, original_post_id)
);

CREATE INDEX idx_refracts_user_id ON refracts(user_id);
CREATE INDEX idx_refracts_original_post_id ON refracts(original_post_id);
CREATE INDEX idx_refracts_created_at ON refracts(created_at DESC);
CREATE INDEX idx_refracts_echo_count ON refracts(echo_count DESC);

-- =====================================================
-- COMMENTS SYSTEM
-- =====================================================

-- Comments on posts, questions, and answers (threaded, max depth 3)
CREATE TABLE comments (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- What is being commented on (one of these must be set)
    post_id INTEGER REFERENCES posts(id) ON DELETE CASCADE,
    question_id INTEGER REFERENCES questions(id) ON DELETE CASCADE,
    answer_id INTEGER REFERENCES answers(id) ON DELETE CASCADE,
    
    -- Threading (max depth 3: parent -> child -> child -> child)
    parent_comment_id INTEGER REFERENCES comments(id) ON DELETE CASCADE,
    depth_level INTEGER DEFAULT 0 CHECK (depth_level <= 3),  -- 0=parent, max 3
    
    -- Content (char limits: 1500/1000/500/250 by depth)
    content_raw TEXT NOT NULL,
    content_rendered_html TEXT NOT NULL,
    content_hash VARCHAR(64),
    
    -- Metadata
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMP WITH TIME ZONE,
    edited_at TIMESTAMP WITH TIME ZONE,
    
    -- Engagement
    helpful_count INTEGER DEFAULT 0,
    reply_count INTEGER DEFAULT 0,  -- Direct children count
    
    -- Must comment on exactly ONE thing
    CONSTRAINT only_one_parent CHECK (
        (post_id IS NOT NULL AND question_id IS NULL AND answer_id IS NULL) OR
        (post_id IS NULL AND question_id IS NOT NULL AND answer_id IS NULL) OR
        (post_id IS NULL AND question_id IS NULL AND answer_id IS NOT NULL)
    )
);

CREATE INDEX idx_comments_post_id ON comments(post_id);
CREATE INDEX idx_comments_question_id ON comments(question_id);
CREATE INDEX idx_comments_answer_id ON comments(answer_id);
CREATE INDEX idx_comments_parent_id ON comments(parent_comment_id);
CREATE INDEX idx_comments_user_id ON comments(user_id);
CREATE INDEX idx_comments_created_at ON comments(created_at DESC);

-- Comment media (only parent comments can have media)
CREATE TABLE comment_media (
    id SERIAL PRIMARY KEY,
    comment_id INTEGER NOT NULL REFERENCES comments(id) ON DELETE CASCADE,
    media_type VARCHAR(20) NOT NULL CHECK (media_type IN ('image', 'video')),
    
    file_path TEXT NOT NULL,
    original_filename VARCHAR(255),
    mime_type VARCHAR(100),
    file_size BIGINT,
    
    width INTEGER,
    height INTEGER,
    duration INTEGER,
    thumbnail_path TEXT,
    
    uploaded_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_comment_media_comment_id ON comment_media(comment_id);

-- Track who marked comments as helpful
CREATE TABLE comment_helpful (
    id SERIAL PRIMARY KEY,
    comment_id INTEGER NOT NULL REFERENCES comments(id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    CONSTRAINT unique_comment_helpful UNIQUE(comment_id, user_id)
);

CREATE INDEX idx_comment_helpful_comment_id ON comment_helpful(comment_id);
CREATE INDEX idx_comment_helpful_user_id ON comment_helpful(user_id);

-- =====================================================
-- ECHOS SYSTEM (Promotion/Boost)
-- =====================================================

-- Echos: Promote content (once only, cannot undo)
CREATE TABLE echos (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- What is being echoed (one of these must be set)
    post_id INTEGER REFERENCES posts(id) ON DELETE CASCADE,
    question_id INTEGER REFERENCES questions(id) ON DELETE CASCADE,
    answer_id INTEGER REFERENCES answers(id) ON DELETE CASCADE,
    refract_id INTEGER REFERENCES refracts(id) ON DELETE CASCADE,
    
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    -- Must echo exactly ONE thing
    CONSTRAINT only_one_target CHECK (
        (post_id IS NOT NULL AND question_id IS NULL AND answer_id IS NULL AND refract_id IS NULL) OR
        (post_id IS NULL AND question_id IS NOT NULL AND answer_id IS NULL AND refract_id IS NULL) OR
        (post_id IS NULL AND question_id IS NULL AND answer_id IS NOT NULL AND refract_id IS NULL) OR
        (post_id IS NULL AND question_id IS NULL AND answer_id IS NULL AND refract_id IS NOT NULL)
    ),
    
    -- User can echo each item only once (no undo)
    CONSTRAINT unique_user_post_echo UNIQUE(user_id, post_id),
    CONSTRAINT unique_user_question_echo UNIQUE(user_id, question_id),
    CONSTRAINT unique_user_answer_echo UNIQUE(user_id, answer_id),
    CONSTRAINT unique_user_refract_echo UNIQUE(user_id, refract_id)
);

CREATE INDEX idx_echos_user_id ON echos(user_id);
CREATE INDEX idx_echos_post_id ON echos(post_id);
CREATE INDEX idx_echos_question_id ON echos(question_id);
CREATE INDEX idx_echos_answer_id ON echos(answer_id);
CREATE INDEX idx_echos_refract_id ON echos(refract_id);
CREATE INDEX idx_echos_created_at ON echos(created_at DESC);

-- =====================================================
-- ENTANGLEMENTS (One-way Follow)
-- =====================================================

-- Entanglements: One-way follow (follower doesn't know they're being followed)
CREATE TABLE entanglements (
    id SERIAL PRIMARY KEY,
    follower_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,  -- User doing the entangling
    following_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,  -- User being entangled
    
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    -- User can entangle someone only once
    CONSTRAINT unique_entanglement UNIQUE(follower_id, following_id),
    
    -- Cannot entangle yourself
    CONSTRAINT no_self_entangle CHECK (follower_id != following_id)
);

CREATE INDEX idx_entanglements_follower_id ON entanglements(follower_id);
CREATE INDEX idx_entanglements_following_id ON entanglements(following_id);
CREATE INDEX idx_entanglements_created_at ON entanglements(created_at DESC);

-- =====================================================
-- COLLECTIONS SYSTEM (Saved Content)
-- =====================================================

-- Collections: User-created groups for saving content
CREATE TABLE collections (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    name VARCHAR(100) NOT NULL,
    description TEXT,
    
    -- Collection type
    is_default BOOLEAN DEFAULT FALSE,  -- For "Saved Posts" and "Saved Questions"
    collection_type VARCHAR(20) CHECK (collection_type IN ('posts', 'questions', 'mixed')),
    
    -- Privacy
    is_public BOOLEAN DEFAULT FALSE,  -- Can be shared
    slug VARCHAR(150) UNIQUE,  -- For public URL sharing
    
    -- Metadata
    item_count INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_collections_user_id ON collections(user_id);
CREATE INDEX idx_collections_slug ON collections(slug);
CREATE INDEX idx_collections_is_public ON collections(is_public);

-- Collection posts (saved posts in collections)
CREATE TABLE collection_posts (
    id SERIAL PRIMARY KEY,
    collection_id INTEGER NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
    post_id INTEGER NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    
    added_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    display_order INTEGER DEFAULT 0,  -- User can reorder
    
    CONSTRAINT unique_collection_post UNIQUE(collection_id, post_id)
);

CREATE INDEX idx_collection_posts_collection_id ON collection_posts(collection_id);
CREATE INDEX idx_collection_posts_post_id ON collection_posts(post_id);

-- Collection questions (saved questions in collections)
CREATE TABLE collection_questions (
    id SERIAL PRIMARY KEY,
    collection_id INTEGER NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
    question_id INTEGER NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
    
    added_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    display_order INTEGER DEFAULT 0,
    
    CONSTRAINT unique_collection_question UNIQUE(collection_id, question_id)
);

CREATE INDEX idx_collection_questions_collection_id ON collection_questions(collection_id);
CREATE INDEX idx_collection_questions_question_id ON collection_questions(question_id);

-- =====================================================
-- NOTIFICATIONS SYSTEM
-- =====================================================

-- Notifications for user interactions
CREATE TABLE notifications (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,  -- Recipient
    
    -- Notification type
    notification_type VARCHAR(20) NOT NULL CHECK (
        notification_type IN ('refract', 'post_comment', 'comment_reply', 'answer_reply')
    ),
    
    -- Actor (who triggered the notification)
    actor_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Related content (based on type)
    refract_id INTEGER REFERENCES refracts(id) ON DELETE CASCADE,
    post_id INTEGER REFERENCES posts(id) ON DELETE CASCADE,
    comment_id INTEGER REFERENCES comments(id) ON DELETE CASCADE,
    answer_id INTEGER REFERENCES answers(id) ON DELETE CASCADE,
    
    -- Read status
    is_read BOOLEAN DEFAULT FALSE,
    read_at TIMESTAMP WITH TIME ZONE,
    
    -- Metadata
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    -- Must have relevant IDs based on type
    CONSTRAINT valid_notification_data CHECK (
        (notification_type = 'refract' AND refract_id IS NOT NULL AND post_id IS NOT NULL) OR
        (notification_type = 'post_comment' AND comment_id IS NOT NULL AND post_id IS NOT NULL) OR
        (notification_type = 'comment_reply' AND comment_id IS NOT NULL) OR
        (notification_type = 'answer_reply' AND comment_id IS NOT NULL AND answer_id IS NOT NULL)
    )
);

CREATE INDEX idx_notifications_user_id ON notifications(user_id);
CREATE INDEX idx_notifications_actor_id ON notifications(actor_id);
CREATE INDEX idx_notifications_is_read ON notifications(is_read);
CREATE INDEX idx_notifications_created_at ON notifications(created_at DESC);
CREATE INDEX idx_notifications_type ON notifications(notification_type);

-- =====================================================
-- UTILITY TABLES
-- =====================================================


-- Media upload logs for debugging and abuse prevention
CREATE TABLE media_upload_logs (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- File info
    filename VARCHAR(500),
    file_size_bytes BIGINT NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    media_type VARCHAR(20),  -- 'image', 'video', 'file'
    
    -- Upload status
    status VARCHAR(50) NOT NULL,  -- 'success', 'failed', 'rejected'
    rejection_reason VARCHAR(200),
    
    -- Metadata
    uploaded_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    ip_address INET,
    user_agent TEXT
);

CREATE INDEX idx_upload_logs_user_id ON media_upload_logs(user_id);
CREATE INDEX idx_upload_logs_status ON media_upload_logs(status);
CREATE INDEX idx_upload_logs_uploaded_at ON media_upload_logs(uploaded_at DESC);

-- =====================================================
-- DATABASE VIEWS
-- =====================================================

-- User statistics view (aggregated stats per user)
CREATE VIEW user_stats AS
SELECT 
    u.id AS user_id,
    u.username,
    u.name,
    u.avatar_url,
    u.created_at AS member_since,
    
    -- Follower stats
    (SELECT COUNT(*) FROM entanglements WHERE following_id = u.id) AS follower_count,
    (SELECT COUNT(*) FROM entanglements WHERE follower_id = u.id) AS following_count,
    
    -- Content stats
    (SELECT COUNT(*) FROM posts WHERE user_id = u.id AND deleted_at IS NULL) AS post_count,
    (SELECT COUNT(*) FROM questions WHERE user_id = u.id AND deleted_at IS NULL) AS question_count,
    (SELECT COUNT(*) FROM answers WHERE user_id = u.id AND deleted_at IS NULL) AS answer_count,
    (SELECT COUNT(*) FROM refracts WHERE user_id = u.id AND deleted_at IS NULL) AS refract_count,
    
    -- Engagement stats
    (SELECT COALESCE(SUM(echo_count), 0) FROM posts WHERE user_id = u.id AND deleted_at IS NULL) AS total_post_echoes,
    (SELECT COALESCE(SUM(echo_count), 0) FROM questions WHERE user_id = u.id AND deleted_at IS NULL) AS total_question_echoes,
    (SELECT COALESCE(SUM(echo_count), 0) FROM answers WHERE user_id = u.id AND deleted_at IS NULL) AS total_answer_echoes,
    
    -- Question stats
    (SELECT COUNT(*) FROM questions WHERE user_id = u.id AND is_solved = TRUE AND deleted_at IS NULL) AS solved_questions_count
FROM users u
WHERE u.deleted_at IS NULL;

-- Trending content view (hot posts and questions)
CREATE VIEW trending_content AS
SELECT 
    'post' AS content_type,
    p.id AS content_id,
    p.title,
    p.user_id,
    p.created_at,
    p.view_count,
    p.echo_count,
    p.comment_count,
    p.refract_count,
    -- Trending score: recency + engagement
    (p.echo_count * 3 + p.comment_count * 2 + p.refract_count * 5 + p.view_count * 0.1) / 
    (EXTRACT(EPOCH FROM (NOW() - p.created_at)) / 3600 + 2) ^ 1.5 AS trend_score
FROM posts p
WHERE p.deleted_at IS NULL

UNION ALL

SELECT 
    'question' AS content_type,
    q.id AS content_id,
    q.title,
    q.user_id,
    q.created_at,
    q.view_count,
    q.echo_count,
    q.comment_count,
    q.answer_count AS refract_count,  -- Using answer_count as equivalent
    (q.echo_count * 3 + q.comment_count * 2 + q.answer_count * 4 + q.view_count * 0.1) / 
    (EXTRACT(EPOCH FROM (NOW() - q.created_at)) / 3600 + 2) ^ 1.5 AS trend_score
FROM questions q
WHERE q.deleted_at IS NULL

ORDER BY trend_score DESC;

-- =====================================================
-- DATABASE FUNCTIONS
-- =====================================================

-- Function to clean up old notifications (older than 30 days and read)
CREATE OR REPLACE FUNCTION cleanup_old_notifications() RETURNS INTEGER AS $$
DECLARE
    v_deleted_count INTEGER;
BEGIN
    DELETE FROM notifications
    WHERE is_read = TRUE 
    AND created_at < NOW() - INTERVAL '30 days';
    
    GET DIAGNOSTICS v_deleted_count = ROW_COUNT;
    
    RETURN v_deleted_count;
END;
$$ LANGUAGE plpgsql;

-- =====================================================
-- TRIGGERS - Counter Updates
-- =====================================================

-- Trigger: Increment post comment_count when comment added
CREATE OR REPLACE FUNCTION increment_post_comment_count() RETURNS TRIGGER AS $$
BEGIN
    IF NEW.post_id IS NOT NULL THEN
        UPDATE posts SET comment_count = comment_count + 1 
        WHERE id = NEW.post_id;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_increment_post_comment_count
AFTER INSERT ON comments
FOR EACH ROW
WHEN (NEW.post_id IS NOT NULL)
EXECUTE FUNCTION increment_post_comment_count();

-- Trigger: Decrement post comment_count when comment deleted
CREATE OR REPLACE FUNCTION decrement_post_comment_count() RETURNS TRIGGER AS $$
BEGIN
    IF NEW.deleted_at IS NOT NULL AND OLD.deleted_at IS NULL THEN
        IF NEW.post_id IS NOT NULL THEN
            UPDATE posts SET comment_count = GREATEST(comment_count - 1, 0)
            WHERE id = NEW.post_id;
        END IF;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_decrement_post_comment_count
AFTER UPDATE ON comments
FOR EACH ROW
EXECUTE FUNCTION decrement_post_comment_count();

-- Trigger: Increment question comment_count when comment added
CREATE OR REPLACE FUNCTION increment_question_comment_count() RETURNS TRIGGER AS $$
BEGIN
    IF NEW.question_id IS NOT NULL THEN
        UPDATE questions SET comment_count = comment_count + 1 
        WHERE id = NEW.question_id;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_increment_question_comment_count
AFTER INSERT ON comments
FOR EACH ROW
WHEN (NEW.question_id IS NOT NULL)
EXECUTE FUNCTION increment_question_comment_count();

-- Trigger: Decrement question comment_count when comment deleted
CREATE OR REPLACE FUNCTION decrement_question_comment_count() RETURNS TRIGGER AS $$
BEGIN
    IF NEW.deleted_at IS NOT NULL AND OLD.deleted_at IS NULL THEN
        IF NEW.question_id IS NOT NULL THEN
            UPDATE questions SET comment_count = GREATEST(comment_count - 1, 0)
            WHERE id = NEW.question_id;
        END IF;
    END IF;
    RETURN NEW;
END;
$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_decrement_question_comment_count
AFTER UPDATE ON comments
FOR EACH ROW
EXECUTE FUNCTION decrement_question_comment_count();

-- Trigger: Increment answer comment_count when comment added
CREATE OR REPLACE FUNCTION increment_answer_comment_count() RETURNS TRIGGER AS $
BEGIN
    IF NEW.answer_id IS NOT NULL THEN
        UPDATE answers SET comment_count = comment_count + 1 
        WHERE id = NEW.answer_id;
    END IF;
    RETURN NEW;
END;
$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_increment_answer_comment_count
AFTER INSERT ON comments
FOR EACH ROW
WHEN (NEW.answer_id IS NOT NULL)
EXECUTE FUNCTION increment_answer_comment_count();

-- Trigger: Decrement answer comment_count when comment deleted
CREATE OR REPLACE FUNCTION decrement_answer_comment_count() RETURNS TRIGGER AS $
BEGIN
    IF NEW.deleted_at IS NOT NULL AND OLD.deleted_at IS NULL THEN
        IF NEW.answer_id IS NOT NULL THEN
            UPDATE answers SET comment_count = GREATEST(comment_count - 1, 0)
            WHERE id = NEW.answer_id;
        END IF;
    END IF;
    RETURN NEW;
END;
$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_decrement_answer_comment_count
AFTER UPDATE ON comments
FOR EACH ROW
EXECUTE FUNCTION decrement_answer_comment_count();

-- Trigger: Increment comment reply_count when child comment added
CREATE OR REPLACE FUNCTION increment_comment_reply_count() RETURNS TRIGGER AS $
BEGIN
    IF NEW.parent_comment_id IS NOT NULL THEN
        UPDATE comments SET reply_count = reply_count + 1 
        WHERE id = NEW.parent_comment_id;
    END IF;
    RETURN NEW;
END;
$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_increment_comment_reply_count
AFTER INSERT ON comments
FOR EACH ROW
WHEN (NEW.parent_comment_id IS NOT NULL)
EXECUTE FUNCTION increment_comment_reply_count();

-- Trigger: Increment echo_count when echo added (posts)
CREATE OR REPLACE FUNCTION increment_post_echo_count() RETURNS TRIGGER AS $
BEGIN
    IF NEW.post_id IS NOT NULL THEN
        UPDATE posts SET echo_count = echo_count + 1 
        WHERE id = NEW.post_id;
    END IF;
    RETURN NEW;
END;
$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_increment_post_echo_count
AFTER INSERT ON echos
FOR EACH ROW
WHEN (NEW.post_id IS NOT NULL)
EXECUTE FUNCTION increment_post_echo_count();

-- Trigger: Increment echo_count when echo added (questions)
CREATE OR REPLACE FUNCTION increment_question_echo_count() RETURNS TRIGGER AS $
BEGIN
    IF NEW.question_id IS NOT NULL THEN
        UPDATE questions SET echo_count = echo_count + 1 
        WHERE id = NEW.question_id;
    END IF;
    RETURN NEW;
END;
$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_increment_question_echo_count
AFTER INSERT ON echos
FOR EACH ROW
WHEN (NEW.question_id IS NOT NULL)
EXECUTE FUNCTION increment_question_echo_count();

-- Trigger: Increment echo_count when echo added (answers)
CREATE OR REPLACE FUNCTION increment_answer_echo_count() RETURNS TRIGGER AS $
BEGIN
    IF NEW.answer_id IS NOT NULL THEN
        UPDATE answers SET echo_count = echo_count + 1 
        WHERE id = NEW.answer_id;
    END IF;
    RETURN NEW;
END;
$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_increment_answer_echo_count
AFTER INSERT ON echos
FOR EACH ROW
WHEN (NEW.answer_id IS NOT NULL)
EXECUTE FUNCTION increment_answer_echo_count();

-- Trigger: Increment echo_count when echo added (refracts)
CREATE OR REPLACE FUNCTION increment_refract_echo_count() RETURNS TRIGGER AS $
BEGIN
    IF NEW.refract_id IS NOT NULL THEN
        UPDATE refracts SET echo_count = echo_count + 1 
        WHERE id = NEW.refract_id;
    END IF;
    RETURN NEW;
END;
$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_increment_refract_echo_count
AFTER INSERT ON echos
FOR EACH ROW
WHEN (NEW.refract_id IS NOT NULL)
EXECUTE FUNCTION increment_refract_echo_count();

-- Trigger: Increment refract_count when refract added
CREATE OR REPLACE FUNCTION increment_post_refract_count() RETURNS TRIGGER AS $
BEGIN
    UPDATE posts SET refract_count = refract_count + 1 
    WHERE id = NEW.original_post_id;
    RETURN NEW;
END;
$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_increment_post_refract_count
AFTER INSERT ON refracts
FOR EACH ROW
EXECUTE FUNCTION increment_post_refract_count();

-- Trigger: Decrement refract_count when refract deleted
CREATE OR REPLACE FUNCTION decrement_post_refract_count() RETURNS TRIGGER AS $
BEGIN
    IF NEW.deleted_at IS NOT NULL AND OLD.deleted_at IS NULL THEN
        UPDATE posts SET refract_count = GREATEST(refract_count - 1, 0)
        WHERE id = NEW.original_post_id;
    END IF;
    RETURN NEW;
END;
$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_decrement_post_refract_count
AFTER UPDATE ON refracts
FOR EACH ROW
EXECUTE FUNCTION decrement_post_refract_count();

-- Trigger: Increment answer_count when answer added
CREATE OR REPLACE FUNCTION increment_question_answer_count() RETURNS TRIGGER AS $
BEGIN
    UPDATE questions SET answer_count = answer_count + 1 
    WHERE id = NEW.question_id;
    RETURN NEW;
END;
$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_increment_question_answer_count
AFTER INSERT ON answers
FOR EACH ROW
EXECUTE FUNCTION increment_question_answer_count();

-- Trigger: Decrement answer_count when answer deleted
CREATE OR REPLACE FUNCTION decrement_question_answer_count() RETURNS TRIGGER AS $
BEGIN
    IF NEW.deleted_at IS NOT NULL AND OLD.deleted_at IS NULL THEN
        UPDATE questions SET answer_count = GREATEST(answer_count - 1, 0)
        WHERE id = NEW.question_id;
    END IF;
    RETURN NEW;
END;
$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_decrement_question_answer_count
AFTER UPDATE ON answers
FOR EACH ROW
EXECUTE FUNCTION decrement_question_answer_count();

-- Trigger: Increment tag usage_count when tag used
CREATE OR REPLACE FUNCTION increment_tag_usage_count() RETURNS TRIGGER AS $
BEGIN
    UPDATE tags SET usage_count = usage_count + 1 
    WHERE id = NEW.tag_id;
    RETURN NEW;
END;
$ LANGUAGE plpgsql;

-- Create triggers for all tag junction tables
CREATE TRIGGER trigger_increment_tag_usage_post
AFTER INSERT ON post_tags
FOR EACH ROW
EXECUTE FUNCTION increment_tag_usage_count();

CREATE TRIGGER trigger_increment_tag_usage_question
AFTER INSERT ON question_tags
FOR EACH ROW
EXECUTE FUNCTION increment_tag_usage_count();

-- Trigger: Decrement tag usage_count when tag removed
CREATE OR REPLACE FUNCTION decrement_tag_usage_count() RETURNS TRIGGER AS $
BEGIN
    UPDATE tags SET usage_count = GREATEST(usage_count - 1, 0)
    WHERE id = OLD.tag_id;
    RETURN OLD;
END;
$ LANGUAGE plpgsql;

-- Create triggers for tag removal
CREATE TRIGGER trigger_decrement_tag_usage_post
AFTER DELETE ON post_tags
FOR EACH ROW
EXECUTE FUNCTION decrement_tag_usage_count();

CREATE TRIGGER trigger_decrement_tag_usage_question
AFTER DELETE ON question_tags
FOR EACH ROW
EXECUTE FUNCTION decrement_tag_usage_count();

-- Trigger: Increment tag follower_count
CREATE OR REPLACE FUNCTION increment_tag_follower_count() RETURNS TRIGGER AS $
BEGIN
    UPDATE tags SET follower_count = follower_count + 1 
    WHERE id = NEW.tag_id;
    RETURN NEW;
END;
$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_increment_tag_follower_count
AFTER INSERT ON user_tag_follows
FOR EACH ROW
EXECUTE FUNCTION increment_tag_follower_count();

-- Trigger: Decrement tag follower_count
CREATE OR REPLACE FUNCTION decrement_tag_follower_count() RETURNS TRIGGER AS $
BEGIN
    UPDATE tags SET follower_count = GREATEST(follower_count - 1, 0)
    WHERE id = OLD.tag_id;
    RETURN OLD;
END;
$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_decrement_tag_follower_count
AFTER DELETE ON user_tag_follows
FOR EACH ROW
EXECUTE FUNCTION decrement_tag_follower_count();

-- Trigger: Increment helpful_count on comments
CREATE OR REPLACE FUNCTION increment_comment_helpful_count() RETURNS TRIGGER AS $
BEGIN
    UPDATE comments SET helpful_count = helpful_count + 1 
    WHERE id = NEW.comment_id;
    RETURN NEW;
END;
$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_increment_comment_helpful_count
AFTER INSERT ON comment_helpful
FOR EACH ROW
EXECUTE FUNCTION increment_comment_helpful_count();

-- Trigger: Decrement helpful_count on comments
CREATE OR REPLACE FUNCTION decrement_comment_helpful_count() RETURNS TRIGGER AS $
BEGIN
    UPDATE comments SET helpful_count = GREATEST(helpful_count - 1, 0)
    WHERE id = OLD.comment_id;
    RETURN OLD;
END;
$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_decrement_comment_helpful_count
AFTER DELETE ON comment_helpful
FOR EACH ROW
EXECUTE FUNCTION decrement_comment_helpful_count();

-- Trigger: Update collection item_count
CREATE OR REPLACE FUNCTION update_collection_item_count() RETURNS TRIGGER AS $
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE collections SET item_count = item_count + 1 
        WHERE id = NEW.collection_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE collections SET item_count = GREATEST(item_count - 1, 0)
        WHERE id = OLD.collection_id;
    END IF;
    RETURN COALESCE(NEW, OLD);
END;
$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_collection_post_count
AFTER INSERT OR DELETE ON collection_posts
FOR EACH ROW
EXECUTE FUNCTION update_collection_item_count();

CREATE TRIGGER trigger_update_collection_question_count
AFTER INSERT OR DELETE ON collection_questions
FOR EACH ROW
EXECUTE FUNCTION update_collection_item_count();

