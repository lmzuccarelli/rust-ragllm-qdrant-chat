use crate::markdown::process::*;
use clap::Parser;
use custom_logger::*;
use ollama_rs::Ollama;
use qdrant_client::Qdrant;
use std::fs;
use std::process::exit;

mod api;
mod error;
mod llama;
mod markdown;
mod qdrant;

// local modules
use api::schema::*;
use qdrant::client::*;

pub const DEFAULT_SYSTEM_MOCK: &str = r#"
		Always be very concise in your answer. 

		Always refer to oc-mirror plugin v2. 

		If asked about anything else please respond saying "The prompt can only be oc-mirror v2 plugin specific".
		"#;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    let cfg = args.config.as_ref().unwrap().to_string();
    let level = args.loglevel.unwrap().to_string();
    let skip_embedding = args.skip_embedding;
    let user_prompt = args.user_prompt.as_ref().unwrap().to_string();

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

    // setup ollama client
    let ollama = Ollama::new(config.spec.ollama_url, config.spec.ollama_port as u16);
    log.debug(&format!("ollama connection {:#?}", ollama));

    // setup qdrant client
    let client = Qdrant::from_url(&format!(
        "{}:{}",
        config.spec.qdrant_url, config.spec.qdrant_port
    ))
    .build();

    if client.is_err() {
        log.error(&format!("qdrant {:#?}", client.err()));
        exit(1);
    }
    let mut qclient = VectorDB::new(client.unwrap());

    log.debug(&format!(
        "qdrant {}:{}",
        config.spec.qdrant_url, config.spec.qdrant_port
    ));

    if !skip_embedding {
        log.info("executing embedding workflow");
        // check our kb docs folder
        let files = load_files_from_dir(log, config.spec.kb_docs_path.into(), ".md", &".".into());

        let result = qclient.reset_collection(config.spec.category.clone()).await;
        if result.is_err() {
            log.error(&format!("qdrant reset collection {:#?}", result.err()));
            exit(1);
        }
        for file in files.unwrap().into_iter() {
            for sentence in file.headings.clone().into_iter() {
                let res = ollama
                    .generate_embeddings(config.spec.model.clone(), sentence, None)
                    .await?;

                // add to collection (vector database)
                qclient
                    .upsert_embedding(config.spec.category.clone(), res, &file)
                    .await?;
            }
        }
    }

    let prompt = user_prompt;
    let res = ollama
        .generate_embeddings(config.spec.model, prompt.clone(), None)
        .await?;

    let vecdb_res = qclient.search(config.spec.category, res).await?;
    if !vecdb_res.payload.is_empty() {
        log.info(&format!("score {:#?}", vecdb_res.score));
        if vecdb_res.score > config.spec.score_threshold {
            let v = vecdb_res.payload["id"].as_str().unwrap().clone();
            let data = fs::read_to_string(v)?;
            log.hi(&format!(
                "Here is my response to your prompt [ {} ]\n",
                prompt
            ));
            println!("{}", data);
        } else {
            log.warn(&format!(
                "I could not find any related info, please refine your prompt [ {} ]",
                prompt
            ));
        }
    } else {
        log.warn(&format!(
            "I could not find any related info, please refine your prompt [ {} ]",
            prompt
        ));
    }

    Ok(())
}
