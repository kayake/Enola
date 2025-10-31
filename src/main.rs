mod core;

use crate::core::logger::{LogLevel, Logger};
use crate::core::request::{ApiMode, RandomUserAgent, exec, parse};
use crate::core::query::{get_lines, Query};
use crate::core::proxy::worker;
use crate::core::save::{is_results_exists, save_results, save_results_simple};

use clap::Parser;
use dirs::home_dir;
use reqwest::{Client, Request, Response};
use std::env::current_dir;
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex, Semaphore};
use futures::stream::{self, StreamExt};

fn data_default() -> std::path::PathBuf {

    let mut path = home_dir().unwrap();
    path.push(".enola/");
    
    return path;
}

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

static DEFAULT_UTILS: LazyLock<std::path::PathBuf> = LazyLock::new(|| data_default().join("/"));

static DEFAULT_UTILS_SITES: LazyLock<std::path::PathBuf> = LazyLock::new(|| DEFAULT_UTILS.join("dorks/sites/all.txt"));
static DEFAULT_UTILS_PAYLOADS: LazyLock<std::path::PathBuf> = LazyLock::new(|| DEFAULT_UTILS.join("dorks/payloads/general.txt"));
static DEFAULT_USER_AGENTS: LazyLock<std::path::PathBuf> = LazyLock::new(|| DEFAULT_UTILS.join("request/user_agents.txt"));
static DEFAULT_API_SITES: LazyLock<std::path::PathBuf> = LazyLock::new(|| DEFAULT_UTILS.join("apis/profile_urls.txt"));

#[derive(Parser)]
#[command(name = "Enola")]
#[command(version = "1.0.0")]
#[command(about = "A powerful search tool", long_about = "Enola uses Google Dorks to get information")]
struct Cli {
    #[arg(short = 't', long, help = "Target", help_heading = "Target")]
    target: String,

    #[arg(
        short = 'v',
        long,
        help = "Verbose Level (1-7)",
        value_parser = clap::value_parser!(u8).range(1..=8),
        help_heading = "Miscellaneous",
        default_value_t = 5
    )]
    verbose: u8,

    #[arg(
        short = 'o',
        long,
        help = "Output path for results",
        help_heading = "Settings",
    )]
    output_path: Option<String>,

    #[arg(short = 'p', long, help = "Provide your Dork", help_heading = "Settings")]
    payload: Option<String>,

    #[arg(
        short = 'P',
        long,
        help = "Provide the list of Dorks",
        help_heading = "Settings",
        default_value_t = DEFAULT_UTILS_PAYLOADS.to_string_lossy().into_owned()
    )]
    payloads: String,

    #[arg(
        short = 's',
        long,
        help = "Provide the list of Sites",
        help_heading = "Settings",
        default_value_t = DEFAULT_UTILS_SITES.to_string_lossy().into_owned()
    )]
    sites: String,
    
    #[arg(short = 'q', long, help = "Provide queries", help_heading = "Settings")]
    queries: Option<String>,
    
    #[arg(
        short,
        long = "api-sites",
        help = "Use APIs search engine's site",
        help_heading = "Settings",
        default_value_t = DEFAULT_API_SITES.to_string_lossy().into_owned()
    )]
    api_sites: String,
    #[arg(
        short = 'c',
        long = "connections",
        help = "Number of simultaneous requests (Only for API-Mode)",
        help_heading = "Request",
        default_value_t = 3
    )]
    simultaneous_requests: usize,

    #[arg(long, help = "Proxy List", help_heading = "Request")]
    proxies: Option<String>,

    #[arg(
        short = 'w',
        long = "workers",
        help = "Number of workers (Only for Proxy-Mode)",
        help_heading = "Request",
        default_value_t = 5
    )]
    workers: usize,

    #[arg(
        short = 'U',
        long = "user-agents",
        help = "Provide an User-Agents list",
        help_heading = "Request",
        default_value_t = DEFAULT_USER_AGENTS.to_string_lossy().into_owned()
    )]
    user_agent_list: String,

    #[arg(
        short = 'd',
        long,
        help = "Delay between requests (in milliseconds)",
        help_heading = "Request",
        default_value_t = 5000
    )]
    delay: u64,

    #[arg(
        short,
        long = "google-dork-mode",
        help = "Use Google Dork (only with Proxy)",
        help_heading = "Mode",
        default_value_t = false
    )]
    google_dork_mode: bool,

}

