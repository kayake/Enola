use std::fs::{File, create_dir_all};
use std::io::{Error, Write};
use std::path::{Path, PathBuf};
use std::{env};

use crate::core::logger::Logger;

const APP_NAME: &str = "enola";

fn results_dir(logger: &Logger) -> PathBuf {
    let exe = env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    let exe_dir = exe.parent().unwrap_or_else(|| Path::new("."));

    if exe_dir.join("src").exists() {
        logger.dbg("Local mode → ./results", false);
        return exe_dir.join("results");
    }

    if let Some(home) = dirs::home_dir() {
        let path = home.join(format!(".local/share/{}/results", APP_NAME));
        logger.dbg(&format!("System mode → {}", path.display()), false);
        return path;
    }

    logger.warn("Fallback to local ./results", false);
    PathBuf::from("./results")
}

fn filename(target: &str) -> String {
    if target.trim().is_empty() {
        APP_NAME.to_string()
    } else {
        target.to_string()
    }
}

fn resolve_output_path(logger: &Logger, target: &str, output: Option<&str>) -> PathBuf {
    match output {
        None => {
            results_dir(logger).join(format!("{}.txt", filename(target)))
        }
        Some(path_str) => {
            let path = PathBuf::from(path_str);

            if path.is_dir() || path_str.ends_with('/') {
                create_dir_all(&path).ok();
                return path.join(format!("{}.txt", filename(target)));
            }

            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    create_dir_all(parent).ok();
                }
            }

            path
        }
    }
}

pub fn save_results(
    logger: &Logger,
    target: &str,
    results: &Vec<(String, String, String)>,
    output_file: Option<&str>
) -> Result<(), Error> {

    let file_path = resolve_output_path(logger, target, output_file);
    let mut file = File::create(&file_path)?;

    logger.inf(&format!("Writing → {}", file_path.display()), true);

    for (title, link, description) in results {
        writeln!(file, "Title: {}\nLink: {}\nDescription: {}\n", title, link, description)?;
    }

    logger.fnd("Saved successfully.", true);
    Ok(())
}

pub fn save_results_simple(
    logger: &Logger,
    target: &str,
    results: &Vec<String>,
    output_file: Option<&str>
) -> Result<(), Error> {

    let file_path = resolve_output_path(logger, target, output_file);
    let mut file = File::create(&file_path)?;

    logger.inf(&format!("Writing → {}", file_path.display()), true);

    for line in results {
        writeln!(file, "{}", line)?;
    }

    logger.fnd("Saved successfully.", true);
    Ok(())
}

pub fn is_results_exists(
    logger: &Logger,
    target: &str,
    output_file: Option<&str>
) -> (bool, PathBuf) {
    let file_path = resolve_output_path(logger, target, output_file);
    let exists = file_path.exists();

    (exists, file_path)
}
