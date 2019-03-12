use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use super::common::GoodMorningError;

#[derive(Debug)]
pub struct Feed {
    pub title: String,
    pub link: String,
    pub updated: String,
    pub entries: Vec<Entry>,
}

#[derive(Debug)]
pub struct Entry {
    pub title: String,
    pub link: String,
    pub guid: String,
    pub updated: String,
}

fn parse_entry<B: std::io::BufRead>(reader: &mut Reader<B>) -> Result<Entry, GoodMorningError> {
    let mut buf = Vec::new();

    let mut title: String = "".to_string();
    let mut link: String = "".to_string();
    let mut guid: String = "".to_string();
    let mut updated: String = "".to_string();

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"title" => title = reader.read_text(b"title", &mut buf)?,
                b"link" => link = extract_attr(b"href", &e, reader)?,
                b"id" => guid = reader.read_text(b"id", &mut buf)?,
                b"updated" => updated = reader.read_text(b"updated", &mut buf)?,
                _ => (),
            },
            Ok(Event::End(ref e)) => {
                if let b"entry" = e.name() {
                    break; // exits the loop when reaching end of current entry
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(GoodMorningError::XmlParse(e)),
            _ => (),
        }

        // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
        buf.clear();
    }

    if title.is_empty() || link.is_empty() || updated.is_empty() || guid.is_empty() {
        return Err(GoodMorningError::MissingFeedInfo);
    }

    Ok(Entry {
        title,
        link,
        guid,
        updated,
    })
}

pub fn parse_atom_feed(xml: &str) -> Result<Feed, GoodMorningError> {
    if xml.is_empty() {
        return Err(GoodMorningError::Parse);
    }

    let mut reader = Reader::from_str(xml);
    reader.trim_text(true).expand_empty_elements(true);

    let mut entries = Vec::new();

    let mut buf = Vec::new();

    let mut title: String = "".to_string();
    let mut link: String = "".to_string();
    let mut updated: String = "".to_string();

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"title" => title = reader.read_text(b"title", &mut buf)?,
                b"updated" => updated = reader.read_text(b"updated", &mut buf)?,
                b"link" => link = extract_attr(b"href", &e, &mut reader)?,
                b"entry" => entries.push(parse_entry(&mut reader)?),
                _ => (),
            },
            Ok(Event::Eof) => break,
            Err(e) => return Err(GoodMorningError::XmlParse(e)),
            _ => (),
        }

        // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
        buf.clear();
    }

    if title.is_empty() || link.is_empty() || updated.is_empty() || entries.is_empty() {
        return Err(GoodMorningError::MissingFeedInfo);
    }

    Ok(Feed {
        title,
        link,
        updated,
        entries,
    })
}

fn extract_attr<B: std::io::BufRead>(
    name: &[u8],
    event: &BytesStart,
    reader: &mut Reader<B>,
) -> Result<String, GoodMorningError> {
    for attr in event.attributes() {
        let attr = attr?;
        if attr.key == name {
            return Ok(attr.unescape_and_decode_value(reader)?);
        }
    }
    Err(GoodMorningError::Parse)
}

#[test]
pub fn should_parse_atom_sample_properly() {
    let atom_sample = r#"
        <?xml version="1.0" encoding="utf-8"?>
        <feed xmlns="http://www.w3.org/2005/Atom">

            <title>Example Feed</title>
            <link href="http://example.org/"/>
            <updated>2003-12-13T18:30:02Z</updated>
            <author>
            <name>John Doe</name>
            </author>
            <id>urn:uuid:60a76c80-d399-11d9-b93C-0003939e0af6</id>

            <entry>
                <title>Atom-Powered Robots Run Amok</title>
                <link href="http://example.org/2003/12/13/atom03"/>
                <id>urn:uuid:1225c695-cfb8-4ebb-aaaa-80da344efa6a</id>
                <updated>2003-12-13T18:30:02Z</updated>
                <summary>Some text.</summary>
            </entry>
        </feed>    
    "#;
    let feed = parse_atom_feed(atom_sample).unwrap();

    assert_eq!(feed.title, "Example Feed");
    assert_eq!(feed.link, "http://example.org/");
    assert_eq!(feed.updated, "2003-12-13T18:30:02Z");

    let entry = feed.entries.first().unwrap();

    assert_eq!(entry.title, "Atom-Powered Robots Run Amok");
    assert_eq!(entry.link, "http://example.org/2003/12/13/atom03");
    assert_eq!(entry.guid, "urn:uuid:1225c695-cfb8-4ebb-aaaa-80da344efa6a");
    assert_eq!(entry.updated, "2003-12-13T18:30:02Z");
}

