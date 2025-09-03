mod core;

use core::logger::Logger
use core::request::{Requester, Response}
use core::query::Query

use clap::Parser;

#[derive(Parser)]
#[command(name = "Enola")
#[command(version = "1.0.0")
#[command(about = "A powerful search tool", long_about = "Enola uses Google Dorks to get information")]
struct Cli {
    #[arg(short, long, help="Target", help_heading="Target")]
    target: String

    #[arg(short, long, help="Verbose Level (1-7)", value_parser=clap::value_parser!(u8).range(1..=7), help_heading="Miscellaneous")]
    verbose: u8

    #[arg(short, long, help="Provide your Dork", help_heading="Settings")]
    payload: String,

    #[arg(short, long, help="Provide the list of Dorks", help_heading="Settings", default_value_t=String::from("src/lib/utils/dorks/payloads/general.txt"))]
    payloads: String
    #[arg(short, long, help="Provide the list of Sites", help_heading="Settings", default_value_t=String::from("src/lib/utils/dorks/sites/all.txt"))]
    sites: String

}
