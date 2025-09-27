use reqwest::{Client, Method, RequestBuilder, Response, Result};
use scraper::{Html, Selector};
use urlencoding::encode;
use rand::{rng, seq::IndexedRandom};


pub async fn send_build(build: RequestBuilder) -> Result<Response> {
        build.send().await
}

pub fn parse(text: &str) -> Vec<(String, String, String)> {
    let mut results: Vec<(String, String, String)> = Vec::new();
    let document = Html::parse_document(text);
    let result_selector = Selector::parse("div.ezO2md").unwrap();
    let link_selector = Selector::parse("a[href]").unwrap();
    let title_selector = Selector::parse("span.CVA68e").unwrap();
    let description_selector = Selector::parse("span.FrIlee").unwrap();

    for result in document.select(&result_selector) {
        let link_tag = result.select(&link_selector).next();
        let title_tag = link_tag.and_then(|lt| lt.select(&title_selector).next());
        let description_tag = result.select(&description_selector).next();
        
        if let (Some(link_el), Some(title_el), Some(description_el)) = (link_tag.clone(), title_tag, description_tag) {
            let href = link_el.value().attr("href").unwrap_or("");
            let link = href
                .strip_prefix("/url?q=")
                .unwrap_or(href)
                .split('&')
                .next()
                .unwrap_or("")
                .to_string();

            let title = title_el.text().collect::<Vec<_>>().join(" ").trim().to_string();
            let description = description_el.text().collect::<Vec<_>>().join(" ").trim().to_string();

            results.push((title, link, description));
        }
    }

    results
    
}

pub struct RandomUserAgent {
    user_agents: Vec<String>,
}

impl RandomUserAgent {
    pub fn new(user_agents: Vec<String>) -> Self {
        Self {
            user_agents: user_agents
        }
    }

    pub fn get_random(&self) -> String {
        self.user_agents.choose(&mut rng()).unwrap().to_string()
    }
}

pub struct ApiMode {
    target: String,
}

impl ApiMode {
    pub fn new(target: String) -> Self {
        Self {
            target
        }
    }

    pub fn build(&self, query: &str, client: Client, user_agent: String) -> RequestBuilder {
        let parts: Vec<&str> = query.splitn(3, ' ').collect();
        if parts.len() != 3 {
            panic!("Invalid API mode query format. Expected: <METHOD> <SITE> <DATA>");
        }
        let method = parts[0];
        let site = parts[1];
        let data = parts[2];
        let user_agent = user_agent;
        let site = site.replace("USER", &encode(self.target.as_str()));
        let method = Method::from_bytes(method.as_bytes()).expect("Invalid HTTP method");
        let build: RequestBuilder = client.request(method, &site)
                                        .header("User-Agent", user_agent);
        if !data.is_empty() {
            return build.body(data.to_string());
        }

        build
    }
}