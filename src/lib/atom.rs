use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

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

//TODO should return Result<Item, Error>
fn parse_entry<B: std::io::BufRead>(reader: &mut Reader<B>) -> Entry {
    let mut buf = Vec::new();

    let mut title: String = "".to_string();
    let mut link: String = "".to_string();
    let mut guid: String = "".to_string();
    let mut updated: String = "".to_string();

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"title" => title = reader.read_text(b"title", &mut buf).unwrap(),
                b"link" => link = extract_attr(b"href", &e, reader),
                b"id" => guid = reader.read_text(b"id", &mut buf).unwrap(),
                b"updated" => updated = reader.read_text(b"updated", &mut buf).unwrap(),
                _ => (),
            },
            Ok(Event::End(ref e)) => {
                if let b"entry" = e.name() {
                    break; // exits the loop when reaching end of current entry
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (),
        }

        // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
        buf.clear();
    }

    debug!("Found entry {:?}", title);

    Entry {
        title,
        link,
        guid,
        updated,
    }
}

//TODO should return Result<Feed, Error>
pub fn parse_atom_feed(xml: &str) -> Feed {
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
                b"title" => title = reader.read_text(b"title", &mut buf).unwrap(),
                b"updated" => updated = reader.read_text(b"updated", &mut buf).unwrap(),
                b"link" => link = extract_attr(b"href", &e, &mut reader),
                b"entry" => entries.push(parse_entry(&mut reader)),
                name => println!("{:?}", String::from_utf8(name.to_vec())),
            },
            Ok(Event::Eof) => break,
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (),
        }

        // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
        buf.clear();
    }

    Feed {
        title,
        link,
        updated,
        entries,
    }
}

fn extract_attr<B: std::io::BufRead>(
    name: &[u8],
    event: &BytesStart,
    reader: &mut Reader<B>,
) -> String {
    event
        .attributes()
        .map(|a| a.unwrap())
        .find(|a| dbg!(a.key == name))
        .map(|a| a.unescape_and_decode_value(reader).unwrap())
        .unwrap()
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
    let feed = parse_atom_feed(atom_sample);

    assert_eq!(feed.title, "Example Feed");
    assert_eq!(feed.link, "http://example.org/");
    assert_eq!(feed.updated, "2003-12-13T18:30:02Z");

    let entry = feed.entries.first().unwrap();

    assert_eq!(entry.title, "Atom-Powered Robots Run Amok");
    assert_eq!(entry.link, "http://example.org/2003/12/13/atom03");
    assert_eq!(entry.guid, "urn:uuid:1225c695-cfb8-4ebb-aaaa-80da344efa6a");
    assert_eq!(entry.updated, "2003-12-13T18:30:02Z");
}
