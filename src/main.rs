extern crate reqwest;

use structopt::StructOpt;
#[macro_use]
extern crate log;
extern crate simple_logger;

use rusqlite::{Connection, OpenFlags};
use std::str::FromStr;
use std::io::{self, Read};

mod lib;

use lib::common::*;
use lib::rss::*;
use lib::data::SQliteSubscriptionRepository;



use lib::rss::*;



#[derive(StructOpt)]
#[structopt(name = "good-morning")]
struct AppCli {
    #[structopt(long = "data-path")]
    data_path: Option<std::path::PathBuf>,

    #[structopt(subcommand)]
    cmd: AppCommand,
}

#[derive(StructOpt)]
#[structopt(name = "good-morning", about = "the stupid rss tracker")]
enum AppCommand {
    #[structopt(name = "import-sub")]
    Import {
        #[structopt(short = "p")]
        file_path: String,
    },

    #[structopt(name = "list-sub")]
    ListSub,

    #[structopt(name = "run")]
    Run {
        #[structopt(long = "dry-run")]
        dry_run: bool,
    },
}

fn main() -> Result<(), Box<std::error::Error>> {
    //env_logger::init();

    //println!("GET https://nickcraver.com/blog/feed.xml");

    //let mut res = reqwest::get("https://nickcraver.com/blog/feed.xml")?;
    

    //println!("Status: {}", res.status());
    //println!("Headers:\n{:?}", res.headers());

    // copy the response body directly to stdout
    //std::io::copy(&mut res, &mut std::io::stdout())?;

    //let body = res.text()?;

    //let feed = rss::parse_rss_feed(body.as_str());

    //println!("{:?}", feed);

    simple_logger::init()?;

    info!("starting up");

    let args = AppCli::from_args();

    let path = &args
        .data_path
        .and_then(|dp| dp.to_str().map(|p| p.to_string()));

    let file_path = match path {
        Some(path) => path,
        None => "./good-morning.db",
    };

    info!("using data from {:?}", file_path);

    let flags = OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE;
    let conn = Connection::open_with_flags(file_path, flags).unwrap();

    let repository = SQliteSubscriptionRepository::new(conn);

    repository.init();

    debug!("sql connection opened");

    match &args.cmd {
        AppCommand::ListSub => lib::list_subscription(&repository),
        AppCommand::Run { dry_run } => lib::run(&repository, *dry_run),
        AppCommand::Import { file_path } => lib::import_subscriptions(&repository, file_path),
    }

    // println!("press enter to finish.");
    // io::stdin().read_line(&mut buffer)?;

    Ok(())
}
