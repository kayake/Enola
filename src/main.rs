mod core;

use crate::core::logger::{ LogLevel, Logger };
use crate::core::request::{ build_request_to_google, send_build};
use crate::core::query::{ Query, get_lines };

use clap::Parser;
use reqwest::{Client, RequestBuilder, Response};
use std::sync::{Mutex, Arc};
use futures::stream::{ StreamExt };

#[derive(Parser)]
#[command(name = "Enola")]
#[command(version = "1.0.0")]
#[command(about = "A powerful search tool", long_about = "Enola uses Google Dorks to get information")]
struct Cli {
    #[arg(short, long, help="Target", help_heading="Target")]
    target: String,

    #[arg(short, long, help="Verbose Level (1-7)", value_parser=clap::value_parser!(u8).range(1..=7), help_heading="Miscellaneous", default_value_t=1)]
    verbose: u8,

    #[arg(short, long, help="Provide your Dork", help_heading="Settings")]
    payload: String,

    #[arg(short, long, help="Provide the list of Dorks", help_heading="Settings", default_value_t=String::from("src/utils/dorks/payloads/general.txt"))]
    payloads: String,
    #[arg(short, long, help="Provide the list of Sites", help_heading="Settings", default_value_t=String::from("src/utils/dorks/sites/all.txt"))]
    sites: String,

    #[arg(short, long, help="Provide queries", help_heading="Settings", default_value_t=String::from(""))]
    queries: String,
}

#[tokio::main]
async fn main() {
    let _args = Cli::parse();
    let _logger = Arc::new(Logger::new(LogLevel::from(_args.verbose)));

    let query: Vec<String> = if _args.queries.is_empty() {
        Query::new(&_args.sites, &_args.payloads, &_args.target).build().unwrap()
    } else {
        get_lines(&_args.queries).unwrap()
    };
    _logger.dbg(&format!("{} build(s) were loaded", query.len()), true);

    let client = Client::new();
    _logger.dbg("Client created", true);
    
    let builds: Vec<RequestBuilder> = query
        .iter()
        .map(|q| build_request_to_google(client.clone(), q))
        .collect();

    _logger.inf(&format!("{} request(s) were built", builds.len()), true);

    let simultaneous_requests = 10;
    let responses: Arc<Mutex<Vec<Response>>> = Arc::new(Mutex::new(Vec::new()));
    _logger.dbg(&format!("Sending requests with {} simultaneous connections", simultaneous_requests), true);

    let stream = futures::stream::iter(builds)
        .map(|build| {
            let responses = Arc::clone(&responses);
            let logger = Arc::clone(&_logger);
            tokio::spawn(async move {
                match send_build(build).await {
                    Ok(response) => {
                        let mut res = responses.lock().unwrap();
                        res.push(response);
                    },
                    Err(e) => {
                        logger.err(&format!("Request failed: {}", e), true);
                    }
            }
        })
    })
    .buffer_unordered(simultaneous_requests);
    stream.collect::<Vec<_>>().await;

    for response in Arc::try_unwrap(responses).unwrap().into_inner().unwrap() {
        _logger.res(&format!("Response: {} {}", response.status(), response.url()), false);
    }
}
