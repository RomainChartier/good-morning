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
    pub last_build_date: String,
    pub items: Vec<Item>,
}

#[derive(Debug)]
pub struct Item {
    pub title: String,
    pub pub_date: String,
    pub guid: String,
    pub link: String
}

fn parse_item<B: std::io::BufRead>(reader: &mut Reader<B>) -> Item {
    let mut buf = Vec::new();

    let mut title: String = "".to_string();
    let mut pub_date: String = "".to_string();
    let mut link: String = "".to_string();
    let mut guid: String = "".to_string();

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"title" => {
                    title = reader
                        .read_text(b"title", &mut buf)
                        .expect("Cannot decode text value");
                }
                b"pubDate" => {
                    pub_date = reader
                        .read_text(b"pubDate", &mut buf)
                        .expect("Cannot decode text value");
                }
                b"link" => {
                    link = reader
                        .read_text(b"link", &mut buf)
                        .expect("Cannot decode text value");
                }
                b"guid" => {
                    guid = reader
                        .read_text(b"guid", &mut buf)
                        .expect("Cannot decode text value");
                }
                _ => (),
            },
            Ok(Event::End(ref e)) => match e.name() {
                b"item" => break,
                _ => (),
            },
            Ok(Event::Eof) => break, // exits the loop when reaching end of file
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (), // There are several other `Event`s we do not consider here
        }

        // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
        buf.clear();
    }

    debug!("Found item {}", title);

    Item {
        title,
        pub_date,
        guid,
        link,
    }
}

fn parse_channel<B: std::io::BufRead>(reader: &mut Reader<B>) -> Channel {
    let mut buf = Vec::new();

    let mut items = Vec::new();
    let mut link: String = "".to_string();
    let mut title: String = "".to_string();
    let mut last_build_date: String = "".to_string();

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"title" => {
                    title = reader
                        .read_text(b"title", &mut buf)
                        .expect("Cannot decode text value");
                }
                b"lastBuildDate" => {
                    last_build_date = reader
                        .read_text(b"lastBuildDate", &mut buf)
                        .expect("Cannot decode text value");
                }
                b"link" => {
                    link = reader
                        .read_text(b"link", &mut buf)
                        .expect("Cannot decode text value");
                }
                b"item" => items.push(parse_item(reader)),
                _ => (),
            },
            Ok(Event::End(ref e)) => match e.name() {
                b"channel" => break,
                _ => (),
            },
            Ok(Event::Eof) => break, // exits the loop when reaching end of file
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (), // There are several other `Event`s we do not consider here
        }

        // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
        buf.clear();
    }

    debug!("Found channel {}", title);

    Channel {
        title,
        link,
        last_build_date,
        items,
    }
}

pub fn parse_rss_feed(xml: &str) -> Feed {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);

    let mut channels = Vec::new();

    let mut buf = Vec::new();

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"channel" => channels.push(parse_channel(&mut reader)),
                _ => (),
            },
            Ok(Event::Eof) => break, // exits the loop when reaching end of file
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (), // There are several other `Event`s we do not consider here
        }

        // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
        buf.clear();
    }

    Feed { channels }
}
