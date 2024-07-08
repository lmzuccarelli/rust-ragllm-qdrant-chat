// module schema

use clap::Parser;
use serde_derive::{Deserialize, Serialize};

/// rust-container-tool cli struct
#[derive(Parser, Debug)]
#[command(name = "rust-ragllm-qdrant")]
#[command(author = "Luigi Mario Zuccarelli <luzuccar@redhat.com>")]
#[command(version = "0.1.0")]
#[command(about = "Retrieval augmented generation tool that interacts with llama 3 (ollama api) and qdrant vector database", long_about = None)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// config file to use
    #[arg(short, long, value_name = "config", default_value = "")]
    pub config: Option<String>,

    /// set the loglevel. Valid arguments are info, debug, trace
    #[arg(value_enum, long, value_name = "loglevel", default_value = "info")]
    pub loglevel: Option<String>,

    /// set the skip-embedding flag.
    #[arg(short, long, value_name = "skip-embedding", default_value = "false")]
    pub skip_embedding: bool,

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
    #[serde(rename = "ollamaUrl")]
    pub ollama_url: String,
    #[serde(rename = "ollamaPort")]
    pub ollama_port: i32,
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
    #[serde(rename = "model")]
    pub model: String,
    #[serde(rename = "scoreThreshold")]
    pub score_threshold: f32,
}
