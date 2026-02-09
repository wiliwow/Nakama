use ollama_rs::{
    generation::completion::{request::GenerationRequest, GenerationResponseStream},
    Ollama,
};
use tokio_stream::StreamExt;

pub struct AICompanion {
    client: Ollama,
    model: String,
}

impl AICompanion {
    /// Create a new AICompanion that will use the given Ollama model (for example "deepseek:latest").
    pub fn new(model: String) -> Self {
        Self {
            client: Ollama::default(),
            model,
        }
    }

    /// Clone the configured model name.
    pub fn model_clone(&self) -> String {
        self.model.clone()
    }

    /// Process a single user input and return the generated text.
    pub async fn process_request(&self, user_input: String) -> Result<String, String> {
        let request = GenerationRequest::new(self.model.clone(), user_input);

        let mut stream: GenerationResponseStream = self
            .client
            .generate_stream(request)
            .await
            .map_err(|e| e.to_string())?;

        let mut result = String::new();

        while let Some(chunk_res) = stream.next().await {
            let chunk = chunk_res.map_err(|e| e.to_string())?;
            for piece in chunk {
                result.push_str(&piece.response);
            }
        }

        Ok(result)
    }
}