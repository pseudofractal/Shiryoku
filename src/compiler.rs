use crate::models::{EmailDraft, UserIdentity};
use base64::{Engine as _, engine::general_purpose};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd, html};
use std::path::PathBuf;
use uuid::Uuid;

pub struct InlineImage {
  pub cid: String,
  pub path: PathBuf,
}

pub struct CompiledEmail {
  pub html_body: String,
  pub plain_body: String,
  pub inline_images: Vec<InlineImage>,
  pub attachments: Vec<PathBuf>,
}

pub fn compile(draft: &EmailDraft, identity: &UserIdentity, worker_url: &str) -> CompiledEmail {
  let (html_content, inline_images) = parse_markdown_with_images(&draft.body);
  let plain_body = strip_markdown(&draft.body);
  let plain_footer = generate_plain_footer(identity);

  let footer = generate_footer(identity);
  let tracker = generate_tracker(worker_url, &draft.recipient);

  let full_html = format!(
    r#"<!DOCTYPE html><html><head><style>body {{ font-family: Arial, sans-serif; color: #333; line-height: 1.6; }} a {{ color: {}; text-decoration: none; }} img {{ max-width: 100%; }}</style></head><body><div style="margin-bottom: 20px;">{}</div><br>{}{}</body></html>"#,
    identity.footer_color, html_content, footer, tracker
  );

  let full_plain = format!("{}\n\n--\n{}", plain_body, plain_footer);

  CompiledEmail {
    html_body: full_html,
    plain_body: full_plain,
    inline_images,
    attachments: draft.attachments.clone(),
  }
}

fn parse_markdown_with_images(markdown_input: &str) -> (String, Vec<InlineImage>) {
  let mut options = Options::empty();
  options.insert(Options::ENABLE_STRIKETHROUGH);
  options.insert(Options::ENABLE_TABLES);

  let parser = Parser::new_ext(markdown_input, options);
  let mut events = Vec::new();
  let mut inline_images = Vec::new();

  for event in parser {
    match event {
      Event::Start(Tag::Image {
        link_type,
        dest_url,
        title,
        id,
      }) => {
        let url_str = dest_url.to_string();
        if url_str.starts_with("http://") || url_str.starts_with("https://") {
          events.push(Event::Start(Tag::Image {
            link_type,
            dest_url,
            title,
            id,
          }));
        } else {
          let cid = Uuid::new_v4().to_string();
          let new_dest = format!("cid:{}", cid);

          inline_images.push(InlineImage {
            cid: cid.clone(),
            path: PathBuf::from(url_str),
          });
          events.push(Event::Start(Tag::Image {
            link_type,
            dest_url: new_dest.into(),
            title,
            id,
          }));
        }
      }
      _ => events.push(event),
    }
  }

  let mut html_output = String::new();
  html::push_html(&mut html_output, events.into_iter());
  (html_output, inline_images)
}

fn strip_markdown(markdown_input: &str) -> String {
  let parser = Parser::new(markdown_input);
  let mut plain = String::new();
  for event in parser {
    match event {
      Event::Text(t) => plain.push_str(&t),
      Event::Code(c) => plain.push_str(&c),
      Event::SoftBreak | Event::HardBreak => plain.push('\n'),
      Event::End(TagEnd::Paragraph) => plain.push_str("\n\n"),
      Event::End(TagEnd::Item) => plain.push('\n'),
      _ => {}
    }
  }
  plain
}

fn generate_footer(identity: &UserIdentity) -> String {
  let emails_html = identity
    .emails
    .iter()
    .map(|email| format!(r#"<a href="mailto:{}" style="color: {}; text-decoration: none;">{}</a>"#, email, identity.footer_color, email))
    .collect::<Vec<String>>()
    .join(" &bull; ");

  format!(
    r#"
    <div style="font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; border-left: 4px solid {color}; padding-left: 15px; margin-top: 25px; color: #4a4a4a;">
      
      <h3 style="margin: 0; color: #2d3436; font-size: 19px; font-weight: 700; line-height: 1.2;">{name}</h3>
      
      <div style="margin-top: 4px; font-size: 13px; color: {color}; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px;">{role}</div>
      
      <div style="margin-top: 2px; font-size: 12px; color: #7f8c8d;">
        {dept} &bull; {inst}
      </div>

      <div style="margin-top: 8px; font-size: 12px; color: #95a5a6; line-height: 1.4;">
        <span style="font-weight: 600;">Tel:</span> {phone}<br>
        <span style="font-weight: 600;">Email:</span> {emails}
      </div>

    </div>
    "#,
    color = identity.footer_color,
    name = identity.name,
    role = identity.role,
    dept = identity.department,
    inst = identity.institution,
    phone = identity.phone,
    emails = emails_html
  )
}

fn generate_plain_footer(identity: &UserIdentity) -> String {
  format!(
      "{}\n{}\n{} - {}\n\nPhone: {}\nEmail: {}",
      identity.name,
      identity.role,
      identity.department,
      identity.institution,
      identity.phone,
      identity.emails.join(" | ")
  )
}


fn generate_tracker(base_url: &str, recipient_email: &str) -> String {
  let encoded_id = general_purpose::URL_SAFE_NO_PAD.encode(recipient_email);
  format!(
    r#"<img src="{}/pixel.png?id={}" alt="" width="1" height="1" border="0" style="width:1px;height:1px;opacity:0.01;" />"#,
    base_url, encoded_id
  )
}
