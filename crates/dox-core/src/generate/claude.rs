//! Claude (Anthropic) API integration for content generation

use super::{ContentGenerator, GenerationRequest, GenerationResponse, Usage};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

/// Claude API client for content generation
pub struct ClaudeProvider {
    client: Client,
    api_key: String,
    base_url: String,
}

impl ClaudeProvider {
    /// Create a new Claude provider
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.anthropic.com/v1".to_string(),
        }
    }

    /// Create Claude provider with custom base URL
    pub fn with_base_url(api_key: String, base_url: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url,
        }
    }

    /// Get API key from environment variable
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| anyhow!("ANTHROPIC_API_KEY environment variable not found"))?;
        Ok(Self::new(api_key))
    }

    /// Convert internal request to Claude API format
    fn build_claude_request(&self, request: &GenerationRequest) -> ClaudeMessageRequest {
        let system_message = self.build_system_message(request);
        let user_message = request.prompt.clone();

        ClaudeMessageRequest {
            model: request.model.clone(),
            max_tokens: request.max_tokens,
            temperature: if request.temperature > 0.0 {
                Some(request.temperature)
            } else {
                None
            },
            system: if !system_message.trim().is_empty() {
                Some(system_message)
            } else {
                None
            },
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: user_message,
            }],
            stream: None, // Disable streaming for now
            stop_sequences: None,
            top_p: None,
        }
    }

    /// Build system message based on content type and context
    fn build_system_message(&self, request: &GenerationRequest) -> String {
        use super::BuiltinTemplates;
        use super::TemplateEngine;

        let template = BuiltinTemplates::get_template(request.content_type, &request.language);

        let mut variables = HashMap::new();
        variables.insert("prompt".to_string(), request.prompt.clone());
        variables.insert("language".to_string(), request.language.clone());
        variables.insert("audience".to_string(), request.audience.clone());
        variables.insert("tone".to_string(), request.tone.clone());

        if let Some(context) = &request.context {
            variables.insert("context".to_string(), context.clone());
        }

        let engine = TemplateEngine::with_variables(variables);
        engine.render(&template).unwrap_or(template)
    }

    /// Send non-streaming request to Claude
    async fn send_request(
        &self,
        claude_request: &ClaudeMessageRequest,
    ) -> Result<ClaudeMessageResponse> {
        debug!(
            "Sending Claude API request: model={}, max_tokens={}",
            claude_request.model, claude_request.max_tokens
        );

        let response = self
            .client
            .post(format!("{}/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&claude_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!("Claude API error: {} - {}", status, error_text);
            return Err(anyhow!("Claude API error {}: {}", status, error_text));
        }

        let claude_response: ClaudeMessageResponse = response.json().await?;
        debug!(
            "Received Claude response: {} content blocks, usage={:?}",
            claude_response.content.len(),
            claude_response.usage
        );

        Ok(claude_response)
    }

    /// Convert Claude response to internal format
    fn convert_response(
        &self,
        claude_response: ClaudeMessageResponse,
        model: String,
    ) -> GenerationResponse {
        let content = claude_response
            .content
            .iter()
            .filter_map(|content| {
                if content.content_type == "text" {
                    Some(content.text.as_deref().unwrap_or_default())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("");

        let usage = claude_response.usage.map(|u| Usage {
            prompt_tokens: u.input_tokens,
            completion_tokens: u.output_tokens,
            total_tokens: u.input_tokens + u.output_tokens,
        });

        GenerationResponse {
            content,
            model,
            provider: "claude".to_string(),
            usage,
            metadata: HashMap::new(),
        }
    }
}

#[async_trait]
impl ContentGenerator for ClaudeProvider {
    async fn generate(&self, request: &GenerationRequest) -> Result<GenerationResponse> {
        info!("Generating content using Claude model: {}", request.model);

        // Validate model support
        if !self.supports_model(&request.model) {
            warn!(
                "Model {} not explicitly supported, but attempting anyway",
                request.model
            );
        }

        let claude_request = self.build_claude_request(request);

        // For now, only support non-streaming requests
        // TODO: Implement streaming support
        if request.stream {
            warn!("Streaming not yet implemented, falling back to non-streaming");
        }

        let claude_response = self.send_request(&claude_request).await?;
        let response = self.convert_response(claude_response, request.model.clone());

        info!(
            "Content generation completed. Generated {} characters, used {} tokens",
            response.content.len(),
            response.usage.as_ref().map(|u| u.total_tokens).unwrap_or(0)
        );

        Ok(response)
    }

    fn provider_name(&self) -> &str {
        "claude"
    }

    fn supported_models(&self) -> Vec<String> {
        vec![
            "claude-3-5-sonnet-20241022".to_string(),
            "claude-3-5-sonnet-20240620".to_string(),
            "claude-3-5-haiku-20241022".to_string(),
            "claude-3-opus-20240229".to_string(),
            "claude-3-sonnet-20240229".to_string(),
            "claude-3-haiku-20240307".to_string(),
        ]
    }
}

/// Claude Messages API request structure
#[derive(Debug, Serialize)]
struct ClaudeMessageRequest {
    model: String,
    max_tokens: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    messages: Vec<ClaudeMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
}

/// Claude message structure
#[derive(Debug, Serialize, Deserialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

/// Claude Messages API response structure
#[derive(Debug, Deserialize)]
struct ClaudeMessageResponse {
    id: String,
    #[serde(rename = "type")]
    response_type: String,
    role: String,
    content: Vec<ClaudeContent>,
    model: String,
    stop_reason: Option<String>,
    stop_sequence: Option<String>,
    usage: Option<ClaudeUsage>,
}

/// Claude content structure
#[derive(Debug, Deserialize)]
struct ClaudeContent {
    #[serde(rename = "type")]
    content_type: String,
    text: Option<String>,
}

/// Claude usage structure
#[derive(Debug, Deserialize)]
struct ClaudeUsage {
    input_tokens: usize,
    output_tokens: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate::ContentType;

    #[test]
    fn test_provider_creation() {
        let provider = ClaudeProvider::new("test-key".to_string());
        assert_eq!(provider.provider_name(), "claude");
        assert!(provider.supports_model("claude-3-5-sonnet-20241022"));
        assert!(provider.supports_model("claude-3-opus-20240229"));
        assert!(!provider.supports_model("invalid-model"));
    }

    #[test]
    fn test_system_message_building() {
        let provider = ClaudeProvider::new("test-key".to_string());
        let request = GenerationRequest {
            prompt: "Test prompt".to_string(),
            content_type: ContentType::Blog,
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 1000,
            temperature: 0.7,
            language: "ko".to_string(),
            audience: "개발자".to_string(),
            tone: "친근한".to_string(),
            context: None,
            stream: false,
            provider_params: HashMap::new(),
        };

        let system_message = provider.build_system_message(&request);
        assert!(system_message.contains("Test prompt"));
        assert!(system_message.contains("한국어"));
        assert!(system_message.contains("개발자"));
    }
}
