use reqwest::{Client, RequestBuilder, Response, Result};

pub fn build_request_to_google(client: Client, query: &str) -> RequestBuilder {
    client.get(format!("https://google.com/search?q={}", query))
    .header("User-Agent", "Mozila/5.0")
    .header("Cookie", "CONSENT=YES+; SOCS=CAESHAgBEhIaAB")
    .header("Accept", "*/*")
}

pub async fn send_build(build: RequestBuilder) -> Result<Response> {
        build.send().await
}
