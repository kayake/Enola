use reqwest::{Client, Response, Error, RequestBuilder};

struct Requester;

impl Requester {
    pub fn build_request_to_google(client: Client, query: &str) -> Result<RequestBuilder> {
        client.get(format!("https://google.com/search?q={}", query)
        .header("User-Agent", "Mozila/5.0")
        .header("Cookie", "CONSENT=YES+; SOCS=CAESHAgBEhIaAB")
    }

    pub fn send_build(build: RequestBuilder) -> Result<Response, Error> {
        build.send().await?;
    }   
}
