use chrono::{Duration, Utc};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct ConfirmationToken {
    token: String,
    subscriber_id: Uuid,
    created_at: chrono::DateTime<chrono::Utc>,
    expires_at: chrono::DateTime<chrono::Utc>,
}

impl ConfirmationToken {
    pub fn new(subscriber_id: Uuid) -> Self {
        let token = Uuid::new_v4().to_string();
        let created_at = Utc::now();
        let expires_at = created_at + Duration::days(1);

        Self {
            token,
            subscriber_id,
            created_at,
            expires_at,
        }
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn subscriber_id(&self) -> Uuid {
        self.subscriber_id
    }

    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.created_at
    }

    pub fn expires_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.expires_at
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confirmation_token_creation() {
        let subscriber_id = Uuid::new_v4();
        let token = ConfirmationToken::new(subscriber_id);

        assert_eq!(token.subscriber_id(), subscriber_id);
        assert!(!token.is_expired());
    }

    #[test]
    fn test_confirmation_token_not_immediately_expired() {
        let subscriber_id = Uuid::new_v4();
        let token = ConfirmationToken::new(subscriber_id);

        assert!(!token.is_expired());
    }
}
