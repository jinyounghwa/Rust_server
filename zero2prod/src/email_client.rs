use crate::validators::is_valid_email;
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
    pub fn parse(s: String) -> Result<Self, String> {
        let email = is_valid_email(&s).map_err(|e| format!("{:?}", e))?;
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
    ) -> Result<(), String> {
        let url = format!("{}/email", self.base_url);
        let request = SendEmailRequest {
            to: recipient.to_string(),
            subject: subject.to_string(),
            html: html_content.to_string(),
        };

        self.http_client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Failed to send email: {}", e);
                format!("Failed to send email: {}", e)
            })?
            .error_for_status()
            .map_err(|e| {
                tracing::error!("Email service returned error: {}", e);
                format!("Email service error: {}", e)
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
    }
}
