use reqwest::{Client, RequestBuilder, Response, Result};
use scraper::{Html, Selector};

pub fn build_request_to_google(client: Client, query: &str) -> RequestBuilder {
    client.get(format!("https://google.com/search?q={}", query))
    .header("User-Agent", "Mozila/5.0")
    .header("Cookie", "CONSENT=YES+; SOCS=CAESHAgBEhIaAB")
    .header("Accept", "*/*")
}

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
