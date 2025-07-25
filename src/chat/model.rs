use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[allow(unused)]
impl Message {
    pub fn system(content: impl ToString) -> Self {
        Self {
            role: "system".to_string(),
            content: content.to_string(),
        }
    }

    pub fn user(content: impl ToString) -> Self {
        Self {
            role: "user".to_string(),
            content: content.to_string(),
        }
    }

    #[allow(unused)]
    pub fn assistant(content: impl ToString) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompletionRequest {
    //pub model: String,
    pub messages: Vec<Message>,
    pub prompt: String,
    pub top_k: usize,
    pub top_p: f32,
    pub n_keep: usize,
    pub n_predict: usize,
    pub cache_prompt: bool,
    pub stop: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    pub stream: bool,
    pub max_tokens: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SimpleCompletionResponse {
    pub data: DataResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataResponse {
    //pub index: String,
    pub content: String,
    //pub tokens: Vec<usize>,
    //pub stop: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: Message,
    pub finish_reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub contents: Vec<Content>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Content {
    pub content_type: String,
    pub body: String,
}

#[allow(unused)]
impl Content {
    pub fn text(content: impl ToString) -> Self {
        Self {
            content_type: "text/plain".to_string(),
            body: content.to_string(),
        }
    }
}
