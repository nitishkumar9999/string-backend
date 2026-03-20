-- Add migration script here
-- Migration: Create rate_limits table
CREATE TABLE rate_limits (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL DEFAULT 0,  -- 0 for anonymous (IP-based)
    ip_address INET,  -- For anonymous users
    action_type VARCHAR(50) NOT NULL,
    window_type VARCHAR(20) NOT NULL,
    window_start TIMESTAMP WITH TIME ZONE NOT NULL,
    count INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    -- Either user_id or ip_address must be set
    CONSTRAINT check_user_or_ip 
        CHECK (user_id > 0 OR ip_address IS NOT NULL)
);

-- For authenticated users (user_id > 0, ip_address must be NULL)
CREATE UNIQUE INDEX unique_user_rate_limit 
    ON rate_limits(user_id, action_type, window_type, window_start)
    WHERE user_id > 0 AND ip_address IS NULL;

-- For anonymous users (ip_address IS NOT NULL, user_id should be 0 or low value)
CREATE UNIQUE INDEX unique_ip_rate_limit 
    ON rate_limits(ip_address, action_type, window_type, window_start)
    WHERE ip_address IS NOT NULL AND user_id = 0;

-- Additional indexes for fast lookups
CREATE INDEX idx_rate_limits_window_start 
    ON rate_limits(window_start);

-- Sessions table for user authentication
CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Security
    ip_address INET,
    user_agent TEXT,
    
    -- Session management
    last_used_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    rotated_from UUID,  -- For session rotation
    
    -- Lifecycle
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL
);

CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);
CREATE INDEX idx_sessions_last_used ON sessions(last_used_at DESC);

-- Cleanup function
CREATE OR REPLACE FUNCTION cleanup_old_rate_limits() RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM rate_limits
    WHERE window_start < NOW() - INTERVAL '24 hours';
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION rotate_session(
    p_old_session_id UUID,
    p_user_id INTEGER
) RETURNS UUID AS $$
DECLARE
    v_new_session_id UUID;
BEGIN
    -- Create new session
    INSERT INTO sessions (user_id, ip_address, user_agent, expires_at, rotated_from)
    SELECT user_id, ip_address, user_agent, 
           NOW() + INTERVAL '30 days', 
           id
    FROM sessions
    WHERE id = p_old_session_id AND user_id = p_user_id
    RETURNING id INTO v_new_session_id;
    
    -- Delete old session
    DELETE FROM sessions WHERE id = p_old_session_id;
    
    RETURN v_new_session_id;
END;
$$ LANGUAGE plpgsql;