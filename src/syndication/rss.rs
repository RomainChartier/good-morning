use quick_xml::events::Event;
use quick_xml::Reader;

use crate::common::GoodMorningError;

#[derive(Debug)]
pub struct Feed {
    pub channels: Vec<Channel>,
}

#[derive(Debug)]
pub struct Channel {
    pub title: String,
    pub link: String,
    pub last_build_date: Option<String>,
    pub items: Vec<Item>,
}

#[derive(Debug)]
pub struct Item {
    pub title: Option<String>,
    pub pub_date: Option<String>,
    pub guid: Option<String>,
    pub link: Option<String>,
}

fn parse_item<B: std::io::BufRead>(reader: &mut Reader<B>) -> Result<Item, GoodMorningError> {
    let mut buf = Vec::new();

    let mut title: Option<String> = None;
    let mut pub_date: Option<String> = None;
    let mut link: Option<String> = None;
    let mut guid: Option<String> = None;

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"title" => title = Some(reader.read_text(b"title", &mut buf)?),
                b"pubDate" => pub_date = Some(reader.read_text(b"pubDate", &mut buf)?),
                b"link" => link = Some(reader.read_text(b"link", &mut buf)?),
                b"guid" => guid = Some(reader.read_text(b"guid", &mut buf)?),
                _ => (),
            },
            Ok(Event::End(ref e)) => {
                if b"item" == e.name() {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(GoodMorningError::XmlParse(e)),
            _ => (),
        }

        // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
        buf.clear();
    }

    Ok(Item {
        title,
        pub_date,
        guid,
        link,
    })
}

fn parse_channel<B: std::io::BufRead>(reader: &mut Reader<B>) -> Result<Channel, GoodMorningError> {
    let mut buf = Vec::new();
    let mut items = Vec::new();

    let mut title: String = "".to_string();
    let mut link: String = "".to_string();
    let mut build_date: Option<String> = None;

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"title" => title = reader.read_text(b"title", &mut buf)?,
                b"lastBuildDate" => {
                    build_date = Some(reader.read_text(b"lastBuildDate", &mut buf)?)
                }
                b"link" => link = reader.read_text(b"link", &mut buf)?,
                b"item" => {
                    if let Ok(item) = parse_item(reader) {
                        items.push(item)
                    }
                }
                _ => (),
            },
            Ok(Event::End(ref e)) => {
                if b"channel" == e.name() {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(GoodMorningError::XmlParse(e)),
            _ => (),
        }

        // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
        buf.clear();
    }

    if title.is_empty() || link.is_empty() || items.is_empty() {
        return Err(GoodMorningError::MissingFeedInfo);
    }

    Ok(Channel {
        title,
        link,
        last_build_date: build_date,
        items,
    })
}

pub fn parse_rss_feed(xml: &str) -> Result<Feed, GoodMorningError> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);

    let mut channels = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                if b"channel" == e.name() {
                    if let Ok(channel) = parse_channel(&mut reader) {
                        channels.push(channel)
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(GoodMorningError::XmlParse(e)),
            _ => (),
        }

        // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
        buf.clear();
    }

    if channels.is_empty() {
        return Err(GoodMorningError::MissingFeedInfo);
    }

    Ok(Feed { channels })
}

