use chrono::Utc;
use reqwest;

use super::common::*;
use super::rss::*;

pub fn check_feed(feed: &MonitoredFeed) -> Option<FeedCheckResult> {
    match feed.kind {
        FeedType::Rss => check_rss(feed),
        FeedType::Atom => None,
    }
}

fn check_rss(feed: &MonitoredFeed) -> Option<FeedCheckResult> {
    debug!("Checking rss feed {:?}", feed.url);
    let now = Utc::now();

    let mut res = reqwest::get(&feed.url).unwrap(); //TODO ...
    let body = res.text().unwrap(); //TODO ...
    let feed = parse_rss_feed(body.as_str());
    let channel = feed.channels.first().unwrap(); //TODO ...
    let last_article = channel.items.first(); //TODO ...

    Some(FeedCheckResult {
        check_date: now.to_rfc3339(),
        title: channel.title.clone(),
        pub_date: channel.last_build_date.clone(),
        last_article_title: last_article.map(|art| art.title.clone()),
        last_article_guid: last_article.map(|art| art.guid.clone()),
        last_article_pub_date: last_article.map(|art| art.pub_date.clone()),
        last_article_hash: Some("None".to_string()),
    })
}
