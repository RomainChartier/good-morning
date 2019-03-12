use quick_xml::events::Event;
use quick_xml::Reader;

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

//TODO should return Result<Item, Error>
fn parse_item<B: std::io::BufRead>(reader: &mut Reader<B>) -> Item {
    let mut buf = Vec::new();

    let mut title: Option<String> = None;
    let mut pub_date: Option<String> = None;
    let mut link: Option<String> = None;
    let mut guid: Option<String> = None;

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"title" => title = reader.read_text(b"title", &mut buf).ok(),
                b"pubDate" => pub_date = reader.read_text(b"pubDate", &mut buf).ok(),
                b"link" => link = reader.read_text(b"link", &mut buf).ok(),
                b"guid" => guid = reader.read_text(b"guid", &mut buf).ok(),
                _ => (),
            },
            Ok(Event::End(ref e)) => {
                if let b"item" = e.name() {
                    break; // exits the loop when reaching end of current item
                }
            }
            Ok(Event::Eof) => break, // exits the loop when reaching end of file
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (),
        }

        // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
        buf.clear();
    }

    debug!("Found item {:?}", title);

    Item {
        title,
        pub_date,
        guid,
        link,
    }
}

//TODO should return Result<Channel, Error>
fn parse_channel<B: std::io::BufRead>(reader: &mut Reader<B>) -> Channel {
    let mut buf = Vec::new();

    let mut items = Vec::new();

    let mut title: String = "".to_string();
    let mut link: String = "".to_string();
    let mut build_date: Option<String> = None;

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"title" => {
                    title = reader
                        .read_text(b"title", &mut buf)
                        .expect("Cannot decode text value");
                }
                b"lastBuildDate" => build_date = reader.read_text(b"lastBuildDate", &mut buf).ok(),
                b"link" => {
                    link = reader
                        .read_text(b"link", &mut buf)
                        .expect("Cannot decode text value");
                }
                b"item" => items.push(parse_item(reader)),
                _ => (),
            },
            Ok(Event::End(ref e)) => {
                if let b"channel" = e.name() {
                    break; // exits the loop when reaching end of current channel
                }
            }
            Ok(Event::Eof) => break, // exits the loop when reaching end of file
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (),
        }

        // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
        buf.clear();
    }

    debug!("Found channel {}", title);

    Channel {
        title,
        link,
        last_build_date: build_date,
        items,
    }
}

//TODO should return Result<Feed, Error>
pub fn parse_rss_feed(xml: &str) -> Feed {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);

    let mut channels = Vec::new();

    let mut buf = Vec::new();

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                if let b"channel" = e.name() {
                    channels.push(parse_channel(&mut reader))
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (),
        }

        // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
        buf.clear();
    }

    Feed { channels }
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
                    <description>Before man travels to Mars, NASA hopes to design new engines that will let us fly through the Solar System more quickly.  The proposed VASIMR engine would do that.</description>
                    <pubDate>Tue, 27 May 2003 08:37:32 GMT</pubDate>
                    <guid>http://liftoff.msfc.nasa.gov/2003/05/27.html#item571</guid>
                </item>
                <item>
                    <title>Astronauts' Dirty Laundry</title>
                    <link>http://liftoff.msfc.nasa.gov/news/2003/news-laundry.asp</link>
                    <description>Compared to earlier spacecraft, the International Space Station has many luxuries, but laundry facilities are not one of them.  Instead, astronauts have other options.</description>
                    <pubDate>Tue, 20 May 2003 08:56:02 GMT</pubDate>
                    <guid>http://liftoff.msfc.nasa.gov/2003/05/20.html#item570</guid>
                </item>
            </channel>
        </rss>
    "#;

    let feed = parse_rss_feed(rss_sample);
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
