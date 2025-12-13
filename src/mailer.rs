use crate::compiler;
use crate::config::AppConfig;
use crate::models::EmailDraft;
use anyhow::{Context, Result};
use lettre::{
  AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
  message::{
    MultiPart, SinglePart,
    header::{self, Header},
  },
  transport::smtp::authentication::Credentials,
};
use std::fs;

pub async fn send_email(config: AppConfig, draft: EmailDraft) -> Result<()> {
  let compiled = compiler::compile(&draft, &config.identity, &config.worker_url);

  let sender_header = if config.identity.name.is_empty() {
      config.smtp_username.clone()
  } else {
      format!("\"{}\" <{}>", config.identity.name, config.smtp_username)
  };

  let email_builder = Message::builder()
    .from(
      sender_header
        .parse()
        .context("Invalid sender format (Name <email>)")?,
    )
    .to(draft.recipient.parse().context("Invalid recipient email")?)
    .subject(draft.subject);

  let alternative = MultiPart::alternative()
    .singlepart(SinglePart::plain(compiled.plain_body))
    .singlepart(SinglePart::html(compiled.html_body));

  let mut related = MultiPart::related().multipart(alternative);

  for img in compiled.inline_images {
    if let Ok(file_content) = fs::read(&img.path) {
      let filename = img
        .path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "image.png".to_string());

      let content_type = mime_guess::from_path(&img.path).first_or_octet_stream();

      let disposition_header = header::ContentDisposition::inline_with_name(&filename);

      let part = SinglePart::builder()
        .header(header::ContentType::parse(content_type.as_ref()).unwrap())
        .header(header::ContentId::parse(&format!("<{}>", img.cid)).unwrap())
        .header(disposition_header)
        .body(file_content);

      related = related.singlepart(part);
    }
  }

  let mut multipart = MultiPart::mixed().multipart(related);

  for path in compiled.attachments {
    if let Ok(file_content) = fs::read(&path) {
      let filename = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
      let content_type = mime_guess::from_path(&path).first_or_octet_stream();

      let part = SinglePart::builder()
        .header(header::ContentType::parse(content_type.as_ref()).unwrap())
        .header(header::ContentDisposition::attachment(&filename))
        .body(file_content);

      multipart = multipart.singlepart(part);
    }
  }

  let email = email_builder
    .multipart(multipart)
    .context("Failed to build email body")?;

  let creds = Credentials::new(config.smtp_username, config.smtp_app_password);
  let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.gmail.com")?
    .credentials(creds)
    .build();

  mailer
    .send(email)
    .await
    .context("SMTP transmission failed")?;

  Ok(())
}
