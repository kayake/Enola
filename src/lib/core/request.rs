use reqwest::{Client, Response, Error, RequestBuilder};

struct Requester;

impl Requester {
    pub fn build_request_to_google(client: Client, query: &str) -> Result<RequestBuilder> {
        client.get(format!("https://google.com/search?q={}", query)
        .header("User-Agent", "Mozila/5.0")
        .header("Cookie", "CONSENT=YES+; SOCS=CAESHAgBEhIaAB")
        .header("Accept", "*/*")
    }

    pub async fn send_build(build: RequestBuilder) -> Result<Response, Error> {
        build.send().await?;
    }
}

struct Response;
impl Response {
    pub async fn get_text(res: Response) <str> {
        res.await.text.await;
    }
}
