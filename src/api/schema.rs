// module schema

use clap::Parser;
use serde_derive::{Deserialize, Serialize};

/// rust-container-tool cli struct
#[derive(Parser, Debug)]
#[command(name = "rust-ragllm-qdrant-chat")]
#[command(author = "Luigi Mario Zuccarelli <luzuccar@redhat.com>")]
#[command(version = "0.1.0")]
#[command(about = "Retrieval augmented generation tool that interacts with llama.cpp server for both model and embeddings service, with a qdrant vector database", long_about = None)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// config file to use
    #[arg(short, long, value_name = "config", default_value = "")]
    pub config: Option<String>,

    /// set the loglevel. Valid arguments are info, debug, trace
    #[arg(value_enum, long, value_name = "loglevel", default_value = "info")]
    pub loglevel: Option<String>,

    /// set the use chat client flag.
    #[arg(short, long, value_name = "chat-client", default_value = "false")]
    pub chat_client: bool,

    /// set the user prompt (used for debugging).
    #[arg(short, long, value_name = "user-prompt", default_value = "")]
    pub user_prompt: Option<String>,
}

/// Application configuration
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApplicationConfig {
    #[serde(rename = "kind")]
    pub kind: String,
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub spec: Spec,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Spec {
    #[serde(rename = "proxy")]
    pub proxy: bool,
    #[serde(rename = "openApiKey")]
    pub openapi_key: String,
    #[serde(rename = "llamacppUrl")]
    pub llamacpp_url: String,
    #[serde(rename = "llamacppPort")]
    pub llamacpp_port: i32,
    #[serde(rename = "llamacppEmbeddingUrl")]
    pub llamacpp_embedding_url: String,
    #[serde(rename = "llamacppEmbeddingPort")]
    pub llamacpp_embedding_port: i32,
    #[serde(rename = "qdrantUrl")]
    pub qdrant_url: String,
    #[serde(rename = "qdrantPort")]
    pub qdrant_port: i32,
    #[serde(rename = "category")]
    pub category: String,
    #[serde(rename = "kbDocsPath")]
    pub kb_docs_path: String,
    #[serde(rename = "serverPort")]
    pub server_port: u16,
    #[serde(rename = "embeddingModel")]
    pub embedding_model: String,
    #[serde(rename = "servingModel")]
    pub serving_model: String,
    #[serde(rename = "scoreThreshold")]
    pub score_threshold: f32,
    #[serde(rename = "useHeaders")]
    pub use_headers: bool,
    #[serde(rename = "fileExtension")]
    pub file_extension: String,
    #[serde(rename = "headerRegex")]
    pub header_regex: Option<String>,
    #[serde(rename = "searchLimit")]
    pub search_limit: u64,
}