async fn run_proxy_mode(
    args: &Cli,
    target: &str,
    logger: &Arc<Logger>,
    user_agent: &RandomUserAgent,
) -> Result<(), String> {
    logger.inf("Google dork mode enabled", true);
    logger.inf("loading queries...", false);
    let query: Vec<String> = if args.queries.is_none() {
        Query::new(&args.sites, &args.payloads, target)
            .build()
            .map_err(|e| format!("Failed to build queries: {}", e))?
    } else {
        get_lines(args.queries.as_deref().unwrap()).map_err(|e| format!("Failed to load queries: {}", e))?
    };

    if args.simultaneous_requests > 3 {
        logger.warn(
            "Using more than 3 simultaneous requests with proxies may increase RAM usage",
            true,
        );
        let input = logger.input("Do you want to continue? [Y/n]").to_lowercase();
        if !input.starts_with('y') && !input.is_empty() {
            return Err("User interrupted".to_string());
        }
    }

    logger.inf("loading proxies...", false);
    let proxies = get_lines(args.proxies.as_ref().unwrap()).map_err(|e| format!("Failed to load proxies: {}", e))?;
    if proxies.is_empty() {
        logger.err("no proxies were found", true);
        return Err("No proxies found".to_string());
    }
    logger.dbg(&format!("{} proxy(ies) were loaded", proxies.len()), true);

    let time_approx = query.len() / args.simultaneous_requests;
    logger.inf(
        &format!(
            "with {} proxies and {} simultaneous requests, this may take approximately \x1b[35;1m{}\x1b[0m",
            proxies.len(),
            args.simultaneous_requests,
            time_format(time_approx as u64)
        ),
        false,
    );
    let continues = logger.input("Do you want to continue? [Y/n]").to_lowercase();
    if !continues.starts_with('y') && !continues.is_empty() {
        return Err("User interrupted".to_string());
    }

    logger.inf("starting workers...", false);
    let start_time = Instant::now();

    let (tx, rx) = mpsc::channel::<String>(100);
    let (log_tx, mut log_rx) = mpsc::channel::<String>(100);
    let (result_tx, mut result_rx) = mpsc::channel::<(String, Result<Response, reqwest::Error>)>(100);
    let semaphore = Arc::new(Semaphore::new(args.simultaneous_requests));
    let rx = Arc::new(Mutex::new(rx));

    for q in query.clone() {
        tx.send(q).await.map_err(|e| format!("Failed to send query: {}", e))?;
    }

    let proxies = get_lines(&args.proxies.as_deref().unwrap())
        .map_err(|e| format!("Failed to load proxies: {}", e))?;
    if proxies.is_empty() {
        logger.err("no proxies were found", true);
        return Err("No proxies found".to_string());
    }

    if args.workers == 0 {
        return Err("Number of workers must be at least 1".to_string());
    }

    if args.workers > 5 {
        logger.warn(
            "Using more than 5 workers may increase RAM usage",
            true,
        );
        let input = logger.input("Do you want to continue? [Y/n]").to_lowercase();
        if !input.starts_with('y') && !input.is_empty() {
            return Err("User interrupted".to_string());
        }
    }

    for i in 0..args.workers {
        let worker_tx = tx.clone();
        let worker_rx = rx.clone();
        let worker_log_tx = log_tx.clone();
        let worker_result_tx = result_tx.clone();
        let worker_semaphore = semaphore.clone();
        let user_agent_str = user_agent.get_random();
        let proxies_clone = proxies.clone();

        tokio::spawn(async move {
            worker(
                i,
                proxies_clone,
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

    let logger_for_log = Arc::clone(logger);
    tokio::spawn(async move {
        while let Some(log) = log_rx.recv().await {
            logger_for_log.req(&log, false);
        }
    });

    let logger_for_result = Arc::clone(logger);
    let target_for_save = target.to_string();
    let output_path = args.output_path.clone();
    tokio::spawn(async move {
        while let Some((url, result)) = result_rx.recv().await {
            match result {
                Ok(res) if res.status().is_success() => {
                    logger_for_result.res(&format!("Status {:?} for {}", res.status(), url), true);
                    let text = res.text().await.unwrap_or_default();
                    let parsed = parse(&text);
                    if parsed.is_empty() {
                        logger_for_result.nfnd(&format!("No results found for {}", url), true);
                        continue;
                    }
                    for (title, link, description) in parsed {
                        if !title.is_empty() && !link.is_empty() && !description.is_empty() {
                            logger_for_result.fnd(
                                &format!("{} - {} ({})", title, description, link),
                                true,
                            );
                            save_results(&logger_for_result, &target_for_save, &vec![(title.clone(), link.clone(), description.clone())], output_path.as_deref())
                                .unwrap_or_else(|e| {
                                    logger_for_result.err(&format!("Failed to save results: {}", e), true);
                                });
                        }
                    }
                }
                Ok(res) => {
                    logger_for_result.warn(&format!("Received non-success response for {}: {}", url, res.status()), true);
                }
                Err(e) => {
                    logger_for_result.err(&format!("Request failed for {}: {}", url, e), true);
                }
            }
        }
    });

    tokio::signal::ctrl_c().await.unwrap();
    logger.inf("Received Ctrl+C, shutting down...", true);
    let duration = start_time.elapsed();
    logger.inf(
        &format!(
            "average {:.2} requests/min completed in {} (Percentage Error: {}%)",
            query.len() as f64 / duration.as_secs_f64(),
            time_format(duration.as_secs()),
            (duration.as_secs_f64() - time_approx as f64) / (time_approx as f64) * 100.0
        ),
        true,
    );
    logger.inf("All tasks completed!", true);
    Ok(())
}

#[derive(Debug)]
enum LogMessage {
    Request(String),
    Response(String),
}

async fn run_api_mode(
    args: &Cli,
    target: &str,
    logger: &Arc<Logger>,
    user_agent: &RandomUserAgent,
) -> Result<(), String> {
    if args.simultaneous_requests > 5 {
        logger.warn(
            "Using more than 5 simultaneous requests may increase RAM usage",
            true,
        );
        let input = logger.input("Do you want to continue? [Y/n]").to_lowercase();
        if !input.starts_with('y') && !input.is_empty() {
            return Err("User interrupted".to_string());
        }
    }

    if args.payload.is_some() {
        logger.warn("Payload will be ignored in API mode", true);
    }

    logger.inf(&format!("loading sites from {}...", args.api_sites), false);
    let sites = get_lines(&args.api_sites).map_err(|e| format!("Failed to load sites: {}", e))?;
    if sites.is_empty() {
        logger.err("no sites were found", true);
        return Err("No sites found".to_string());
    }

    logger.dbg(&format!("{} site(s) were loaded", sites.len()), true);
    logger.inf("creating client...", false);
    let client = Client::builder()
        .tcp_keepalive(Some(Duration::from_secs(15)))
        .tcp_nodelay(true)
        .tcp_keepalive_interval(Some(Duration::from_secs(15)))
        .tcp_keepalive_retries(3)
        .build()
        .map_err(|e| format!("Failed to build reqwest client: {}", e))?;

    let manager = ApiMode::new(target.to_string());
    let builds = sites
        .iter()
        .map(|site| {
            let client_ = client.clone();
            manager.build(site, &client_, user_agent.get_random())
        })
        .collect::<Vec<Request>>();
    logger.dbg(&format!("{} build(s) were loaded", builds.len()), true);

    let time_approx = builds.len() as u64 * args.delay / args.simultaneous_requests as u64;
    logger.inf(
        &format!(
            "this may take approximately \x1b[35;1m{}\x1b[0m",
            time_format(time_approx)
        ),
        false,
    );
    let continues = logger.input("Do you want to continue? [Y/n]").to_lowercase();
    if !continues.starts_with('y') && !continues.is_empty() {
        return Err("User interrupted".to_string());
    }

    logger.inf("starting requests...", false);
    let start_time = Instant::now();
    let semaphore = Arc::new(Semaphore::new(args.simultaneous_requests));

    let (log_tx, mut log_rx) = mpsc::channel::<LogMessage>(100);
    let logger_for_logs = Arc::clone(logger);

    tokio::spawn(async move {
        while let Some(log) = log_rx.recv().await {
            match log {
                LogMessage::Request(msg) => logger_for_logs.req(&msg, true),
                LogMessage::Response(msg) => logger_for_logs.res(&msg, true),
            }
        }
    });

    let results: Vec<_> = stream::iter(builds.into_iter().map(|build| {
        let client_ = client.clone();
        let log_tx = log_tx.clone();
        let sem = Arc::clone(&semaphore);
        async move {
            let _permit = sem.acquire().await.unwrap();
            let url = build.url().to_string();
            let _ = log_tx
                .send(LogMessage::Request(format!("Sending request to {}", url)))
                .await
                .map_err(|e| eprintln!("Failed to send log: {}", e));

            let result = exec(&client_, build).await;
            let _ = log_tx
                .send(LogMessage::Response(match &result {
                    Ok(res) => format!("Received response for {} with status: {}", url, res.status()),
                    Err(e) => format!("Error receiving response for {}: {}", url, e),
                }))
                .await
                .map_err(|e| eprintln!("Failed to send log: {}", e));

            result
        }
    }))
    .buffer_unordered(args.simultaneous_requests)
    .collect()
    .await;

    let duration = start_time.elapsed();
    logger.inf(
        &format!(
            "average {:.2} requests/min completed in {} (Percentage Error: {}%)",
            results.len() as f64 / duration.as_secs_f64(),
            time_format(duration.as_secs()),
            (duration.as_secs_f64() - time_approx as f64) / (time_approx as f64) * 100.0
        ),
        true,
    );

    logger.inf("processing responses...", false);
    if results.is_empty() {
        logger.err("no responses were received", true);
        return Err("No responses received".to_string());
    }

    logger.dbg(&format!("{} responses loaded", results.len()), false);
    let mut found_urls = Vec::new();
    for result in results {
        match result {
            Ok(res) if res.status().is_success() => {
                let url = res.url().clone();
                let status = res.status();
                let text = res.text().await.unwrap_or_default();
                if text.contains("Not Found") || text.contains("404") {
                    logger.nfnd(&format!("No results found for {}", url), true);
                } else {
                    logger.fnd(
                        &format!("Results found for {} => \x1b[35;1m{}\x1b[0m", url, status),
                        true,
                    );
                    found_urls.push(url.to_string());
                }
            }
            Ok(_) => {}
            Err(e) => {
                logger.res(&format!("Error occurred: {}", e), true);
            }
        }
    }

    save_results_simple(&logger, &args.target, &found_urls, args.output_path.as_deref()).unwrap_or_else(|e| {
        logger.err(&format!("Failed to save results: {}", e), true);
    });

    logger.inf("All tasks completed!", true);
    logger.inf(
        &format!(
            "Results saved to {}/results/{}.txt",
            current_dir().unwrap().display(),
            target.replace("/", "_")
        ),
        true,
    );

    Ok(())
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    let logger = Arc::new(Logger::new(LogLevel::from(args.verbose)));
    let target = args.target.clone();

    let (exists, file) = is_results_exists(&logger, &target, args.output_path.as_deref());

    if exists {
        logger.warn(
            &format!(
                "Results for {} already exists in {}",
                target,
                file.display()
            ),
            true,
        );
        let input = logger.input("Do you want to continue? [Y/n]").to_lowercase();
        if !input.starts_with('y') && !input.is_empty() {
            logger.err("User interrupted", false);
            return;
        }
    }

    logger.inf(&format!("loading user-agents from {}...", args.user_agent_list), false);
    let user_agents = get_lines(&args.user_agent_list).unwrap_or_default();
    if user_agents.is_empty() {
        logger.warn("no user-agents were found", true);
        let input = logger.input("Do you want to continue? [Y/n]").to_lowercase();
        if !input.starts_with('y') && !input.is_empty() {
            logger.err("User interrupted", false);
            return;
        }
    }
    let user_agent = RandomUserAgent::new(user_agents);

    match (args.proxies.is_some(), args.google_dork_mode) {
        (true, true) => {
            if let Err(e) = run_proxy_mode(&args, &target, &logger, &user_agent).await {
                logger.err(&format!("Error during execution: {}", e), true);
                std::process::exit(1);
            }
        },
        _ => {
            if let Err(e) = run_api_mode(&args, &target, &logger, &user_agent).await {
                logger.err(&format!("Error during execution: {}", e), true);
                std::process::exit(1);
            }
        }
    }
}