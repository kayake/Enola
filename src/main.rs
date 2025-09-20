mod core;

use crate::core::logger::{ LogLevel, Logger };
use crate::core::request::{ build_request_to_google, send_build, parse};
use crate::core::query::{ Query, get_lines };

use clap::Parser;
use reqwest::header::HeaderMap;
use reqwest::{Client, RequestBuilder, Response};
use tokio::sync::{Mutex};
use std::sync::Arc;
use tokio::time::sleep;
use futures::stream::{ StreamExt };

fn time_format(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    let mut parts = Vec::new();
    if hours > 0 {
        parts.push(format!("{} hour{}", hours, if hours > 1 { "s" } else { "" }));
    }
    if minutes > 0 {
        parts.push(format!("{} minute{}", minutes, if minutes > 1 { "s" } else { "" }));
    }
    if secs > 0 || parts.is_empty() {
        parts.push(format!("{} second{}", secs, if secs > 1 { "s" } else { "" }));
    }  

    if parts.len() > 1 {
        let last = parts.len() - 1;
        let value = std::mem::take(&mut parts[last]);
        parts[last] = format!("and {}", value);
    }

    parts.join(", ")
}

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

    #[arg(short='c', long="connections", help="Number of simultaneous requests", help_heading="Request", default_value_t=3)]
    simultaneous_requests: usize,

    #[arg(long, help="Proxy List", help_heading="Request")]
    proxies: Option<String>,

    #[arg(short='d', long, help="Delay between requests (in milliseconds)", help_heading="Request", default_value_t=1000)]
    delay: u64,

    #[arg(short, long="apimode", help="Use APIs search engine's site", help_heading="Mode", default_value_t=false)]
    apimode: bool,
}

#[tokio::main]
async fn main() {
    let _args = Cli::parse();
    let delay = _args.delay;
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

    let builds_len = builds.len();

    _logger.inf(&format!("Done | {} request(s) were built", builds.len()), true);

    let simultaneous_requests = _args.simultaneous_requests;
    if simultaneous_requests == 0 {
        _logger.err("number of simultaneous requests cannot be zero", true);
        return;
    } else if simultaneous_requests > 3 {
        _logger.warn("number of simultaneous requests greater than 3 may lead to rate limiting by Google", false);
        let input = _logger.input("Do you want to continue? [Y/n]").to_lowercase();
        if !input.starts_with('y') && !input.is_empty() {
            return _logger.err("User interrupted", false);
        }
    }

    if delay < 1000 {
        _logger.warn("delay less than 1000 milliseconds may lead to rate limiting by Google", false);
        let input = _logger.input("Do you want to continue? [Y/n]").to_lowercase();
        if !input.starts_with('y') && !input.is_empty() {
            return _logger.err("User interrupted", false);
        }
    }
    let responses: Arc<Mutex<Vec<Response>>> = Arc::new(Mutex::new(Vec::new()));
    _logger.dbg(&format!("sending requests with {} simultaneous connections", simultaneous_requests), true);

    let time_approx = (builds.len() as u64 * delay) / simultaneous_requests as u64 / 1000;
    _logger.inf(&format!("this may take approximately \x1b[35;1m{}\x1b[0m", time_format(time_approx)), false);
    let continues = _logger.input("Do you want to continue? [Y/n]").to_lowercase();
    if !continues.starts_with('y') && !continues.is_empty() {
        return
    }
    _logger.inf("starting requests..", false);
    let start_time = std::time::Instant::now();
    let stream = futures::stream::iter(builds)
        .map(|build| {
            let responses = Arc::clone(&responses);
            let logger = Arc::clone(&_logger);
            async move {
                sleep(std::time::Duration::from_millis(delay)).await;
                match send_build(build).await {
                    Ok(response) => {
                        responses.lock().await.push(response);
                    },
                    Err(e) => {
                        return;
                    }
            }
        }
    })
        .buffer_unordered(simultaneous_requests);
stream.collect::<Vec<_>>().await;
let duration = start_time.elapsed();
_logger.inf(&format!("avarage {:.2} requests/min completed in {:.2?} (Percentage Error: {}%)", builds_len as f64 / duration.as_secs_f64(), time_format(duration.as_secs()), (duration.as_secs_f64() - time_approx as f64) / (time_approx as f64) * 100 as f64), true);
_logger.inf("processing responses..", false);
let _responses = Arc::try_unwrap(responses).unwrap().into_inner();
_logger.dbg(&format!("{} responses loaded", _responses.len()), false);
if _responses.is_empty() {
    _logger.err("no responses were received", true);
    return;
}
for response in _responses {
    let status = response.status();
    let url = response.url().clone();
    let text = parse(response.text().await.unwrap().as_str());
    _logger.req(&format!("Sending request to {}", url), false);
    _logger.res(&format!("Received response: {} {}", status, url), true);
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
