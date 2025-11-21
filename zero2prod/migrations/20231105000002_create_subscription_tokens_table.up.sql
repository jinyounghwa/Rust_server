-- Create subscription_tokens table for email confirmation
CREATE TABLE subscription_tokens(
    subscription_token TEXT NOT NULL,
    subscriber_id uuid NOT NULL REFERENCES subscriptions (id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL,
    expires_at timestamptz NOT NULL,
    PRIMARY KEY (subscription_token)
);

-- Create index for faster lookups
CREATE INDEX idx_subscription_tokens_subscriber_id
ON subscription_tokens(subscriber_id);

-- Add status column to subscriptions table for confirmation tracking
ALTER TABLE subscriptions
ADD COLUMN status VARCHAR(20) NOT NULL DEFAULT 'pending';

-- Create index for confirmed subscribers
CREATE INDEX idx_subscriptions_status
ON subscriptions(status);
