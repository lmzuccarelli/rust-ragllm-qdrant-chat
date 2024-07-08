use custom_logger::*;
use http_body_util::{BodyExt, Full};
use hyper::body::*;
use hyper::{Method, Request, Response};
use ollama_rs::Ollama;
use qdrant_client::Qdrant;
use serde::{Deserialize, Serialize};
use std::fs;
use std::str;

use crate::api::schema::*;
use crate::qdrant::client::*;

// pub type Result<T> = core::result::Result<T, Error>;

// pub type Error = Box<dyn std::error::Error>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QueryDetails {
    #[serde(rename = "category")]
    pub category: String,

    #[serde(rename = "query")]
    pub query: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ResponseDetails {
    #[serde(rename = "status")]
    pub status: String,

    #[serde(rename = "query")]
    pub query: Option<String>,

    #[serde(rename = "data")]
    pub data: String,

    #[serde(rename = "score")]
    pub score: String,
}

#[derive(Clone, Copy, Debug)]
pub struct ImplPayloadInterface {}

pub trait PayloadInterface {
    async fn payload(
        &self,
        log: &Logging,
        config: ApplicationConfig,
        data: String,
    ) -> Result<ResponseDetails, Box<dyn std::error::Error>>;
}

impl PayloadInterface for ImplPayloadInterface {
    async fn payload(
        &self,
        log: &Logging,
        config: ApplicationConfig,
        query: String,
    ) -> Result<ResponseDetails, Box<dyn std::error::Error>> {
        let result: ResponseDetails;
        // use config to create both
        // ollama client and qdrant client
        // setup qdrant client
        let client = Qdrant::from_url(&format!(
            "{}:{}",
            config.clone().spec.qdrant_url,
            config.clone().spec.qdrant_port
        ))
        .build();

        if client.is_err() {
            let res_err = ResponseDetails {
                status: "KO".to_string(),
                query: None,
                score: 0.0.to_string(),
                data: format!("qdrant {:#?}", client.err()),
            };
            return Ok(res_err);
        }

        let qclient = VectorDB::new(client.unwrap());
        let ollama = Ollama::new(config.spec.ollama_url, config.spec.ollama_port as u16);
        log.debug(&format!("ollama connection {:#?}", ollama));

        let res = ollama
            .generate_embeddings(config.spec.model, query.clone(), None)
            .await?;

        let vecdb_res = qclient.search(config.spec.category, res).await?;
        if !vecdb_res.payload.is_empty() {
            log.info(&format!("score {:#?}", vecdb_res.score));
            if vecdb_res.score > config.spec.score_threshold {
                let v = vecdb_res.payload["id"].as_str().unwrap().clone();
                let markdown_data = fs::read_to_string(v)?;
                result = ResponseDetails {
                    status: "OK".to_string(),
                    query: Some(query.clone()),
                    score: vecdb_res.score.clone().to_string(),
                    data: markdown_data,
                };
            } else {
                result = ResponseDetails {
                    status: "KO".to_string(),
                    query: Some(query.clone()),
                    score: 0.0.to_string(),
                    data: "I could not find any related info, please refine your prompt"
                        .to_string(),
                };
            }
        } else {
            result = ResponseDetails {
                status: "KO".to_string(),
                query: Some(query.clone()),
                score: 0.0.to_string(),
                data: "I could not find any related info, please refine your prompt".to_string(),
            };
        }
        Ok(result)
    }
}

