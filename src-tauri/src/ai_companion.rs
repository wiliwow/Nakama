use reqwest::Client as HttpClient;
use serde_json::json;
use std::env;
use tauri::Emitter;
use tracing::warn;
use crate::conversation::ConversationMessage;

fn llm_endpoint() -> String {
    env::var("LLM_ENDPOINT")
        .or_else(|_| env::var("LLM_BASE_URL"))
        .unwrap_or_else(|_| "".to_string())
}

fn llm_api_key() -> Option<String> {
    env::var("LLM_API_KEY")
        .or_else(|_| env::var("OPENAI_API_KEY"))
        .ok()
}

fn llm_model() -> String {
    env::var("LLM_MODEL").unwrap_or_else(|_| "".to_string())
}

fn strip_data_url(data_url: &str) -> &str {
    data_url
        .strip_prefix("data:image/png;base64,")
        .or_else(|| data_url.strip_prefix("data:image/jpeg;base64,"))
        .or_else(|| data_url.strip_prefix("data:image/webp;base64,"))
        .unwrap_or(data_url)
}

#[derive(Clone, Debug, serde::Deserialize)]
struct ChatChoice {
    pub delta: StreamDelta,
    pub finish_reason: Option<String>,
}

#[derive(Clone, Debug, serde::Deserialize)]
struct StreamDelta {
    pub content: Option<String>,
}

#[derive(Clone, Debug, serde::Deserialize)]
struct ChatResponse {
    pub choices: Vec<ChatChoice>,
}

pub struct AICompanion {
    client: HttpClient,
    endpoint: String,
    api_key: Option<String>,
    model: String,
}

impl AICompanion {
    pub fn new(model: String) -> Self {
        let endpoint = llm_endpoint();
        let api_key = llm_api_key();
        let model = if model.is_empty() { llm_model() } else { model };
        Self {
            client: HttpClient::new(),
            endpoint,
            api_key,
            model,
        }
    }

    pub fn is_configured(&self) -> bool {
        !self.endpoint.is_empty() && !self.model.is_empty()
    }

    pub fn model_clone(&self) -> String {
        self.model.clone()
    }

    pub fn build_prompt_with_context(&self, prompt: String, history: &[ConversationMessage]) -> String {
        if history.is_empty() {
            return prompt;
        }
        let mut sys = String::from(
            "You are a helpful AI assistant that can control the user's computer through mouse and keyboard actions.\n\n\
             You can execute computer control actions by including specific commands in your response:\n\
             - Mouse movement: \"move to 100,200\" or \"go to coordinates (100,200)\"\n\
             - Clicking: \"click at 100,200\" or \"right click\" or \"double click\"\n\
             - Dragging: \"drag from 100,100 to 200,200\"\n\
             - Scrolling: \"scroll up 50\" or \"scroll down 100\"\n\
             - Typing: \"type hello world\"\n\
             - Key presses: \"press enter\" or \"hit escape\" or \"press ctrl+c\" or \"ctrl c\"\n\n\
             When the user asks you to perform an action, describe what you're doing AND include the command.\n\
             Answer user questions clearly and directly.\n\nHere is the conversation history:\n\n",
        );
        for m in history {
            let role = match m.role.as_str() {
                "user" => "User",
                "assistant" => "Assistant",
                _ => "Unknown",
            };
            use std::fmt::Write;
            let _ = writeln!(sys, "{}: {}", role, m.content);
        }
        use std::fmt::Write;
        let _ = writeln!(sys, "\nUser: {}\n\nAssistant:", prompt);
        sys
    }

    fn build_chat_messages(
        &self,
        prompt: String,
        history: &[ConversationMessage],
        image_b64: Option<&str>,
    ) -> Vec<serde_json::Value> {
        let mut msgs: Vec<serde_json::Value> = history
            .iter()
            .map(|m| {
                let role = if m.role == "assistant" { "assistant" } else { "user" };
                json!({ "role": role, "content": m.content })
            })
            .collect();

        if let Some(raw) = image_b64 {
            let b64 = if raw.starts_with("data:") { strip_data_url(raw) } else { raw };
            msgs.push(json!({
                "role": "user",
                "content": [
                    { "type": "text", "text": prompt },
                    { "type": "image_url", "image_url": { "url": format!("data:image/png;base64,{}", b64) } }
                ]
            }));
        } else {
            msgs.push(json!({ "role": "user", "content": prompt }));
        }
        msgs
    }

