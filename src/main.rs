mod core;

use crate::core::logger::{ LogLevel, Logger };
use crate::core::request::{ build_request_to_google, send_build, parse};
use crate::core::query::{ Query, get_lines };

use clap::Parser;
use reqwest::header::HeaderMap;
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

    #[arg(short='v', long, help="Verbose Level (1-7)", value_parser=clap::value_parser!(u8).range(1..=8), help_heading="Miscellaneous", default_value_t=5)]
    verbose: u8,

    #[arg(short='p', long, help="Provide your Dork", help_heading="Settings")]
    payload: Option<String>,

    #[arg(short='P', long, help="Provide the list of Dorks", help_heading="Settings", default_value_t=String::from("src/utils/dorks/payloads/general.txt"))]
    payloads: String,
    #[arg(short='s', long, help="Provide the list of Sites", help_heading="Settings", default_value_t=String::from("src/utils/dorks/sites/all.txt"))]
    sites: String,

    #[arg(short='q', long, help="Provide queries", help_heading="Settings")]
    queries: Option<String>,

    #[arg(short='c', long="connections", help="Number of simultaneous requests", help_heading="Settings", default_value_t=10)]
    simultaneous_requests: usize,

    #[arg(long, help="Proxy List", help_heading="Settings")]
    proxies: Option<String>,

    #[arg(short, long="apimode", help="Use APIs search engine's site", help_heading="Mode", default_value_t=false)]
    apimode: bool,
}

#[tokio::main]
async fn main() {
    let _args = Cli::parse();
    let _logger = Arc::new(Logger::new(LogLevel::from(_args.verbose)));
    _logger.inf("loading queries..", false);
    let query: Vec<String> = if _args.queries.is_none() {
        Query::new(&_args.sites, &_args.payloads, &_args.target).build().unwrap()
    } else {
        get_lines(_args.queries.as_deref().unwrap()).unwrap()
    };
    _logger.dbg(&format!("{} build(s) were loaded", query.len()), true);
    _logger.inf("creating client..", false);
    let mut headers = HeaderMap::new();
    // Add any default headers you want, for example:
    // headers.insert("Accept", "text/html".parse().unwrap());

    headers.insert("Cookie", "CONSENT=YES+; SOCS=CAESHAgBEhIaAB".parse().unwrap());
    headers.insert("Accept", "*/*".parse().unwrap());

    let client: Client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3")
        .default_headers(headers)
        .tcp_keepalive(Some(std::time::Duration::from_secs(15)))
        .tcp_nodelay(true)
        .tcp_keepalive_interval(Some(std::time::Duration::from_secs(15)))
        .tcp_keepalive_retries(3)
        .build()
        .expect("Failed to build reqwest client");

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
                logger.req(&format!("Sending request to {}", build.try_clone().unwrap().build().unwrap().url()), false);
                match send_build(build).await {
                    Ok(response) => {
                        logger.res(&format!("Received response: {} {}", response.status(), response.url()), false);
                        responses.lock().unwrap().push(response);
                    },
                    Err(e) => {
                        logger.err(&format!("Request failed: {}", e), true);
                        return;
                    }
            }
        })
    })
    .buffer_unordered(simultaneous_requests);
    stream.collect::<Vec<_>>().await;
    _logger.inf("processing responses..", false);
    let _responses = Arc::try_unwrap(responses).unwrap().into_inner().unwrap();
    _logger.dbg(&format!("{} responses loaded", _responses.len()), false);
    if _responses.is_empty() {
        _logger.err("no responses were received", true);
        return;
    }
    for response in _responses {
        let status = response.status();
        let url = response.url().clone();
        let text = parse(response.text().await.unwrap().as_str());
        if text.is_empty() {
            _logger.nfnd(&format!("No results found for {} => {}", url, status), true);
            continue;
        }
        for (title, link, description) in text {
            if !title.is_empty() && !link.is_empty() && !description.is_empty() {
                _logger.fnd(format!("{} - {} ({}) => {}", title, description, link, status).as_str(), true);
            }
        }
    }
}
