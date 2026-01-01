//! MCP Tools definitions
//!
//! Available tools for the MCP server

use serde_json::json;

use crate::{CallToolResult, Tool};
use neuro_inference::{
    InferenceModel, InferenceConfig, GenerateOptions, SamplerConfig,
    translation::{build_translation_prompt, detect_language, Language},
};

/// Get all available tools
pub fn get_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "generate".to_string(),
            description: "Generate text using BitNet model. Supports queries in English and Spanish.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "prompt": {
                        "type": "string",
                        "description": "The prompt to generate text from. Can be in English or Spanish."
                    },
                    "max_tokens": {
                        "type": "integer",
                        "description": "Maximum number of tokens to generate (default: 512)",
                        "default": 512
                    },
                    "temperature": {
                        "type": "number",
                        "description": "Temperature for generation (default: 0.7)",
                        "default": 0.7
                    }
                },
                "required": ["prompt"]
            }),
        },
        Tool {
            name: "translate".to_string(),
            description: "Translate text to English using BitNet model.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "The text to translate to English"
                    }
                },
                "required": ["text"]
            }),
        },
        Tool {
            name: "ask".to_string(),
            description: "Ask a question and get an answer. Automatically handles Spanish queries.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "question": {
                        "type": "string",
                        "description": "The question to ask. Can be in English or Spanish."
                    },
                    "context": {
                        "type": "string",
                        "description": "Optional context to help answer the question"
                    }
                },
                "required": ["question"]
            }),
        },
        Tool {
            name: "summarize".to_string(),
            description: "Summarize a text using BitNet model.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "The text to summarize"
                    },
                    "max_length": {
                        "type": "integer",
                        "description": "Maximum length of the summary in words",
                        "default": 100
                    }
                },
                "required": ["text"]
            }),
        },
    ]
}

/// Execute a tool
pub async fn execute_tool(
    name: &str,
    arguments: serde_json::Value,
    model_path: &str,
) -> CallToolResult {
    match name {
        "generate" => execute_generate(arguments, model_path).await,
        "translate" => execute_translate(arguments, model_path).await,
        "ask" => execute_ask(arguments, model_path).await,
        "summarize" => execute_summarize(arguments, model_path).await,
        _ => CallToolResult::error(format!("Unknown tool: {}", name)),
    }
}

async fn execute_generate(args: serde_json::Value, model_path: &str) -> CallToolResult {
    let prompt = match args.get("prompt").and_then(|v| v.as_str()) {
        Some(p) => p,
        None => return CallToolResult::error("Missing required parameter: prompt".to_string()),
    };

    let max_tokens = args
        .get("max_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(512) as u32;

    let temperature = args
        .get("temperature")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.7) as f32;

    // Detect language and translate if needed
    let lang = detect_language(prompt);
    let english_prompt = if matches!(lang, Language::Spanish) {
        let translation_prompt = build_translation_prompt(prompt);
        match run_model(model_path, &translation_prompt, 256, temperature).await {
            Ok(translated) => translated.trim().to_string(),
            Err(e) => return CallToolResult::error(format!("Translation failed: {}", e)),
        }
    } else {
        prompt.to_string()
    };

    match run_model(model_path, &english_prompt, max_tokens, temperature).await {
        Ok(result) => CallToolResult::text(result),
        Err(e) => CallToolResult::error(format!("Generation failed: {}", e)),
    }
}

async fn execute_translate(args: serde_json::Value, model_path: &str) -> CallToolResult {
    let text = match args.get("text").and_then(|v| v.as_str()) {
        Some(t) => t,
        None => return CallToolResult::error("Missing required parameter: text".to_string()),
    };

    let prompt = build_translation_prompt(text);
    match run_model(model_path, &prompt, 256, 0.3).await {
        Ok(result) => CallToolResult::text(result),
        Err(e) => CallToolResult::error(format!("Translation failed: {}", e)),
    }
}

async fn execute_ask(args: serde_json::Value, model_path: &str) -> CallToolResult {
    let question = match args.get("question").and_then(|v| v.as_str()) {
        Some(q) => q,
        None => return CallToolResult::error("Missing required parameter: question".to_string()),
    };

    let context = args.get("context").and_then(|v| v.as_str());

    // Detect language and translate if needed
    let lang = detect_language(question);
    let english_question = if matches!(lang, Language::Spanish) {
        let translation_prompt = build_translation_prompt(question);
        match run_model(model_path, &translation_prompt, 256, 0.3).await {
            Ok(translated) => translated.trim().to_string(),
            Err(e) => return CallToolResult::error(format!("Translation failed: {}", e)),
        }
    } else {
        question.to_string()
    };

    // Build prompt with context if provided
    let prompt = if let Some(ctx) = context {
        format!(
            "Context: {}\n\nQuestion: {}\n\nAnswer:",
            ctx, english_question
        )
    } else {
        format!("Question: {}\n\nAnswer:", english_question)
    };

    match run_model(model_path, &prompt, 512, 0.7).await {
        Ok(result) => CallToolResult::text(result),
        Err(e) => CallToolResult::error(format!("Failed to answer: {}", e)),
    }
}

async fn execute_summarize(args: serde_json::Value, model_path: &str) -> CallToolResult {
    let text = match args.get("text").and_then(|v| v.as_str()) {
        Some(t) => t,
        None => return CallToolResult::error("Missing required parameter: text".to_string()),
    };

    let max_length = args
        .get("max_length")
        .and_then(|v| v.as_u64())
        .unwrap_or(100);

    let prompt = format!(
        "Summarize the following text in {} words or less:\n\n{}\n\nSummary:",
        max_length, text
    );

    match run_model(model_path, &prompt, 256, 0.5).await {
        Ok(result) => CallToolResult::text(result),
        Err(e) => CallToolResult::error(format!("Summarization failed: {}", e)),
    }
}

/// Run the BitNet model
async fn run_model(
    model_path: &str,
    prompt: &str,
    max_tokens: u32,
    temperature: f32,
) -> anyhow::Result<String> {
    let options = GenerateOptions::new(max_tokens)
        .with_sampler(SamplerConfig::default().with_temperature(temperature));
    
    let model_path = model_path.to_string();
    let prompt = prompt.to_string();
    
    let result = tokio::task::spawn_blocking(move || {
        let model = InferenceModel::load(InferenceConfig::new(&model_path))?;
        model.generate(&prompt, &options)
    })
    .await??;

    Ok(result)
}
