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
    pub pub_date: Option<String>,
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

#[derive(Debug, Fail)]
pub enum GoodMorningError {
    #[fail(display = "Xml Parse error")]
    XmlParse(#[cause] quick_xml::Error),
    #[fail(display = "Parse error")]
    Parse,
    #[fail(display = "Some mandatory information miss from the feed")]
    MissingFeedInfo,

    #[fail(display = "Http error")]
    HttpError(#[cause] reqwest::Error),
}

impl From<quick_xml::Error> for GoodMorningError {
    fn from(error: quick_xml::Error) -> GoodMorningError {
        GoodMorningError::XmlParse(error)
    }
}

impl From<reqwest::Error> for GoodMorningError {
    fn from(error: reqwest::Error) -> GoodMorningError {
        GoodMorningError::HttpError(error)
    }
}

pub trait SubscriptionRepository: Send {
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