/// handler - reads json as input
pub async fn process_payload<T: PayloadInterface>(
    req: Request<hyper::body::Incoming>,
    log: &Logging,
    config: ApplicationConfig,
    q: T,
) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error + Send + Sync>> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/query") => {
            let max = req.body().size_hint().upper().unwrap_or(u64::MAX);
            if max > 1024 * 64 {
                let mut resp = Response::new(Full::new(Bytes::from("body too big")));
                *resp.status_mut() = hyper::StatusCode::PAYLOAD_TOO_LARGE;
                return Ok(resp);
            }

            let req_body = req.collect().await?.to_bytes();
            let payload = String::from_utf8(req_body.to_vec()).unwrap();
            log.info(&format!("payload {:#?}", payload));
            let query_json: QueryDetails = serde_json::from_str(&payload).unwrap();
            let res = q.payload(log, config, query_json.query.clone()).await;
            let resp_json = serde_json::to_string(&res.unwrap()).unwrap();
            Ok(Response::new(Full::new(Bytes::from(resp_json))))
        }
        // health endpoint
        (&Method::GET, "/isalive") => {
            let resp_details = ResponseDetails {
                status: "OK".to_string(),
                score: 0.0.to_string(),
                query: None,
                data: "service is up".to_string(),
            };
            let resp_json = serde_json::to_string(&resp_details).unwrap();
            Ok(Response::new(Full::new(Bytes::from(resp_json))))
        }
        // all other routes
        _ => {
            let resp_details = ResponseDetails {
                status: "KO".to_string(),
                score: 0.0.to_string(),
                query: None,
                data: "ensure you post to the /query endpoint with valid json".to_string(),
            };
            let resp_json = serde_json::to_string(&resp_details).unwrap();
            Ok(Response::new(Full::new(Bytes::from(resp_json))))
        }
    }
}

/*
#[cfg(test)]
mod tests {
    // this brings everything from parent's scope into this scope
    use super::*;
    use hyper::{Body, Request, Uri};
    use serial_test::serial;
    use std::fs;
    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e).unwrap()
        };
    }
    struct Mock {}
    impl MessageQueueInterface for Mock {
        fn publish(
            &self,
            log: &Logging,
            json_data: String,
            _host: String,
            _topic: String,
        ) -> Result<(), Box<dyn std::error::Error>> {
            log.info("testing queue publish");
            log.info(&format!("data {:#?}", json_data));
            Ok(())
        }
    }
    #[test]
    #[serial]
    fn test_handler_post_setvars_pass() {
        let log = &Logging {
            log_level: Level::TRACE,
        };
        env::remove_var("REDIS_HOST");
        env::remove_var("TOPIC");
        let tst = Mock {};
        let payload = fs::read_to_string("./payload.json").expect("should read payload.json file");
        let req = Request::new(Body::from(payload));
        let uri = "https://www.rust-lang.org/publish".parse::<Uri>().unwrap();
        let (mut parts, body) = req.into_parts();
        parts.method = Method::POST;
        parts.uri = uri;
        let request = Request::from_parts(parts, body);
        aw!(process_payload(request, log, tst));
    }
    #[test]
    #[serial]
    fn test_handler_post_novars_pass() {
        let log = &Logging {
            log_level: Level::TRACE,
        };
        let tst = Mock {};
        env::set_var("REDIS_HOST", "redis://test");
        env::set_var("TOPIC", "test");
        let payload = fs::read_to_string("./payload.json").expect("should read payload.json file");
        let req = Request::new(Body::from(payload));
        let uri = "https://www.rust-lang.org/publish".parse::<Uri>().unwrap();
        let (mut parts, body) = req.into_parts();
        parts.method = Method::POST;
        parts.uri = uri;
        let request = Request::from_parts(parts, body);
        aw!(process_payload(request, log, tst));
    }
    #[test]
    #[serial]
    fn test_handler_get_pass() {
        let log = &Logging {
            log_level: Level::INFO,
        };
        let tst = Mock {};
        let req = Request::new(Body::from("ok"));
        let uri = "https://www.rust-lang.org/isalive".parse::<Uri>().unwrap();
        let (mut parts, body) = req.into_parts();
        parts.method = Method::GET;
        parts.uri = uri;
        let request = Request::from_parts(parts, body);
        aw!(process_payload(request, log, tst));
        env::remove_var("REDIS_HOST");
        env::remove_var("TOPIC");
    }
    #[test]
    fn test_handler_other_pass() {
        let log = &Logging {
            log_level: Level::INFO,
        };
        let tst = Mock {};
        let req = Request::new(Body::from("please check your payload"));
        let uri = "https://www.rust-lang.org/".parse::<Uri>().unwrap();
        let (mut parts, body) = req.into_parts();
        parts.method = Method::PUT;
        parts.uri = uri;
        let request = Request::from_parts(parts, body);
        aw!(process_payload(request, log, tst));
    }
}
*/
