use super::common::*;
use super::sendgrid::*;

//TODO: make types to handle config
pub fn notify_updates(
    api_token: &str,
    updates: Vec<(MonitoredFeed, FeedUpdateKind)>,
) -> Result<(), GoodMorningError> {
    let mut content: String = String::new();

    for (feed, update_kind) in updates {
        match update_kind {
            FeedUpdateKind::NewArticle => {
                content.push_str(&format!("NewArticle at {:?}", feed.url))
            }
            FeedUpdateKind::FirstCheck => {
                content.push_str(&format!("FirstCheck for {:?}", feed.url))
            }
            FeedUpdateKind::LastArticle => {
                content.push_str(&format!("LastArticle updated at {:?}", feed.url))
            }
            FeedUpdateKind::Title => content.push_str(&format!("Title updated for {:?}", feed.url)),
        }
    }

    if !content.is_empty() {
        let to = "hihihi@hahaha.com";
        let mail_request =
            MailRequest::new("New blog posts", to, "good-morning@chartios.com", &content);
        send_mail(api_token, &mail_request)?;
    }

    Ok(())
}