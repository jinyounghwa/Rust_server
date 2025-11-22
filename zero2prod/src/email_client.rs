use crate::validators::is_valid_email;
use crate::error::EmailError;
use serde::Serialize;

#[derive(Clone)]
pub struct EmailClient {
    http_client: reqwest::Client,
    base_url: String,
    sender: ConfirmedSubscriber,
}

#[derive(Clone)]
pub struct ConfirmedSubscriber(String);

impl ConfirmedSubscriber {
    pub fn parse(s: String) -> Result<Self, EmailError> {
        let email = is_valid_email(&s)
            .map_err(|_| EmailError::InvalidRecipient(
                "Invalid sender email address".to_string()
            ))?;
        Ok(Self(email))
    }

    pub fn inner(&self) -> &str {
        &self.0
    }
}

#[derive(Serialize)]
pub struct SendEmailRequest {
    to: String,
    #[serde(rename = "Html")]
    html: String,
    #[serde(rename = "Subject")]
    subject: String,
}

impl EmailClient {
    pub fn new(
        base_url: String,
        sender: ConfirmedSubscriber,
        http_client: reqwest::Client,
    ) -> Self {
        Self {
            http_client,
            base_url,
            sender,
        }
    }

    pub async fn send_email(
        &self,
        recipient: &str,
        subject: &str,
        html_content: &str,
    ) -> Result<(), EmailError> {
        // Validate recipient email
        is_valid_email(recipient)
            .map_err(|_| EmailError::InvalidRecipient(
                format!("Invalid recipient email: {}", recipient)
            ))?;

        let url = format!("{}/email", self.base_url);
        let request = SendEmailRequest {
            to: recipient.to_string(),
            subject: subject.to_string(),
            html: html_content.to_string(),
        };

        let response = self.http_client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Failed to send email request: {}", e);
                EmailError::SendFailed(format!("HTTP request failed: {}", e))
            })?;

        response
            .error_for_status()
            .map_err(|e| {
                let status = e.status().map(|s| s.as_u16()).unwrap_or(0);
                if status == 503 || status == 502 || status == 504 {
                    tracing::error!("Email service unavailable: {}", e);
                    EmailError::ServiceUnavailable(
                        format!("Email service returned status {}", status)
                    )
                } else {
                    tracing::error!("Email service returned error: {}", e);
                    EmailError::SendFailed(format!("Email service error: {}", e))
                }
            })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confirmed_subscriber_parse_valid_email() {
        let email = "test@example.com".to_string();
        let subscriber = ConfirmedSubscriber::parse(email);
        assert!(subscriber.is_ok());
    }

    #[test]
    fn test_confirmed_subscriber_parse_invalid_email() {
        let email = "invalid-email".to_string();
        let subscriber = ConfirmedSubscriber::parse(email);
        assert!(subscriber.is_err());

        // Verify error type
        match subscriber {
            Err(EmailError::InvalidRecipient(_)) => (),
            _ => panic!("Expected InvalidRecipient error"),
        }
    }
}
