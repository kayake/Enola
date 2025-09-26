mod core;

use crate::core::logger::{ LogLevel, Logger };
use crate::core::request::{ RandomUserAgent, build_request_to_google, send_build, parse};
use crate::core::query::{ Query, get_lines };
use crate::core::proxy::worker;

use clap::Parser;
use reqwest::header::HeaderMap;
use reqwest::{Client, RequestBuilder, Response};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tokio::sync::{mpsc, Mutex, Semaphore};

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

    #[arg(short='c', long="connections", help="Number of simultaneous requests (Only for API-Mode)", help_heading="Request", default_value_t=3)]
    simultaneous_requests: usize,

    #[arg(long, help="Proxy List", help_heading="Request")]
    proxies: Option<String>,

    #[arg(short='U', long="user-agents", help="Provide an User-Agents list", help_heading="Request", default_value_t=String::from("src/utils/request/user_agents.txt"))]
    user_agent_list: String,

    #[arg(short='d', long, help="Delay between requests (in milliseconds)", help_heading="Request", default_value_t=5000)]
    delay: u64,

    #[arg(short, long="apimode", help="Use APIs search engine's site", help_heading="Mode", default_value_t=false)]
    apimode: bool,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    let delay = args.delay;
    let logger = Arc::new(Logger::new(LogLevel::from(args.verbose)));
    logger.inf("loading queries...", false);
    let query: Vec<String> = if args.queries.is_none() {
        Query::new(&args.sites, &args.payloads, &args.target).build().unwrap()
    } else {
        get_lines(args.queries.as_deref().unwrap()).unwrap()
    };

    logger.inf(&format!("loading user-agents from {}...", args.user_agent_list), false);
    let user_agents = get_lines(&args.user_agent_list.to_string()).unwrap();
    if user_agents.is_empty() {
        logger.warn("no user-agents were found", true);
        let input = logger.input("Do you want to continue? [Y/n]").to_lowercase();
        if !input.starts_with('y') && !input.is_empty() {
            return logger.err("User interrupted", false);
        }
    }

    let user_agent: RandomUserAgent = RandomUserAgent::new(user_agents);

    if let Some(proxies) = args.proxies {
    logger.inf("loading proxies...", false);
    let proxies = get_lines(&proxies).unwrap();
    if proxies.is_empty() {
        logger.err("no proxies were found", true);
        return;
    }
    logger.dbg(&format!("{} proxy(ies) were loaded", proxies.len()), true);
    logger.inf("starting workers...", false);

    let (tx, rx) = mpsc::channel::<String>(100);
    let (log_tx, mut log_rx) = mpsc::channel::<String>(100);
    let (result_tx, mut result_rx) = mpsc::channel::<(String, Result<Response, reqwest::Error>)>(100);
    let semaphore = Arc::new(Semaphore::new(args.simultaneous_requests));

    let rx = Arc::new(Mutex::new(rx));

    for q in query.clone().into_iter() {
        tx.send(q).await.unwrap();
    };
    for (i, proxy) in proxies.iter().enumerate() {
        let worker_tx = tx.clone();
        let worker_rx = rx.clone(); 
        let worker_log_tx = log_tx.clone();
        let worker_result_tx = result_tx.clone();
        let worker_semaphore = semaphore.clone();
        let proxy_url = proxy.clone();
        let user_agent_str = user_agent.get_random();

        tokio::spawn(async move {
            worker(
                i,
                &proxy_url,
                &user_agent_str,
                worker_rx,
                worker_tx,
                worker_log_tx,
                worker_result_tx,
                worker_semaphore,
            )
            .await;
        });
    }


    let logger_for_log = Arc::clone(&logger);
    tokio::spawn(async move {
        while let Some(log) = log_rx.recv().await {
            logger_for_log.dbg(&log, false);
        }
    });

    let logger_for_result = Arc::clone(&logger);
    tokio::spawn(async move {
        while let Some((url, result)) = result_rx.recv().await {
            match result {
                Ok(res) if res.status().is_success() => {
                    let text = res.text().await.unwrap();
                    let parsed = parse(&text);
                    if parsed.is_empty() {
                        logger_for_result.nfnd(&format!("No results found for {}", url), true);
                        continue;
                    }
                    for (title, link, description) in parsed {
                        if !title.is_empty() && !link.is_empty() && !description.is_empty() {
                            logger_for_result.fnd(
                                format!("{} - {} ({})", title, description, link).as_str(),
                                true,
                                );
                            }
                        }
                    }
                    Ok(_) => {
                        logger_for_result.warn(&format!("Received non-success response for {}", url), true);
                    }
                    Err(_) => todo!(),
                }
            }
        });
        tokio::signal::ctrl_c().await.unwrap();
        logger.inf("Received Ctrl+C, shutting down...", true);
    }

    logger.dbg(&format!("{} build(s) were loaded", query.len()), true);
    logger.inf("creating client...", false);
    let mut headers = HeaderMap::new();

    headers.insert("Cookie", "CONSENT=YES+; SOCS=CAESHAgBEhIaAB".parse().unwrap());
    headers.insert("Accept", "*/*".parse().unwrap());

    let client: Client = Client::builder()
        .default_headers(headers)
        .tcp_keepalive(Some(std::time::Duration::from_secs(15)))
        .tcp_nodelay(true)
        .tcp_keepalive_interval(Some(std::time::Duration::from_secs(15)))
        .tcp_keepalive_retries(3)
        .build()
        .expect("Failed to build reqwest client");

    logger.dbg("client created", true);
    logger.inf("building requests..", false);
    let builds: Vec<RequestBuilder> = query
        .iter()
        .map(|q| build_request_to_google(client.clone(), q, user_agent.get_random()))
        .collect();

    let builds_len = builds.len();

    logger.inf(&format!("Done | {} request(s) were built", builds.len()), true);

    if delay < 1000 {
        logger.warn("delay less than 1000 milliseconds may lead to rate limiting by Google", false);
        let input = logger.input("Do you want to continue? [Y/n]").to_lowercase();
        if !input.starts_with('y') && !input.is_empty() {
            return logger.err("User interrupted", false);
        }
    }

    let time_approx = (builds_len as u64 * delay) / 1000;
    logger.inf(&format!("this may take approximately \x1b[35;1m{}\x1b[0m", time_format(time_approx)), false);
    let continues = logger.input("Do you want to continue? [Y/n]").to_lowercase();
    if !continues.starts_with('y') && !continues.is_empty() {
        return
    }
    logger.inf("starting requests..", false);
    let start_time = Instant::now();

    let mut responses: Vec<Response> = Vec::new();

    for build in builds {
        sleep(Duration::from_millis(delay)).await;
        logger.req(&format!("Sending request {:?}", build), false);
        let res: Response = send_build(build).await.unwrap();
        logger.res(&format!("Received response: {} {}", res.status(), res.url()), true);
        responses.push(res);
    }
    
    let duration = start_time.elapsed();
    logger.inf(&format!(
        "avarage {:.2} requests/min completed in {:.2?} (Percentage Error: {}%)", 
        builds_len as f64 / duration.as_secs_f64(), 
        time_format(duration.as_secs()), 
        (duration.as_secs_f64() - time_approx as f64) / (time_approx as f64) * 100 as f64
        ), 
        true
    );
    logger.inf("processing responses..", false);
    logger.dbg(&format!("{} responses loaded", responses.len()), false);
    if responses.is_empty() {
        logger.err("no responses were received", true);
        return;
    }
    for response in responses {
        let status = response.status();
        let url = response.url().clone();
        let text = parse(response.text().await.unwrap().as_str());
        if text.is_empty() {
            logger.nfnd(&format!("No results found for {} => {}", url, status), true);
            continue;
        }
        for (title, link, description) in text {
            if !title.is_empty() && !link.is_empty() && !description.is_empty() {
                logger.fnd(format!("{} - {} ({}) => {}", title, description, link, status).as_str(), true);
            }
            }
        }
}