#[test]
pub fn should_parse_rss_sample_properly() {
    let rss_sample = r#"
        <?xml version="1.0"?>
        <rss version="2.0">
            <channel>
                <title>Liftoff News</title>
                <link>http://liftoff.msfc.nasa.gov/</link>
                <description>Liftoff to Space Exploration.</description>
                <language>en-us</language>
                <pubDate>Tue, 10 Jun 2003 04:00:00 GMT</pubDate>
                <lastBuildDate>Tue, 10 Jun 2003 09:41:01 GMT</lastBuildDate>
                <docs>http://blogs.law.harvard.edu/tech/rss</docs>
                <generator>Weblog Editor 2.0</generator>
                <managingEditor>editor@example.com</managingEditor>
                <webMaster>webmaster@example.com</webMaster>
                <item>
                    <title>The Engine That Does More</title>
                    <link>http://liftoff.msfc.nasa.gov/news/2003/news-VASIMR.asp</link>
                    <description>Before man travels to Mars, N...would do that.</description>
                    <pubDate>Tue, 27 May 2003 08:37:32 GMT</pubDate>
                    <guid>http://liftoff.msfc.nasa.gov/2003/05/27.html#item571</guid>
                </item>
                <item>
                    <title>Astronauts' Dirty Laundry</title>
                    <link>http://liftoff.msfc.nasa.gov/news/2003/news-laundry.asp</link>
                    <description>Compared to earlier spacecraf...ther options.</description>
                    <pubDate>Tue, 20 May 2003 08:56:02 GMT</pubDate>
                    <guid>http://liftoff.msfc.nasa.gov/2003/05/20.html#item570</guid>
                </item>
            </channel>
        </rss>
    "#;

    let feed = parse_rss_feed(rss_sample).unwrap();
    let channel = feed.channels.first().unwrap();

    assert_eq!(channel.title, "Liftoff News");
    assert_eq!(channel.link, "http://liftoff.msfc.nasa.gov/");
    assert_eq!(
        channel.last_build_date,
        Some("Tue, 10 Jun 2003 09:41:01 GMT".to_string())
    );

    let item = &channel.items[0];

    assert_eq!(item.title, Some("The Engine That Does More".to_string()));
    assert_eq!(
        item.link,
        Some("http://liftoff.msfc.nasa.gov/news/2003/news-VASIMR.asp".to_string())
    );
    assert_eq!(
        item.guid,
        Some("http://liftoff.msfc.nasa.gov/2003/05/27.html#item571".to_string())
    );
    assert_eq!(
        item.pub_date,
        Some("Tue, 27 May 2003 08:37:32 GMT".to_string())
    );

    let item = &channel.items[1];

    assert_eq!(item.title, Some("Astronauts' Dirty Laundry".to_string()));
    assert_eq!(
        item.link,
        Some("http://liftoff.msfc.nasa.gov/news/2003/news-laundry.asp".to_string())
    );
    assert_eq!(
        item.guid,
        Some("http://liftoff.msfc.nasa.gov/2003/05/20.html#item570".to_string())
    );
    assert_eq!(
        item.pub_date,
        Some("Tue, 20 May 2003 08:56:02 GMT".to_string())
    );
}

#[test]
pub fn should_fail_on_invalid_xml() {
    let rss_sample = r#"
        <?xml version="1.0"?>
        <rss version="2.0">
            <channel>
                <title>Liftoff News</title>
                <link>http://liftoff.msfc.nasa.gov/</link
                <pubDate>Tue, 10 Jun 2003 04:00:00 GMT</pubDate>
                <lastBuildDate>Tue, 10 Jun 2003 09:41:01 GMT</lastBuildDate>
               
                <item>
                    <title>Astronauts' Dirty Laundry</title>
                    <link>http://liftoff.msfc.nasa.gov/news/2003/news-laundry.asp</link>
                    <pubDate>Tue, 20 May 2003 08:56:02 GMT</pubDate>
                    <guid>http://liftoff.msfc.nasa.gov/2003/05/20.html#item570</guid>
                </item>
            </channel>
        </rss>
    "#;

    assert!(parse_rss_feed(rss_sample).is_err());
}

#[test]
pub fn should_fail_on_empty_xml() {
    let rss_sample = r#"  "#;

    assert!(parse_rss_feed(rss_sample).is_err());
}

#[test]
pub fn should_fail_on_empty_items() {
    let rss_sample = r#"
        <?xml version="1.0"?>
        <rss version="2.0">
            <channel>
                <title>Liftoff News</title>
                <link>http://liftoff.msfc.nasa.gov/</link>
                <pubDate>Tue, 10 Jun 2003 04:00:00 GMT</pubDate>
                <lastBuildDate>Tue, 10 Jun 2003 09:41:01 GMT</lastBuildDate>
               
            </channel>
        </rss>
    "#;

    assert!(parse_rss_feed(rss_sample).is_err());
}

#[test]
pub fn should_fail_on_missing_fields() {
    let rss_sample = r#"
        <?xml version="1.0"?>
        <rss version="2.0">
            <channel>
                <link>http://liftoff.msfc.nasa.gov/</link
                <pubDate>Tue, 10 Jun 2003 04:00:00 GMT</pubDate>
                <lastBuildDate>Tue, 10 Jun 2003 09:41:01 GMT</lastBuildDate>
               
                <item>
                    <title>Astronauts' Dirty Laundry</title>
                    <link>http://liftoff.msfc.nasa.gov/news/2003/news-laundry.asp</link>
                    <pubDate>Tue, 20 May 2003 08:56:02 GMT</pubDate>
                    <guid>http://liftoff.msfc.nasa.gov/2003/05/20.html#item570</guid>
                </item>
            </channel>
        </rss>
    "#;

    assert!(parse_rss_feed(rss_sample).is_err());
}
