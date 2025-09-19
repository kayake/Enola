mod core;

use crate::core::logger::{ LogLevel, Logger };
use crate::core::request::{ build_request_to_google, send_build, parse};
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
    #[arg(short='t', long, help="Target", help_heading="Target")]
    target: String,

    #[arg(short='v', long, help="Verbose Level (1-7)", value_parser=clap::value_parser!(u8).range(1..=7), help_heading="Miscellaneous", default_value_t=1)]
    verbose: u8,

    #[arg(short='p', long, help="Provide your Dork", help_heading="Settings")]
    payload: String,

    #[arg(short='P', long, help="Provide the list of Dorks", help_heading="Settings", default_value_t=String::from("src/utils/dorks/payloads/general.txt"))]
    payloads: String,
    #[arg(short='s', long, help="Provide the list of Sites", help_heading="Settings", default_value_t=String::from("src/utils/dorks/sites/all.txt"))]
    sites: String,

    #[arg(short='q', long, help="Provide queries", help_heading="Settings", default_value_t=String::from(""))]
    queries: String,

    #[arg(short='c', long, help="Number of simultaneous requests", help_heading="Settings", default_value_t=10)]
    simultaneous_requests: usize,
}

#[tokio::main]
async fn main() {
    let _args = Cli::parse();
    let _logger = Arc::new(Logger::new(LogLevel::from(_args.verbose)));
    _logger.inf("loading queries..", false);
    let query: Vec<String> = if _args.queries.is_empty() {
        Query::new(&_args.sites, &_args.payloads, &_args.target).build().unwrap()
    } else {
        get_lines(&_args.queries).unwrap()
    };
    _logger.dbg(&format!("{} build(s) were loaded", query.len()), true);
    _logger.inf("creating client..", false);
    let client = Client::new();
    _logger.dbg("client created", true);

    _logger.inf("building requests..", false);
    let builds: Vec<RequestBuilder> = query
        .iter()
        .map(|q| build_request_to_google(client.clone(), q))
        .collect();

    _logger.inf(&format!("Done | {} request(s) were built", builds.len()), true);

    let simultaneous_requests = _args.simultaneous_requests;
    if simultaneous_requests == 0 {
        _logger.err("number of simultaneous requests cannot be zero", true);
        return;
    } else if simultaneous_requests > 10 {
        _logger.warn("number of simultaneous requests greater than 10 may lead to rate limiting by Google", false);
    }
    let responses: Arc<Mutex<Vec<Response>>> = Arc::new(Mutex::new(Vec::new()));
    _logger.dbg(&format!("sending requests with {} simultaneous connections", simultaneous_requests), true);

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
        let status = response.status();
        let url = response.url().clone();
        _logger.req(&format!("Request to {}", url), false);
        _logger.res(&format!("Response: {} {}", status, url), false);
        let text = parse(response.text().await.unwrap().as_str());
        for (title, link, description) in text {
            if !title.is_empty() && !link.is_empty() && !description.is_empty() {
                _logger.fnd(format!("{} - {} ({}) => {}", title, description, link, status).as_str(), true);
            } else {
                _logger.nfnd(format!("{} => {}", url, status).as_str(), true);
            }
        }
    }
}
