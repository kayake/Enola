use std::fs::File;
use std::io::{self, BufRead, BufReader};
use regex::Regex;

fn get_lines(file_path: &str) -> io::Result<Vec<String>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let lines: Vec<String> = reader.lines().collect::<Result<_,_>>();
    Ok(lines)
}

fn replacer(list: Vec<String>, site: &str, string: &str) -> io::Result<Vec<String>> {
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
        .collect();
}

struct QueryBuilder {
    sites_file: String,
    payloads_file: String
}

impl QueryBuilder {
    pub fn new(sites_file: &str, payloads_file: &str) -> Self {
        Self {
            sites_file: sites_file,
            payloads_file: payloads_file
        }
    }

    pub fn get_sites(&self) -> io::Result<Vec<String>> {
        get_lines(self.sites_file);
    }

    pub fn get_payloads(&self) -> io::Result<Vec<String>> {
        get_lines(self.payloads_file);
    }
}

struct Query {
    builder: QueryBuilder,
    target: String
}
impl Query {
    pub fn new(sites: &str, payloads: &str, target: &str) -> Self {
        Self {
            builder: QueryBuilder::new(sites, payloads),
            target: target
        }
    }

    pub fn build(&self) -> io::Result<Vec<String>> {
        let sites: Vec<String> = self.get_sites();
        let payloads: Vec<String> = self.get_payloads();

        Ok(
            sites
            .iter()
            .flat_map(|site| {
                replacer(payloads, site, self.target)
            })
            .collect()
        );
    }
}
