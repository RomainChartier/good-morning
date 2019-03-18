use log::Level;
use simple_logger;
use std::fs::File;
use std::io::Read;
use structopt::StructOpt;
use toml;

#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

mod lib;

use lib::common::{Config, GoodMorningError, ReportType, SubscriptionRepository};
use lib::data::SQliteSubscriptionRepository;

fn main() -> Result<(), Box<std::error::Error>> {
    simple_logger::init_with_level(Level::Info)?;

    info!("starting up");

    let args = AppCli::from_args();

    let path = &args
        .data_path
        .and_then(|dp| dp.to_str().map(|p| p.to_string()));

    let data_file_path = match path {
        Some(path) => path,
        None => "./good-morning.db",
    };

    let config = load_config("./good-morning.config.toml").expect("config loading failed...");

    info!("using data from {:?}", data_file_path);

    match &config.report_type {
        ReportType::Email => info!("sending mail to {:?}", config.mail_to),
        ReportType::Stdout => info!("reporting to stdout"),
    }

    let repo = SQliteSubscriptionRepository::new(data_file_path);

    repo.init();

    match &args.cmd {
        AppCommand::ListSub => lib::list_subscription(&repo),
        AppCommand::Run { dry_run } => lib::run(&repo, *dry_run, &config).expect("run failed..."), //TODO
        AppCommand::Import { file_path } => lib::import_subscriptions(&repo, file_path),
    }

    // println!("press enter to finish.");
    // io::stdin().read_line(&mut buffer)?;

    Ok(())
}

fn load_config(file_path: &str) -> Result<Config, GoodMorningError> {
    debug!("Read config file {:?}", file_path);

    let mut f = File::open(file_path)?;
    let mut buffer = String::new();

    f.read_to_string(&mut buffer)?;

    let decoded: Config = toml::from_str(&buffer)?;
    Ok(decoded)
}

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
