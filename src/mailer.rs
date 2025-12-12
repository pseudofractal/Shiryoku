use crate::compiler;
use crate::config::AppConfig;
use crate::models::EmailDraft;
use anyhow::{Context, Result};
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{MultiPart, SinglePart, header::ContentType},
    transport::smtp::authentication::Credentials,
};

pub async fn send_email(config: AppConfig, draft: EmailDraft) -> Result<()> {
    let compiled = compiler::compile(&draft, &config.identity, &config.worker_url);
    let email_builder = Message::builder()
        .from(
            config
                .smtp_username
                .parse()
                .context("Invalid sender email in config")?,
        )
        .to(draft.recipient.parse().context("Invalid recipient email")?)
        .subject(draft.subject);

    let body = MultiPart::mixed().singlepart(
        SinglePart::builder()
            .header(ContentType::TEXT_HTML)
            .body(compiled.html_body),
    );

    let email = email_builder
        .multipart(body)
        .context("Failed to build email body")?;

    let creds = Credentials::new(config.smtp_username, config.smtp_app_password);

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.gmail.com")
        .context("Failed to create SMTP transport")?
        .credentials(creds)
        .build();

    mailer
        .send(email)
        .await
        .context("SMTP transmission failed")?;

    Ok(())
}
