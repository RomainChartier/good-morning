extern crate reqwest;


mod rss;

fn main() -> Result<(), Box<std::error::Error>> {
    //env_logger::init();

    //println!("GET https://nickcraver.com/blog/feed.xml");

    let mut res = reqwest::get("https://nickcraver.com/blog/feed.xml")?;
    

    //println!("Status: {}", res.status());
    //println!("Headers:\n{:?}", res.headers());

    // copy the response body directly to stdout
    //std::io::copy(&mut res, &mut std::io::stdout())?;

    let body = res.text()?;

    let feed = rss::parse_rss_feed(body.as_str());

    println!("{:?}", feed);
    //println!("\n\nDone.");
    Ok(())
}
