use reqwest::{Client, Proxy, Response, Error};
use tokio::sync::{Semaphore, mpsc, Mutex};
use std::{sync::Arc};
use rand::{rng, seq::IndexedRandom};

async fn build_client(proxy: &str, user_agent: &str) -> Client {
    Client::builder()
        .proxy(Proxy::all(proxy).unwrap())
        .user_agent(user_agent)
        .build()
        .unwrap()
}

pub async fn worker(
    id: usize,
    proxies: Vec<String>,
    user_agent: &str,
    rx: Arc<Mutex<mpsc::Receiver<String>>>,
    tx: mpsc::Sender<String>,
    log_tx: mpsc::Sender<String>,
    result_tx: mpsc::Sender<(String, Result<Response, Error>)>,
    semaphore: Arc<Semaphore>,
) {
    let proxy = proxies.choose(&mut rng()).unwrap();
    let client = build_client(proxy, user_agent).await;

    loop {
        let maybe_url = {
            let mut locked_rx = rx.lock().await;
            locked_rx.recv().await
        };

        let url = match maybe_url {
            Some(u) => u,
            None => {
                let _ = log_tx.send(format!("[#{}]: Receiver closed", id)).await;
                break;
            }
        };

        let permit = semaphore.acquire().await.unwrap();

        let result = client
            .get(&url)
            .header("Cookie", "CONSENT=YES+; SOCS=CAESHAgBEhIaAB")
            .header("Accept", "*/*")
            .send()
            .await;

        match &result {
            Ok(res) if res.status().is_success() => {
                let _ = log_tx
                    .send(format!("[#{} => {}] Successfully fetched {}", id, proxy.clone().split("://").collect::<Vec<&str>>()[1],  url))
                    .await;
                let _ = result_tx.send((url.clone(), result)).await;
            }
            _ => {
                let _ = log_tx
                    .send(format!("[#{} => {}] Failed to fetch {}", id, proxy.clone().split("://").collect::<Vec<&str>>()[1], url))
                    .await;
                let _ = tx.send(url.clone()).await;
            }
        }

        drop(permit);
    }
}
