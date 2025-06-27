use crate::chat::client::OpenAIClient;
use crate::chat::process::ChatSession;
use crate::error::handler::EmbeddingsError;
use crate::markdown::process::*;
use clap::Parser;
use custom_logger as log;
use llamacpp::generate::get_embeddings;
use qdrant_client::Qdrant;
use std::process::exit;
use std::sync::Arc;
use std::time::Instant;
use std::{fs, str::FromStr};

mod api;
mod chat;
mod error;
mod llamacpp;
mod markdown;
mod qdrant;

// local modules
use api::schema::*;
use qdrant::client::*;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = Cli::parse();
    let cfg = args.config.as_ref().unwrap().to_string();
    let level = args.loglevel.unwrap();
    let res_log_level = log::LevelFilter::from_str(&level)
        .map_err(|_| EmbeddingsError::new(&format!("invalid log level \"{level}\"")))?;

    let chat_client = args.chat_client;

    // setup logging
    log::Logging::new()
        .with_level(res_log_level)
        .init()
        .expect("should initialize");

    // read config file
    let cfg_data = fs::read_to_string(cfg.clone())?;
    let cfg = serde_json::from_str::<ApplicationConfig>(&cfg_data.clone())?;

    // setup qdrant client
    let client = Qdrant::from_url(&format!(
        "{}:{}",
        cfg.clone().spec.qdrant_url,
        cfg.clone().spec.qdrant_port
    ))
    .build();

    if client.is_err() {
        log::error!("qdrant {:#?}", client.err());
        exit(1);
    }

    log::debug!("qdrant {}:{}", cfg.spec.qdrant_url, cfg.spec.qdrant_port);
    let mut qclient = VectorDB::new(client.unwrap());
    log::info!("executing embedding workflow");

    let embedding_url = format!(
        "{}:{}/embedding",
        cfg.spec.llamacpp_embedding_url, cfg.spec.llamacpp_embedding_port
    );

    if !chat_client {
        let now = Instant::now();
        // check our kb docs folder
        let folder = &format!(
            "{}/{}",
            cfg.spec.kb_docs_path.clone(),
            cfg.spec.category.clone()
        );
        let files = load_files_from_dir(
            folder.into(),
            &cfg.spec.file_extension,
            &".".into(),
            cfg.spec.use_headers,
            cfg.spec.header_regex,
        );

        log::debug!("markdown batch {:?}", files.as_ref().unwrap());

        let category = cfg.spec.category.clone();
        let result = qclient.reset_collection(category.clone()).await;
        if result.is_err() {
            log::error!("qdrant reset collection {:#?}", result.err());
            exit(1);
        }

        for mkd in files.as_ref().unwrap().into_iter() {
            let mut contents = String::new();
            if cfg.spec.use_headers {
                contents.push_str(&mkd.headers.as_ref().unwrap().clone());
                log::info!("markdown headers {:?}", contents);
            } else {
                contents.push_str(&mkd.contents.clone());
                log::debug!("markdown contents {}", contents);
            }
            let res_embeddings = get_embeddings(embedding_url.clone(), contents.clone()).await;
            log::debug!("res embeddings {:?}", res_embeddings);
            let qdrant_res = qclient
                .upsert_embedding(category.clone(), res_embeddings.unwrap(), &mkd)
                .await;
            log::debug!("qdrant upsert results {:?}", qdrant_res);
        }
        let elapsed = now.elapsed();
        log::info!("indexed {} files", files.unwrap().len());
        log::info!("time to complete indexing : {:.2?}", elapsed);
    } else {
        // chat mode

        // create openai client
        let api_key = cfg.spec.openapi_key.clone();
        let url = format!(
            "{}:{}/completion",
            cfg.spec.llamacpp_url, cfg.spec.llamacpp_port
        );
        let model = cfg.spec.serving_model.clone();
        log::info!("url   : {:?}", url);
        log::info!("model : {:?}", model);
        let openai_client = Arc::new(OpenAIClient::new(api_key, Some(url), Some(cfg.spec.proxy)));

        // create chat session
        let mut session = ChatSession::new(
            qclient,
            openai_client,
            model,
            embedding_url,
            cfg.spec.category.clone(),
            cfg.spec.search_limit,
            cfg.spec.score_threshold,
        );

        // build system prompt with tool info
        let system_prompt = "you are an assistant, use the context to help the user:\n".to_string();

        // add system prompt
        session.add_system_prompt(system_prompt);

        // start chat
        let res = session.chat().await;
        if res.is_err() {
            log::error!("{:?}", res.as_ref().err().unwrap().to_string());
        }
    }

    Ok(())
}
