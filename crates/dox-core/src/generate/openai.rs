//! OpenAI API integration for content generation

use super::{ContentGenerator, GenerationRequest, GenerationResponse, Usage};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

/// OpenAI API client for content generation
pub struct OpenAIProvider {
    client: Client,
    api_key: String,
    base_url: String,
}

impl OpenAIProvider {
    /// Create a new OpenAI provider
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
        }
    }

    /// Create OpenAI provider with custom base URL
    pub fn with_base_url(api_key: String, base_url: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url,
        }
    }

    /// Get API key from environment variable
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| anyhow!("OPENAI_API_KEY environment variable not found"))?;
        Ok(Self::new(api_key))
    }

    /// Convert internal request to OpenAI API format
    fn build_openai_request(&self, request: &GenerationRequest) -> OpenAIChatRequest {
        let system_message = self.build_system_message(request);
        let user_message = request.prompt.clone();

        OpenAIChatRequest {
            model: request.model.clone(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: system_message,
                },
                Message {
                    role: "user".to_string(),
                    content: user_message,
                },
            ],
            max_tokens: Some(request.max_tokens),
            temperature: Some(request.temperature),
            stream: Some(request.stream),
            n: Some(1),
            stop: None,
            presence_penalty: None,
            frequency_penalty: None,
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
        engine.render(&template).unwrap_or_else(|_| template)
    }

    /// Send non-streaming request to OpenAI
    async fn send_request(&self, openai_request: &OpenAIChatRequest) -> Result<OpenAIChatResponse> {
        debug!("Sending OpenAI API request: model={}, max_tokens={:?}", 
               openai_request.model, openai_request.max_tokens);

        let response = self
            .client
            .post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&openai_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("OpenAI API error: {} - {}", status, error_text);
            return Err(anyhow!("OpenAI API error {}: {}", status, error_text));
        }

        let openai_response: OpenAIChatResponse = response.json().await?;
        debug!("Received OpenAI response: {} choices, usage={:?}", 
               openai_response.choices.len(), openai_response.usage);

        Ok(openai_response)
    }

    /// Convert OpenAI response to internal format
    fn convert_response(&self, openai_response: OpenAIChatResponse, model: String) -> GenerationResponse {
        let content = openai_response
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .unwrap_or_default();

        let usage = openai_response.usage.map(|u| Usage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        });

        GenerationResponse {
            content,
            model,
            provider: "openai".to_string(),
            usage,
            metadata: HashMap::new(),
        }
    }
}

#[async_trait]
impl ContentGenerator for OpenAIProvider {
    async fn generate(&self, request: &GenerationRequest) -> Result<GenerationResponse> {
        info!("Generating content using OpenAI model: {}", request.model);

        // Validate model support
        if !self.supports_model(&request.model) {
            warn!("Model {} not explicitly supported, but attempting anyway", request.model);
        }

        let openai_request = self.build_openai_request(request);

        // For now, only support non-streaming requests
        // TODO: Implement streaming support
        if request.stream {
            warn!("Streaming not yet implemented, falling back to non-streaming");
        }

        let openai_response = self.send_request(&openai_request).await?;
        let response = self.convert_response(openai_response, request.model.clone());

        info!("Content generation completed. Generated {} characters, used {} tokens", 
              response.content.len(), 
              response.usage.as_ref().map(|u| u.total_tokens).unwrap_or(0));

        Ok(response)
    }

    fn provider_name(&self) -> &str {
        "openai"
    }

    fn supported_models(&self) -> Vec<String> {
        vec![
            "gpt-4".to_string(),
            "gpt-4-turbo".to_string(),
            "gpt-4o".to_string(),
            "gpt-4o-mini".to_string(),
            "gpt-3.5-turbo".to_string(),
            "gpt-3.5-turbo-16k".to_string(),
        ]
    }
}

/// OpenAI Chat Completions API request structure
#[derive(Debug, Serialize)]
struct OpenAIChatRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: Option<usize>,
    temperature: Option<f32>,
    stream: Option<bool>,
    n: Option<i32>,
    stop: Option<Vec<String>>,
    presence_penalty: Option<f32>,
    frequency_penalty: Option<f32>,
}

/// OpenAI message structure
#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

/// OpenAI Chat Completions API response structure
#[derive(Debug, Deserialize)]
struct OpenAIChatResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<Choice>,
    usage: Option<OpenAIUsage>,
}

/// OpenAI choice structure
#[derive(Debug, Deserialize)]
struct Choice {
    index: i32,
    message: Message,
    finish_reason: Option<String>,
}

/// OpenAI usage structure
#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate::ContentType;

    #[test]
    fn test_provider_creation() {
        let provider = OpenAIProvider::new("test-key".to_string());
        assert_eq!(provider.provider_name(), "openai");
        assert!(provider.supports_model("gpt-4"));
        assert!(provider.supports_model("gpt-3.5-turbo"));
        assert!(!provider.supports_model("invalid-model"));
    }

    #[test]
    fn test_system_message_building() {
        let provider = OpenAIProvider::new("test-key".to_string());
        let request = GenerationRequest {
            prompt: "Test prompt".to_string(),
            content_type: ContentType::Blog,
            model: "gpt-4".to_string(),
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