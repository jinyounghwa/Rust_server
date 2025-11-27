-- Create users table for authentication
CREATE TABLE users(
    id uuid NOT NULL PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    created_at timestamptz NOT NULL,
    updated_at timestamptz NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true
);

-- Index for email lookups during login (performance)
CREATE INDEX idx_users_email ON users(email);

-- Index for active users only
CREATE INDEX idx_users_active ON users(is_active) WHERE is_active = true;
