mod atom;
mod rss;

use chrono::Utc;
use reqwest;

use crate::common::*;
use atom::parse_atom_feed;
use rss::parse_rss_feed;

pub fn check_feed(feed: &MonitoredFeed) -> Option<FeedCheckResult> {
    debug!("Checking {:?}", feed.url);

    let mut res = match reqwest::get(&feed.url) {
        Ok(req_result) => req_result,
        Err(err) => {
            warn!("Error happened while requesting {:?} ({:?})", feed.url, err);
            return None;
        }
    };

    let body = match res.text() {
        Ok(b) => b,
        Err(err) => {
            warn!(
                "Error happened while opening body of {:?} ({:?})",
                feed.url, err
            );
            return None;
        }
    };

    match feed.kind {
        FeedType::Rss => check_rss(feed, body.as_str()),
        FeedType::Atom => check_atom(feed, body.as_str()),
    }
}

fn check_rss(feed: &MonitoredFeed, body: &str) -> Option<FeedCheckResult> {
    debug!("Parsing rss feed {:?}", feed.url);

    let feed = match parse_rss_feed(body).ok() {
        None => return None,
        Some(f) => f,
    };

    let channel = feed.channels.first().unwrap(); //TODO ...
    let last_article = channel.items.first(); //TODO ...

    Some(FeedCheckResult {
        check_date: Utc::now().to_rfc3339(),
        title: channel.title.clone(),
        pub_date: channel.last_build_date.clone(),
        last_article_title: last_article.and_then(|art| art.title.clone()),
        last_article_guid: last_article.and_then(|art| art.guid.clone()),
        last_article_pub_date: last_article.and_then(|art| art.pub_date.clone()),
        last_article_hash: Some("None".to_string()),
    })
}

fn check_atom(feed: &MonitoredFeed, body: &str) -> Option<FeedCheckResult> {
    debug!("Parsing atom feed {:?}", feed.url);

    let feed = match parse_atom_feed(body).ok() {
        None => return None,
        Some(f) => f,
    };

    let last_article = feed.entries.first(); //TODO ...

    Some(FeedCheckResult {
        check_date: Utc::now().to_rfc3339(),
        title: feed.title.clone(),
        pub_date: Some(feed.updated.clone()),
        last_article_title: last_article.map(|art| art.title.clone()),
        last_article_guid: last_article.map(|art| art.guid.clone()),
        last_article_pub_date: last_article.map(|art| art.updated.clone()),
        last_article_hash: Some("None".to_string()),
    })
}
