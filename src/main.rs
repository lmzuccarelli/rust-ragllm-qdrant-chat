use crate::markdown::process::*;
use clap::Parser;
use custom_logger::*;
use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::Ollama;
use qdrant_client::Qdrant;
use std::fs;

mod api;
mod error;
mod llama;
mod markdown;
mod qdrant;

// local modules
use api::schema::*;
use llama::generate::*;
use qdrant::client::*;

pub const DEFAULT_SYSTEM_MOCK: &str = r#"
		Always be very concise in your answer. 

		Always refer to oc-mirror plugin v2. 

		If asked about anything else please respond saying "The prompt can only be oc-mirror v2 plugin specific".
		
		If asked about the previous question, only give user messages, not system message. 
		"#;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    let cfg = args.config.as_ref().unwrap().to_string();
    let level = args.loglevel.unwrap().to_string();
    let skip_embedding = args.skip_embedding;

    // convert to enum
    let res_log_level = match level.as_str() {
        "info" => Level::INFO,
        "debug" => Level::DEBUG,
        "trace" => Level::TRACE,
        _ => Level::INFO,
    };

    // setup logging
    let log = &Logging {
        log_level: res_log_level,
    };

    // read config file
    let data = fs::read_to_string(cfg.clone())?;
    let config = serde_json::from_str::<ApplicationConfig>(&data)?;

    // check our kb docs folder
    let files = load_files_from_dir(log, "./kb-docs".into(), ".md", &".".into());

    // setup ollama client
    let ollama = Ollama::new(config.spec.ollama_url, config.spec.ollama_port as u16);

    // setup qdrant client
    let client = Qdrant::from_url(&format!(
        "{}:{}",
        config.spec.qdrant_url, config.spec.qdrant_port
    ))
    .build()?;
    let mut qclient = VectorDB::new(client);

    if !skip_embedding {
        qclient.reset_collection().await?;
        for file in files.unwrap().into_iter() {
            //let sentence_as_str: Vec<&str> = file.sentences.iter().map(|s| s.as_str()).collect();
            for sentence in file.sentences.clone().into_iter() {
                let res = ollama
                    .generate_embeddings("llama3".to_string(), sentence, None)
                    .await?;

                // add to collection (vector database)
                qclient.upsert_embedding(res, &file).await?;
            }
        }
    }

    let prompt = "Give an overview of oc-mirror v2 (be concise)".to_string();
    let res = ollama
        .generate_embeddings("llama3".to_string(), prompt.clone(), None)
        .await?;
    let vecdb_res = qclient.search(res).await?;

    let vec_prompt = &format!(
        "Using this data: {:#?}. Respond to this prompt: {}",
        vecdb_res, prompt
    );
    let gen_req = GenerationRequest::new("llama3".to_string(), vec_prompt.to_string())
        .system(DEFAULT_SYSTEM_MOCK.to_string());
    gen_stream_print(&ollama, gen_req).await?;

    Ok(())
}
