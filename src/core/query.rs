use std::fs::File;
use std::io::{BufRead, BufReader};
use std::io::Error;
use regex::Regex;

pub fn get_lines(file_path: &str) -> Result<Vec<String>, Error> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let lines: Vec<String> = reader
        .lines()
        .collect::<Result<Vec<String>, Error>>()?;
    Ok(lines)
}

fn replacer(list: Vec<String>, site: &str, string: &str) -> Vec<String> {
    let re = Regex::new("SITE|STRING").unwrap();

    list
        .into_iter()
        .map(|line| {
            re.replace_all(&line, |caps: &regex::Captures| {
                match &caps[0] {
                    "SITE" => site,
                    "STRING" => string,
                    _ => &caps[0],
                }.to_string()
            }).to_string()
        })
        .collect()
}

struct QueryBuilder {
    sites_file: String,
    payloads_file: String
}
impl QueryBuilder {
    pub fn new(sites_file: &str, payloads_file: &str) -> Self {
        Self {
            sites_file: sites_file.to_string(),
            payloads_file: payloads_file.to_string()
        }
    }

    pub fn get_sites(&self) -> Result<Vec<String>, Error> {
        get_lines(&self.sites_file)
    }

    pub fn get_payloads(&self) -> Result<Vec<String>, Error> {
        get_lines(&self.payloads_file)
    }
}

pub(crate) struct Query {
    builder: QueryBuilder,
    target: String,
}

impl Query {
    pub fn new(sites: &str, payloads: &str, target: &str) -> Self {
        Self {
            builder: QueryBuilder::new(sites, payloads),
            target: target.to_string()
        }
    }

    pub fn build(&self) -> Result<Vec<String>, Error> {
        let sites: Vec<String> = self.builder.get_sites()?;
        let payloads: Vec<String> = self.builder.get_payloads()?;

        Ok(
            sites
            .iter()
            .flat_map(|site| {
                replacer(payloads.clone(), site, &self.target)
            })
            .collect()
        )
    }
}
