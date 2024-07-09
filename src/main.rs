use crate::markdown::process::*;
use clap::Parser;
use custom_logger::*;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use ollama_rs::Ollama;
use qdrant_client::Qdrant;
use std::fs;
use std::net::SocketAddr;
use std::process::exit;
use tokio::net::TcpListener;

mod api;
mod error;
mod handler;
mod llama;
mod markdown;
mod qdrant;

// local modules
use api::schema::*;
use handler::payload::*;
use qdrant::client::*;

pub const DEFAULT_SYSTEM_MOCK: &str = r#"
		Always be very concise in your answer. 

		Always refer to oc-mirror plugin v2. 

		If asked about anything else please respond saying "The prompt can only be oc-mirror v2 plugin specific".
		"#;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = Cli::parse();
    let cfg = args.config.as_ref().unwrap().to_string();
    //let level = args.loglevel.unwrap().to_string();
    let skip_embedding = args.skip_embedding;

    // setup logging
    let log = &Logging {
        log_level: Level::DEBUG,
    };

    // read config file
    let cfg_data = fs::read_to_string(cfg.clone())?;
    let cfg = serde_json::from_str::<ApplicationConfig>(&cfg_data.clone())?;

    if !skip_embedding {
        // setup qdrant client
        let client = Qdrant::from_url(&format!(
            "{}:{}",
            cfg.clone().spec.qdrant_url,
            cfg.clone().spec.qdrant_port
        ))
        .build();

        if client.is_err() {
            log.error(&format!("qdrant {:#?}", client.err()));
            exit(1);
        }

        log.debug(&format!(
            "qdrant {}:{}",
            cfg.spec.qdrant_url, cfg.spec.qdrant_port
        ));

        let mut qclient = VectorDB::new(client.unwrap());
        // setup ollama client
        let ollama = Ollama::new(cfg.spec.ollama_url, cfg.spec.ollama_port as u16);

        log.debug(&format!("ollama connection {:#?}", ollama));

        log.info("executing embedding workflow");
        // check our kb docs folder
        let folder = &format!(
            "{}/{}",
            cfg.spec.kb_docs_path.clone(),
            cfg.spec.category.clone()
        );
        let files = load_files_from_dir(log, folder.into(), ".md", &".".into());

        let result = qclient.reset_collection(cfg.spec.category.clone()).await;
        if result.is_err() {
            log.error(&format!("qdrant reset collection {:#?}", result.err()));
            exit(1);
        }
        for file in files.unwrap().into_iter() {
            for sentence in file.headings.clone().into_iter() {
                let res = ollama
                    .generate_embeddings(cfg.spec.model.clone(), sentence, None)
                    .await;

                if res.is_err() {
                    log.error(&format!("embeddings {:#?}", res.err().unwrap()));
                    exit(1);
                }

                // add to collection (vector database)
                let res = qclient
                    .upsert_embedding(cfg.spec.category.clone(), res.unwrap(), &file)
                    .await;
                if res.is_err() {
                    log.error(&format!("upsert {:#?}", res.err().unwrap()));
                    exit(1);
                }
            }
        }
        log.info("completed embedding workflow");
        exit(0);
    }

    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.spec.server_port));
    let listener = TcpListener::bind(addr).await?;
    let real = ImplPayloadInterface {};

    loop {
        let (stream, _) = listener.accept().await?;
        let config = serde_json::from_str::<ApplicationConfig>(&cfg_data.clone())?;
        let io = TokioIo::new(stream);
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(move |req| process_payload(req, log, config.clone(), real)),
                )
                .await
            {
                log.error(&format!("serving connection: {:}", err));
            }
        });
    }
}
