use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
}

#[derive(Deserialize)]
struct OllamaResponse {
    content: String,
}

pub struct AICompanion {
    client: Client,
    api_url: String,
}

impl AICompanion {
    pub fn new(api_url: String) -> Self {
        Self {
            client: Client::new(),
            api_url,
        }
    }

    pub async fn process_request(&self, user_input: String) -> Result<String, String> {
        let request_body = OllamaRequest {
            model: "deepseek".to_string(), // Change to "qwen" if needed
            prompt: user_input,
        };

        let response = self
            .client
            .post(&self.api_url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.status().is_success() {
            let ollama_response: OllamaResponse = response.json().await.map_err(|e| e.to_string())?;
            Ok(ollama_response.content)
        } else {
            Err(format!("Error: {}", response.status()))
        }
    }
}

pub async fn handle_ai_request(
    ai_companion: Arc<Mutex<AICompanion>>,
    user_input: String,
) -> Result<String, String> {
    let ai = ai_companion.lock().await;
    ai.process_request(user_input).await
}
