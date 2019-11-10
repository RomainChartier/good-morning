mod sendgrid;

use crate::common::*;
use sendgrid::{send_mail, MailRequest};
use std::fmt::Write;

//TODO: make types to handle config
pub fn notify_updates(
    config: &Config,
    updates: Vec<(MonitoredFeed, FeedUpdateKind)>,
) -> Result<(), GoodMorningError> {
    let mut content: String = String::new();

    for (feed, update_kind) in updates {
        match update_kind {
            FeedUpdateKind::NewArticle => {
                writeln!(content, "NewArticle at {}", feed.url).expect("Formatting error")
            }
            FeedUpdateKind::FirstCheck => {
                writeln!(content, "FirstCheck for {}", feed.url).expect("Formatting error")
            }
            FeedUpdateKind::LastArticle => {
                writeln!(content, "LastArticle updated for {}", feed.url).expect("Formatting error")
            }
            FeedUpdateKind::Title => {
                writeln!(content, "Title updated for {}", feed.url).expect("Formatting error")
            }
        }
    }

    if !content.is_empty() {
        match &config.report_type {
            ReportType::Email => {
                let to = &config.mail_to;
                let mail_request =
                    MailRequest::new("New blog posts", to, "good-morning@chartios.com", &content);
                send_mail(&config.sendgrid_token, &mail_request)?;
            }
            ReportType::Stdout => println!("{}", content),
        }
    }

    Ok(())
}
