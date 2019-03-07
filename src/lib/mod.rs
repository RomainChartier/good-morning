pub mod common;
pub mod data;
pub mod import;
mod rss;
pub mod syndication;

use std::collections::HashSet;

use common::*;
use import::*;
use syndication::*;

pub fn list_subscription(repo: &SubscriptionRepository) {
    info!("Listing subscription");

    for feed in repo.get_monitored_feeds().into_iter() {
        println!("Found {:?}", feed);
    }
}

pub fn import_subscriptions(repo: &SubscriptionRepository, file_path: &str) {
    info!("Importing {:?} to db", file_path);

    let csv_feeds = read_csv(file_path);

    let existing_feeds: HashSet<(String, FeedType)> = repo
        .get_monitored_feeds()
        .into_iter()
        .map(|feed| (feed.url, feed.kind))
        .collect();

    for (url, kind) in csv_feeds.difference(&existing_feeds) {
        println!("Adding new feed {:?}", url);
        repo.add_sub(url, *kind);
    }
}

pub fn run(repo: &SubscriptionRepository, dry_run: bool) {
    info!("Run (dry: {:?})", dry_run);

    for feed in repo.get_monitored_feeds().into_iter() {
        let check_result = check_feed(&feed);

        let check_result = match check_result {
            None => {
                warn!("Feed without result {:?}", feed.url);
                continue;
            }
            Some(r) => r,
        };

        repo.add_check(&feed, &check_result);

        // Move to notify.rs
        match get_update_kind(&feed, &check_result) {
            None => println!("Already up to date {:?}", feed.url),
            Some(FeedUpdateKind::NewArticle) => println!("NewArticle for {:?}", feed.url),
            Some(FeedUpdateKind::FirstCheck) => println!("FirstCheck for {:?}", feed.url),
            Some(FeedUpdateKind::LastArticle) => println!("LastArticle updated for {:?}", feed.url),
            Some(FeedUpdateKind::Title) => println!("Title updated for {:?}", feed.url),
        }
    }
}

fn get_update_kind(feed: &MonitoredFeed, check_result: &FeedCheckResult) -> Option<FeedUpdateKind> {
    match feed.last_check {
        None => Some(FeedUpdateKind::FirstCheck),
        Some(ref last_check) if last_check.title != check_result.title => {
            Some(FeedUpdateKind::Title)
        }
        Some(ref last_check) if last_check.last_article_guid != check_result.last_article_guid => {
            Some(FeedUpdateKind::NewArticle)
        }
        Some(ref last_check)
            if last_check.last_article_pub_date != check_result.last_article_pub_date =>
        {
            Some(FeedUpdateKind::LastArticle)
        }
        _ => None,
    }
}
