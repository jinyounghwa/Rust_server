-- Create refresh tokens table for JWT token rotation
CREATE TABLE refresh_tokens(
    id uuid NOT NULL PRIMARY KEY,
    user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL UNIQUE,
    expires_at timestamptz NOT NULL,
    created_at timestamptz NOT NULL,
    revoked_at timestamptz,
    is_revoked BOOLEAN NOT NULL DEFAULT false
);

-- Index for user's refresh tokens (lookup by user_id)
CREATE INDEX idx_refresh_tokens_user_id ON refresh_tokens(user_id);

-- Index for token lookups (lookup by token_hash)
CREATE INDEX idx_refresh_tokens_hash ON refresh_tokens(token_hash);

-- Index for active tokens only (optimization for validation queries)
CREATE INDEX idx_refresh_tokens_active ON refresh_tokens(user_id, expires_at)
WHERE is_revoked = false;
