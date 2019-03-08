extern crate reqwest;

use structopt::StructOpt;
#[macro_use]
extern crate log;
extern crate simple_logger;

use log::Level;

use rusqlite::{Connection, OpenFlags};

mod lib;

use lib::common::*;
use lib::data::SQliteSubscriptionRepository;

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
    simple_logger::init_with_level(Level::Error)?;

    info!("starting up");

    let args = AppCli::from_args();

    let path = &args
        .data_path
        .and_then(|dp| dp.to_str().map(|p| p.to_string()));

    let data_file_path = match path {
        Some(path) => path,
        None => "./good-morning.db",
    };

    info!("using data from {:?}", data_file_path);

    let flags = OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE;
    let conn = Connection::open_with_flags(data_file_path, flags).unwrap();

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
