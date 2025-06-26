use custom_logger as log;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE};
use reqwest::Client;
use serde_derive::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Payload {
    pub content: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Embeddings {
    pub index: usize,
    pub embedding: Vec<Vec<f32>>,
}

pub async fn get_embeddings(
    url: String,
    content: String,
) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    let mut header_map: HeaderMap = HeaderMap::new();
    header_map.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    header_map.insert(ACCEPT, HeaderValue::from_static("application/json"));
    let client = Client::new();
    let payload = Payload { content };
    let json_payload = serde_json::to_string(&payload).unwrap();
    let res_post = client
        .post(url.clone())
        .body(json_payload.clone())
        .headers(header_map.clone())
        .send()
        .await;

    let mut embeddings: Vec<Embeddings> = Vec::new();

    log::trace!("embeddings result {:?}", res_post);

    if res_post.is_ok() {
        let body = res_post.unwrap().bytes().await;
        if body.is_ok() {
            let data = body.unwrap();
            embeddings = serde_json::from_slice(&data).unwrap();
            log::debug!(
                "index {} : embedding {}",
                embeddings[0].index,
                embeddings[0].embedding.len()
            );
            let mut count = 0;
            for array in embeddings[0].embedding.iter() {
                log::debug!("embedding : count {} : {:?}", count, array);
                count += 1;
            }
        }
    }

    Ok(embeddings[0].embedding[0].clone())
}
