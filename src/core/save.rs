use std::fs::{File, create_dir};
use std::io::{Error, Write};
use std::path::Path;
use std::env;

pub fn save_results(target: &str, results: &Vec<(String, String, String)>) -> Result<(), Error> {
    let file_path = format!(".results/{}.txt", target);
    if !Path::new(env!("CARGO_MANIFEST_DIR")).join(Path::new(".results")).exists() {
        create_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join(Path::new(".results")))?;
    }
    let mut file = File::create(Path::new(env!("CARGO_MANIFEST_DIR")).join(Path::new(&file_path)))?;
    
    for (title, link, description) in results {
        writeln!(file, "Title: {}\nLink: {}\nDescription: {}\n", title, link, description)?;
    }
    Ok(())
}

pub fn save_results_simple(target: &str, results: &Vec<String>) -> Result<(), Error> {
    let file_path = format!(".results/{}.txt", target);
    if !Path::new(env!("CARGO_MANIFEST_DIR")).join(Path::new(".results")).exists() {
        create_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join(Path::new(".results")))?;
    }
    let mut file = File::create(Path::new(env!("CARGO_MANIFEST_DIR")).join(Path::new(&file_path)))?;
    for line in results {
        writeln!(file, "{}", line)?;
    }
    Ok(())
}

pub fn is_results_exists(target: &str) -> bool {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(Path::new(&format!(".results/{}", target))).exists()
}