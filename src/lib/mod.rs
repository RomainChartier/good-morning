pub mod common;
pub mod data;
pub mod import;
pub mod rss;

use std::collections::HashSet;

use common::*;
use import::*;

pub fn list_subscription(repo: &SubscriptionRepository) {
    info!("Listing subscription");

    for feed in repo.get_monitored_feeds().into_iter() {
        println!("Found {:?}", feed);
    }
}

pub fn import_subscriptions(repo: &SubscriptionRepository, file_path: &str) {
    info!("Importing {:?} to db", file_path);

    let csv_feeds = read_csv(file_path);

    let existing_feeds: HashSet<(String, FeedType)> = repo.get_monitored_feeds().into_iter()
        .map(|feed| (feed.url, feed.kind))
        .collect();

    for (url, kind) in csv_feeds.difference(&existing_feeds) {
        println!("Adding new feed {:?}", url);
        repo.add_sub(url, *kind);
    }
}

pub fn run(repo: &SubscriptionRepository, dry_run: bool) {
    println!("Run (dry: {:?})", dry_run);
}
