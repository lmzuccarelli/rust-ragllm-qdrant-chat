use crate::{llamacpp::generate::get_embeddings, qdrant::client::VectorDB};
use custom_logger as log;
use std::{
    io::{self, Write},
    sync::Arc,
};

use crate::chat::{client::ChatClient, model::CompletionRequest, model::Message};

#[allow(unused)]
pub struct ChatSession {
    qclient: VectorDB,
    client: Arc<dyn ChatClient>,
    model: String,
    embedding_url: String,
    category: String,
    messages: Vec<Message>,
    search_limit: u64,
    score_threshold: f32,
}

impl ChatSession {
    pub fn new(
        qclient: VectorDB,
        client: Arc<dyn ChatClient>,
        model: String,
        url: String,
        category: String,
        search_limit: u64,
        score_threshold: f32,
    ) -> Self {
        Self {
            qclient,
            client,
            model,
            embedding_url: url,
            category,
            messages: Vec::new(),
            search_limit,
            score_threshold,
        }
    }

    pub fn add_system_prompt(&mut self, prompt: impl ToString) {
        self.messages.push(Message::system(prompt));
    }

    pub async fn chat(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("welcome!! input your question at the prompt. Use 'exit' to quit");

        let mut prompt = "A chat between a curious human and an artificial intelligence assistant. The assistant gives helpful, detailed, and polite answers to the human's questions.".to_owned();
        //### Human: Hello, Assistant.
        //### Assistant: Hello. How may I help you today?
        //### Human: Please tell me the largest city in Europe.
        //### Assistant: Sure. The largest city in Europe is Moscow, the capital of Russia.".to_owned();

        loop {
            print!("> ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            input = input.trim().to_string();

            if input.is_empty() {
                continue;
            }

            if input == "exit" {
                break;
            }

            let res = get_embeddings(self.embedding_url.clone(), input.clone()).await;
            let qdrant_res = self
                .qclient
                .search(self.category.clone(), res.unwrap(), self.search_limit)
                .await;

            let mut extra_prompt =
                "\nAnswer the question based only on the following context:\n\n".to_string();

            let question = "Summarize the answer based on the above context: ".to_string();
            let mut source = "".to_string();
            let mut found = false;
            for result in qdrant_res.as_ref().unwrap().iter() {
                if result.score > self.score_threshold {
                    let map = result.payload.clone();
                    log::info!("score {}", result.score);
                    let content = map["contents"].as_str().unwrap();
                    extra_prompt.push_str(&content.clone());
                    extra_prompt.push_str("\n --- \n");
                    source.push_str(&format!("{} ", map["id"].as_str().unwrap()));
                    found = true;
                }
            }

            if found {
                prompt.push_str(&extra_prompt);
                prompt.push_str(&format!(
                    "\n### Human: {} {}\n### Assitant:",
                    question, input
                ));
            } else {
                // create request
                let end_prompt = format!("\n### Human: {}\n### Assistant:", input);
                prompt.push_str(&end_prompt);
            }

            log::info!("messages : {}", prompt.clone());

            let request = CompletionRequest {
                messages: self.messages.clone(),
                prompt: prompt.clone(),
                top_k: 20,
                top_p: 0.7,
                n_keep: 68,
                n_predict: 256,
                cache_prompt: false,
                stop: vec!["\n".to_string(), "### Human:".to_string()],
                temperature: Some(0.2),
                stream: true,
                max_tokens: 2048,
            };

            // send request
            let _response = self.client.complete(request).await?;
            if found {
                log::info!("sources : {}", source);
            }

            prompt = String::new();
        }
        Ok(())
    }
}
