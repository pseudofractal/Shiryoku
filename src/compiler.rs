use crate::models::{EmailDraft, UserIdentity};
use base64::{Engine as _, engine::general_purpose};
use pulldown_cmark::{Options, Parser, html};

pub struct CompiledEmail {
    pub html_body: String,
}

pub fn compile(draft: &EmailDraft, identity: &UserIdentity, worker_url: &str) -> CompiledEmail {
    let html_content = parse_markdown(&draft.body);
    let footer = generate_footer(identity);
    let tracker = generate_tracker(worker_url, &draft.recipient);

    let full_html = format!(
        r#"
<!DOCTYPE html>
<html>
<head>
<style>
    body {{ font-family: Arial, sans-serif; color: #333; line-height: 1.6; }}
    a {{ color: #4A90E2; text-decoration: none; }}
</style>
</head>
<body>
    <div style="margin-bottom: 20px;">
        {}
    </div>
    <br>
    {}
    {}
</body>
</html>
        "#,
        html_content, footer, tracker
    );

    CompiledEmail {
        html_body: full_html,
    }
}

fn parse_markdown(markdown_input: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(markdown_input, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    html_output
}

fn generate_footer(identity: &UserIdentity) -> String {
    let emails_html = identity
        .emails
        .iter()
        .map(|email| format!("<a href=\"mailto:{}\">{}</a>", email, email))
        .collect::<Vec<String>>()
        .join(" || ");

    format!(
        r#"
<div style="font-family: sans-serif; border-left: 4px solid #4A90E2; padding-left: 12px; color: #333;">
    <h3 style="margin: 0; color: #2c3e50;">{}</h3>
    <p style="margin: 2px 0; font-size: 14px;">{}<br>{}</p>
    <p style="margin: 2px 0; font-size: 12px; color: #666;">{}</p>
    <br>
    <div style="font-size: 13px;">
        <span style="color: #4A90E2;">Phone:</span> {}<br>
        <span style="color: #4A90E2;">E-mail:</span> {}
    </div>
</div>
        "#,
        identity.name,
        identity.role,
        identity.department,
        identity.institution,
        identity.phone,
        emails_html
    )
}

fn generate_tracker(base_url: &str, recipient_email: &str) -> String {
    let encoded_id = general_purpose::URL_SAFE_NO_PAD.encode(recipient_email);
    let tracking_url = format!("{}/pixel.png?id={}", base_url, encoded_id);

    format!(
        r#"<img src="{}" alt="" width="1" height="1" border="0" style="display:none;" />"#,
        tracking_url
    )
}
