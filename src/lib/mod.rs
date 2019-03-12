mod atom;
pub mod common;
pub mod data;
mod import;
mod rss;
mod syndication;

use crossbeam::crossbeam_channel::bounded;
use std::collections::HashSet;
use std::thread;

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

// TODO from cli
const PARALLEL_DOWNLOAD_MAX: usize = 4;

pub fn run(repo: &SubscriptionRepository, dry_run: bool) {
    info!("Run (dry: {:?})", dry_run);

    let (dl_chan_s, dl_chan_r) = bounded(PARALLEL_DOWNLOAD_MAX * 2);
    let (storage_chan_s, storage_chan_r) = bounded(PARALLEL_DOWNLOAD_MAX * 2);

    for i in 0..PARALLEL_DOWNLOAD_MAX {
        let my_dl = dl_chan_r.clone();
        let my_storage = storage_chan_s.clone();

        debug!("Starting dl thread {:?}", i);
        thread::spawn(move || loop {
            match my_dl.recv() {
                Ok(feed) => {
                    let check_result = check_feed(&feed);
                    my_storage.send((feed, check_result)).unwrap();
                }
                Err(_err) => {
                    drop(my_storage);
                    break;
                }
            }
        });
    }

    drop(storage_chan_s);

    debug!("Starting storage thread");
    thread::spawn(move || {});

    for feed in repo.get_monitored_feeds().into_iter() {
        dl_chan_s.send(feed).unwrap();
    }

    drop(dl_chan_s);

    while let Ok((feed, check_result)) = storage_chan_r.recv() {
        debug!("processing check result for {:?}", feed.url);
        process_feed(repo, &feed, &check_result);
    }
}

fn process_feed(
    repo: &SubscriptionRepository,
    feed: &MonitoredFeed,
    check_result: &Option<FeedCheckResult>,
) {
    let check_result = match check_result {
        None => {
            warn!("Feed without result {:?}", feed.url);
            return;
        }
        Some(r) => r,
    };
    let update_kind = get_update_kind(&feed, &check_result);

    //store
    match update_kind {
        None => (),
        Some(_) => repo.add_check(&feed, &check_result),
    }

    // Move to notify.rs
    match update_kind {
        None => println!("Already up to date {:?}", feed.url),
        Some(FeedUpdateKind::NewArticle) => println!("NewArticle for {:?}", feed.url),
        Some(FeedUpdateKind::FirstCheck) => println!("FirstCheck for {:?}", feed.url),
        Some(FeedUpdateKind::LastArticle) => println!("LastArticle updated for {:?}", feed.url),
        Some(FeedUpdateKind::Title) => println!("Title updated for {:?}", feed.url),
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
