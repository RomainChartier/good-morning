extern crate reqwest;
use quick_xml::events::Event;
use quick_xml::Reader;

fn main() -> Result<(), Box<std::error::Error>> {
    //env_logger::init();

    //println!("GET https://nickcraver.com/blog/feed.xml");

    let mut res = reqwest::get("https://nickcraver.com/blog/feed.xml")?;

    //println!("Status: {}", res.status());
    //println!("Headers:\n{:?}", res.headers());

    // copy the response body directly to stdout
    //std::io::copy(&mut res, &mut std::io::stdout())?;

    let body = res.text()?;
    let mut reader = Reader::from_str(body.as_str());
    reader.trim_text(true);

    //let mut count = 0;
    let mut txt = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"title" => {
                    txt.push(
                        reader
                            .read_text(b"title", &mut Vec::new())
                            .expect("Cannot decode text value"),
                    );
                    
                }
                _ => (),
            },
            //Ok(Event::Text(e)) => txt.push(e.unescape_and_decode(&reader).unwrap()),
            Ok(Event::Eof) => break, // exits the loop when reaching end of file
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (), // There are several other `Event`s we do not consider here
        }

        // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
        buf.clear();
    }
    println!("{:?}", txt);
    //println!("\n\nDone.");
    Ok(())
}
