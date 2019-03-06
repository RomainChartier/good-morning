use std::str::FromStr;

use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use rusqlite::{Connection, Result, NO_PARAMS};

use super::common::*;

#[derive(Debug)]
pub struct SQliteSubscriptionRepository {
    conn: Connection,
}

impl SQliteSubscriptionRepository {
    pub fn new(conn: Connection) -> SQliteSubscriptionRepository {
        SQliteSubscriptionRepository { conn: conn }
    }

    fn execute(&self, sql: &str) -> Result<usize> {
        self.conn.execute(sql, NO_PARAMS)
    }
}

impl FromSql for FeedType {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_str().and_then(|s| match FromStr::from_str(s) {
            Ok(k) => Ok(k),
            Err(_) => Err(FromSqlError::InvalidType),
        })
    }
}

impl ToSql for FeedType {
    fn to_sql(&self) -> Result<ToSqlOutput> {
        Ok(ToSqlOutput::from(self.to_string()))
    }
}

impl SubscriptionRepository for SQliteSubscriptionRepository {
    fn init(&self) {
        debug!("Init database");
        self.execute(
            "CREATE TABLE IF NOT EXISTS subscription (
                id          INTEGER PRIMARY KEY,
                url         TEXT NOT NULL UNIQUE,
                kind        TEXT NOT NULL
            )",
        )
        .unwrap();

        self.execute(
            "CREATE TABLE IF NOT EXISTS subscription_check (
                id                    INTEGER PRIMARY KEY,
                subscription_id       INTEGER NOT NULL,

                check_date            TEXT NOT NULL,
                title                 TEXT NOT NULL,
                pub_date              TEXT NOT NULL,

                last_article_title    TEXT NOT NULL,
                last_article_guid     TEXT NOT NULL,
                last_article_pub_date TEXT NOT NULL,
                last_article_hash     TEXT NOT NULL,


                FOREIGN KEY(subscription_id) REFERENCES subscription(id)
            )",
        )
        .unwrap();
    }

    fn get_monitored_feeds(&self) -> Vec<MonitoredFeed> {
        debug!("Retrieving monitored feeds");
        let mut stmt = self
            .conn
            .prepare(
                "
                SELECT *
                FROM (
                    SELECT s.id as subscription_id,
                        url,
                        kind,
                        sc.id AS check_id, 
                        sc.check_date,
                        sc.title,
                        sc.pub_date,
                        last_article_title,
                        sc.last_article_guid,
                        sc.last_article_pub_date,
                        sc.last_article_hash,
                        ROW_NUMBER() OVER (PARTITION BY s.id ORDER BY check_date DESC) AS rownumber
                    FROM subscription AS s
                        LEFT OUTER JOIN  subscription_check AS sc ON sc.subscription_id = s.id
                    ) AS i
                WHERE rownumber = 1",
            )
            .unwrap();

        stmt.query_map(NO_PARAMS, |row| {
            let check_id: Option<u32> = row.get(3);

            let last_check = check_id.map(|_check_id| FeedCheckResult {
                check_date: row.get(4),
                title: row.get(5),
                pub_date: row.get(6),
                last_article_title: row.get(7),
                last_article_guid: row.get(8),
                last_article_pub_date: row.get(9),
                last_article_hash: row.get(10),
            });

            MonitoredFeed {
                id: row.get(0),
                url: row.get(1),
                kind: row.get(2),
                last_check: last_check,
            }
        })
        .unwrap()
        .map(|r| r.unwrap())
        .collect()
    }

    fn add_sub(&self, url: &str, kind: FeedType) {
        debug!("Adding feed {:?}", url);
        self.conn
            .execute(
                "INSERT INTO subscription (url, kind) VALUES (?1, ?2)",
                &[&url as &ToSql, &kind as &ToSql],
            )
            .unwrap();
    }

    fn add_check(&self, feed: &MonitoredFeed, check: &FeedCheckResult) {
        debug!("Adding check for feed {:?}", feed.id);
        self.conn
            .execute(
                "INSERT INTO subscription_check (subscription_id, check_date, title, pub_date, last_article_title, last_article_guid, last_article_pub_date, last_article_hash) 
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                //&[&url as &ToSql, &kind as &ToSql],
                NO_PARAMS
            )
            .unwrap();
    }
}
