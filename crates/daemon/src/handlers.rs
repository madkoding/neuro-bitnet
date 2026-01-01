//! HTTP handlers for the daemon API

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use neuro_inference::{GenerateOptions, SamplerConfig};
use neuro_inference::translation::{detect_language, build_translation_prompt, Language};

use crate::AppState;

/// Request for text generation
#[derive(Debug, Deserialize)]
pub struct GenerateRequest {
    /// The prompt or question
    pub prompt: String,
    /// Maximum tokens to generate (optional)
    pub max_tokens: Option<u32>,
    /// Temperature (optional)
    pub temperature: Option<f32>,
    /// Whether to translate non-English queries (optional, uses server default)
    pub translate: Option<bool>,
}

/// Response from text generation
#[derive(Debug, Serialize)]
pub struct GenerateResponse {
    /// The generated text
    pub response: String,
    /// Original prompt
    pub prompt: String,
    /// Whether the prompt was translated
    pub was_translated: bool,
    /// The translated prompt (if applicable)
    pub translated_prompt: Option<String>,
    /// Detected language
    pub detected_language: String,
    /// Time taken in milliseconds
    pub time_ms: u64,
}

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub model_loaded: bool,
    pub version: String,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Health check endpoint
pub async fn health(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let model_loaded = state.is_model_loaded().await;
    
    Json(HealthResponse {
        status: if model_loaded { "healthy".to_string() } else { "loading".to_string() },
        model_loaded,
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Generate text endpoint
pub async fn generate(
    State(state): State<Arc<AppState>>,
    Json(request): Json<GenerateRequest>,
) -> Result<Json<GenerateResponse>, (StatusCode, Json<ErrorResponse>)> {
    let start = std::time::Instant::now();
    
    // Get model
    let model_guard = state.model.read().await;
    let model = model_guard.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: "Model not loaded yet".to_string(),
            }),
        )
    })?;

    // Detect language
    let detected_lang = detect_language(&request.prompt);
    let should_translate = request.translate.unwrap_or(state.auto_translate) 
        && !matches!(detected_lang, Language::English);

    // Translate if needed
    let (effective_prompt, was_translated, translated_prompt) = if should_translate {
        let translate_prompt = build_translation_prompt(&request.prompt);
        
        let translate_options = GenerateOptions::new(100)
            .with_sampler(SamplerConfig::default().with_temperature(0.1));
        
        let translation = model.generate(&translate_prompt, &translate_options)
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!("Translation failed: {}", e),
                    }),
                )
            })?;
        
        let english = translation.trim().to_string();
        (english.clone(), true, Some(english))
    } else {
        (request.prompt.clone(), false, None)
    };

    // Generate response
    let max_tokens = request.max_tokens.unwrap_or(state.max_tokens);
    let temperature = request.temperature.unwrap_or(state.temperature);
    
    let prompt = format!("Q: {}\nA:", effective_prompt);
    
    let gen_options = GenerateOptions::new(max_tokens)
        .with_sampler(SamplerConfig::default().with_temperature(temperature));
    
    let response = model.generate(&prompt, &gen_options)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Generation failed: {}", e),
                }),
            )
        })?;

    let time_ms = start.elapsed().as_millis() as u64;

    Ok(Json(GenerateResponse {
        response: response.trim().to_string(),
        prompt: request.prompt,
        was_translated,
        translated_prompt,
        detected_language: format!("{:?}", detected_lang),
        time_ms,
    }))
}

/// Chat endpoint (for compatibility)
pub async fn chat(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, (StatusCode, Json<ErrorResponse>)> {
    let gen_request = GenerateRequest {
        prompt: request.messages.last()
            .map(|m| m.content.clone())
            .unwrap_or_default(),
        max_tokens: request.max_tokens,
        temperature: request.temperature,
        translate: Some(true),
    };

    let result = generate(State(state), Json(gen_request)).await?;
    
    Ok(Json(ChatResponse {
        id: format!("chatcmpl-{}", uuid_simple()),
        object: "chat.completion".to_string(),
        created: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        model: "bitnet-2b".to_string(),
        choices: vec![ChatChoice {
            index: 0,
            message: ChatMessage {
                role: "assistant".to_string(),
                content: result.response.clone(),
            },
            finish_reason: "stop".to_string(),
        }],
        usage: ChatUsage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        },
    }))
}

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChatChoice>,
    pub usage: ChatUsage,
}

#[derive(Debug, Serialize)]
pub struct ChatChoice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: String,
}

#[derive(Debug, Serialize)]
pub struct ChatUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", nanos)
}
