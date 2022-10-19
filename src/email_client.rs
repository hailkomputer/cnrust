use crate::domain::SubscriberEmail;
use reqwest::Client;
use secrecy::{ExposeSecret, Secret};

pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: SubscriberEmail,
    authorization_token: Secret<String>,
}

impl EmailClient {
    pub fn new(
        base_url: String,
        sender: SubscriberEmail,
        authoriation_token: Secret<String>,
    ) -> Self {
        Self {
            http_client: Client::new(),
            base_url,
            sender,
            authorization_token: authoriation_token,
        }
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        content: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/messages", self.base_url);
        let request_body = SendEmailRequest {
            from: self.sender.as_ref().to_owned(),
            to: recipient.as_ref().to_owned(),
            subject: subject.to_owned(),
            body: content.to_owned(),
        };
        self.http_client
            .post(&url)
            .json(&request_body)
            .basic_auth("api", Some(self.authorization_token.expose_secret()))
            .send()
            .await?;
        Ok(())
    }
}

#[derive(serde::Serialize)]
struct SendEmailRequest {
    from: String,
    to: String,
    subject: String,
    body: String,
}

#[cfg(test)]
mod tests {

    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::Faker;
    use fake::{faker::internet::en::SafeEmail, Fake};
    use secrecy::Secret;
    use wiremock::matchers::{header, header_exists, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        let mock_server = MockServer::start().await;
        let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let email_client = EmailClient::new(mock_server.uri(), sender, Secret::new(Faker.fake()));

        Mock::given(header_exists("Authorization"))
            .and(header("Content-Type", "application/json"))
            .and(path("/messages"))
            .and(method("POST"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        let _ = email_client
            .send_email(subscriber_email, &subject, &content)
            .await;
    }
}
