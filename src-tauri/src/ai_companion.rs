use ollama_rs::{
    generation::completion::{request::GenerationRequest, GenerationResponseStream},
    Ollama,
};
use tokio_stream::StreamExt;
use crate::conversation::ConversationMessage;

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

    /// Build a prompt with conversation context
    pub fn build_prompt_with_context(&self, user_input: String, conversation_history: &[ConversationMessage]) -> String {
        if conversation_history.is_empty() {
            return user_input;
        }

        let mut prompt = String::from("You are a helpful AI assistant. Here is the conversation history:\n\n");

        for message in conversation_history {
            let role = match message.role.as_str() {
                "user" => "User",
                "assistant" => "Assistant",
                _ => "Unknown",
            };
            prompt.push_str(&format!("{}: {}\n\n", role, message.content));
        }

        prompt.push_str(&format!("User: {}\n\nAssistant:", user_input));
        prompt
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

    /// Process a user input with conversation context
    pub async fn process_request_with_context(
        &self,
        user_input: String,
        conversation_history: &[ConversationMessage]
    ) -> Result<String, String> {
        let prompt = self.build_prompt_with_context(user_input, conversation_history);
        let request = GenerationRequest::new(self.model.clone(), prompt);

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