    async fn post_chat(&self, messages: Vec<serde_json::Value>, stream: bool) -> Result<reqwest::Response, String> {
        let url = format!("{}/v1/chat/completions", self.endpoint);
        let body = json!({
            "model": self.model,
            "messages": messages,
            "stream": stream,
        });
        self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key.as_deref().unwrap_or("")))
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                warn!(error = %e, url = %url, stream = stream, "LLM: /v1/chat/completions POST failed");
                format!("LLM request failed: {}", e)
            })
    }

    pub async fn process_request(&self, user_input: String) -> Result<String, String> {
        self.process_request_with_images(user_input, None, &[])
            .await
    }

    pub async fn process_request_with_context(
        &self,
        user_input: String,
        conversation_history: &[ConversationMessage],
    ) -> Result<String, String> {
        let prompt = self.build_prompt_with_context(user_input, conversation_history);
        self.process_request_with_images(prompt, None, conversation_history)
            .await
    }

    pub async fn process_request_with_images(
        &self,
        prompt: String,
        image_b64: Option<&str>,
        conversation_history: &[ConversationMessage],
    ) -> Result<String, String> {
        if !self.is_configured() {
            return Err("No LLM endpoint configured. Set LLM_ENDPOINT and LLM_MODEL environment variables.".into());
        }
        let messages = self.build_chat_messages(prompt, conversation_history, image_b64);
        let resp = self.post_chat(messages, false).await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            warn!(status = %status, "LLM: non-success status");
            return Err(format!("LLM error {}: {}", status, text));
        }

        let body_text = resp.text().await.unwrap_or_default();
        let parsed: ChatResponse = serde_json::from_str(&body_text)
            .map_err(|e| {
                warn!(error = %e, "LLM: JSON parse failed");
                format!("LLM JSON parse error: {} — raw body: {}", e,
                    body_text.chars().take(500).collect::<String>())
            })?;


        Ok(parsed.choices.into_iter().next().and_then(|c| c.delta.content).unwrap_or_default())
    }

    pub async fn stream_chat_with_images(
        &self,
        model: String,
        prompt: String,
        history: &[ConversationMessage],
        image_b64: Option<&str>,
        window: &tauri::Window,
    ) -> Result<(), String> {
        if !self.is_configured() {
            let _ = window.emit("ai-stream-error", "No LLM endpoint configured.".to_string());
            let _ = window.emit("ai-stream-done", ());
            return Ok(());
        }
        let messages = self.build_chat_messages(prompt, history, image_b64);
        let resp = self.post_chat(messages, true).await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            warn!(status = %status, "LLM: streaming returned non-success");
            let _ = window.emit("ai-stream-error", format!("LLM returned {}: {}", status, text));
            let _ = window.emit("ai-stream-done", ());
            return Ok(());
        }

        let body = resp.text().await.map_err(|e| {
            warn!(error = %e, "LLM: stream body read failed");
            format!("LLM stream body error: {}", e)
        })?;

        for line in body.lines().filter(|l| !l.trim().is_empty()) {
            if line.starts_with("data: ") {
                let data = &line[6..];
                if data == "[DONE]" {
                    let _ = window.emit("ai-stream-done", ());
                    return Ok(());
                }
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(data) {
                    if let Some(err_val) = val.get("error") {
                        let err_type_str = val.get("error_type").and_then(|v| v.as_str()).unwrap_or("unknown");
                        let preview = err_val.as_str().unwrap_or_default().chars().take(200).collect::<String>();
                        warn!(error = %preview, error_type = %err_type_str, "LLM: error event in streaming");
                        let _ = window.emit("ai-stream-error", format!("LLM error ({}): {}", err_type_str, preview));
                        let _ = window.emit("ai-stream-done", ());
                        return Ok(());
                    }
                    if let Some(content) = val["choices"][0]["delta"]["content"].as_str() {
                        let _ = window.emit("ai-stream-chunk", content.to_string());
                    }
                }
            } else if !line.starts_with(":") {
                warn!(raw = %line, "LLM: unexpected line in streaming");
            }
        }

        let _ = window.emit("ai-stream-done", ());
        Ok(())
    }

    pub async fn generate_completion(
        &self,
        prompt: String,
        history: &[ConversationMessage],
        image_b64: Option<&str>,
    ) -> Result<String, String> {
        if !self.is_configured() {
            return Err("No LLM endpoint configured. Set LLM_ENDPOINT and LLM_MODEL environment variables.".into());
        }
        let messages = self.build_chat_messages(prompt, history, image_b64);
        let resp = self.post_chat(messages, false).await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("LLM error {}: {}", status, text));
        }

        let body_text = resp.text().await.unwrap_or_default();
        let parsed: ChatResponse = serde_json::from_str(&body_text)
            .map_err(|e| format!("LLM JSON parse error: {} — raw body: {}", e, 
                body_text.chars().take(500).collect::<String>()))?;


        Ok(parsed.choices.into_iter().next().and_then(|c| c.delta.content).unwrap_or_default())
    }
}