#[test]
pub fn should_fail_on_invalid_xml() {
    let atom_sample = r#"
        <?xml version="1.0" encoding="utf-8"?>
        <feed xmlns="http://www.w3.org/2005/Atom">

            <title>Example Feed</title>
            <link href="http://example.org/"/>
            <updated>2003-12-13T18:30:02Z</updated>
            <author>
            <name>John Doe
            </author>
            <id>urn:uuid:60a76c80-d399-11d9-b93C-0003939e0af6</id>

            <entry>
                <title>Atom-Powered Robots Run Amok</title>
                <link href="http://example.org/2003/12/13/atom03"/>
                <id>urn:uuid:1225c695-cfb8-4ebb-aaaa-80da344efa6a</id>
                <updated>2003-12-13T18:30:02Z</updated>
                <summary>Some text.</summary>
            </entry>
        </feed>    
    "#;

    assert!(parse_atom_feed(atom_sample).is_err());
}

#[test]
pub fn should_fail_on_empty_xml() {
    let atom_sample = r#"  "#;

    assert!(parse_atom_feed(atom_sample).is_err());
}

#[test]
pub fn should_fail_on_empty_entries() {
    let atom_sample = r#"
        <?xml version="1.0" encoding="utf-8"?>
        <feed xmlns="http://www.w3.org/2005/Atom">

            <title>Example Feed</title>
            <link href="http://example.org/"/>
            <updated>2003-12-13T18:30:02Z</updated>
            <author>
            <name>John Doe
            </author>
            <id>urn:uuid:60a76c80-d399-11d9-b93C-0003939e0af6</id>

        </feed>    
    "#;

    assert!(parse_atom_feed(atom_sample).is_err());
}

#[test]
pub fn should_fail_on_missing_fields() {
    let atom_sample = r#"
        <?xml version="1.0" encoding="utf-8"?>
        <feed xmlns="http://www.w3.org/2005/Atom">
            <title>Example Feed</title>
            <link href="http://example.org/"/>
            <updated>2003-12-13T18:30:02Z</updated>
            <id>urn:uuid:60a76c80-d399-11d9-b93C-0003939e0af6</id>
            <entry>
                <title>Atom-Powered Robots Run Amok</title>
                <link href="http://example.org/2003/12/13/atom03"/>
                <id>urn:uuid:1225c695-cfb8-4ebb-aaaa-80da344efa6a</id>
                <updated></updated>
            </entry>
        </feed>    
    "#;

    assert!(parse_atom_feed(atom_sample).is_err());

    let atom_sample = r#"
        <?xml version="1.0" encoding="utf-8"?>
        <feed xmlns="http://www.w3.org/2005/Atom">
            <title>Example Feed</title>
            <link href=""/>
            <updated>2003-12-13T18:30:02Z</updated>
            <id>urn:uuid:60a76c80-d399-11d9-b93C-0003939e0af6</id>
            <entry>
                <title>Atom-Powered Robots Run Amok</title>
                <link href="http://example.org/2003/12/13/atom03"/>
                <id>urn:uuid:1225c695-cfb8-4ebb-aaaa-80da344efa6a</id>
                <updated>2003-12-13T18:30:02Z</updated>
            </entry>
        </feed>    
    "#;

    assert!(parse_atom_feed(atom_sample).is_err());
}
