use std::str::FromStr;
use std::string::ToString;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum FeedType {
    Rss,
    Atom,
}

#[derive(Clone, Debug)]
pub struct FeedCheckResult {
    pub check_date: String,
    pub title: String,
    pub pub_date: String,
    pub last_article_title: Option<String>,
    pub last_article_guid: Option<String>,
    pub last_article_pub_date: Option<String>,
    pub last_article_hash: Option<String>,
}

#[derive(Clone, Debug)]
pub struct MonitoredFeed {
    pub id: u32,
    pub url: String,
    pub kind: FeedType,
    pub last_check: Option<FeedCheckResult>,
}

#[derive(Copy, Clone, Debug)]
pub enum FeedUpdateKind {
    FirstCheck,
    NewArticle,
    Title,
    LastArticle,
}

pub trait SubscriptionRepository {
    fn init(&self);
    fn get_monitored_feeds(&self) -> Vec<MonitoredFeed>;
    fn add_sub(&self, url: &str, kind: FeedType);

    fn add_check(&self, feed: &MonitoredFeed, check: &FeedCheckResult);
}

impl FromStr for FeedType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "rss" => Ok(FeedType::Rss),
            "atom" => Ok(FeedType::Atom),
            _ => Err(format!("Unknown FeedType {}", s).to_string()),
        }
    }
}

impl ToString for FeedType {
    fn to_string(&self) -> String {
        match self {
            FeedType::Rss => "rss".to_string(),
            FeedType::Atom => "atom".to_string(),
        }
    }
}
