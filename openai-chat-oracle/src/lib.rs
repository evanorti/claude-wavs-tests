mod trigger;
use trigger::{decode_trigger_event, encode_trigger_output, Destination};
use wavs_wasi_utils::http::{fetch_json, http_request_post_json};
pub mod bindings;
use crate::bindings::{export, Guest, TriggerAction, WasmResponse};
use alloy_sol_types::{SolCall, SolValue};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;
use wstd::{http::HeaderValue, runtime::block_on};

struct Component;
export!(Component with_types_in bindings);

impl Guest for Component {
    fn run(action: TriggerAction) -> std::result::Result<Option<WasmResponse>, String> {
        let (trigger_id, req, dest) =
            decode_trigger_event(action.data).map_err(|e| e.to_string())?;

        // Clone request data to avoid ownership issues
        let req_clone = req.clone();

        // Decode the prompt string using proper ABI decoding
        let prompt =
            if let Ok(decoded) = trigger::solidity::generateResponseCall::abi_decode(&req_clone) {
                // Successfully decoded as function call
                decoded.prompt
            } else {
                // Try decoding just as a string parameter
                match <String as SolValue>::abi_decode(&req_clone) {
                    Ok(s) => s,
                    Err(e) => return Err(format!("Failed to decode input as ABI string: {}", e)),
                }
            };

        println!("Decoded prompt: {}", prompt);

        // Generate AI response
        let res = block_on(async move {
            let chat_data = generate_openai_response(&prompt).await?;
            serde_json::to_vec(&chat_data).map_err(|e| e.to_string())
        })?;

        let output = match dest {
            Destination::Ethereum => Some(encode_trigger_output(trigger_id, &res)),
            Destination::CliOutput => Some(WasmResponse { payload: res.into(), ordering: None }),
        };
        Ok(output)
    }
}

/// Sends a prompt to OpenAI's API and returns the AI response
async fn generate_openai_response(prompt: &str) -> Result<ChatResponse, String> {
    // Get API key from environment
    let api_key = env::var("WAVS_ENV_OPENAI_KEY")
        .map_err(|_| "Failed to get WAVS_ENV_OPENAI_KEY from environment variables".to_string())?;

    // Create request payload
    let request_data = OpenAIRequest {
        seed: 42,
        model: "gpt-4o".to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: "You are a helpful assistant.".to_string(),
            },
            Message { role: "user".to_string(), content: prompt.to_string() },
        ],
    };

    // Create HTTP request
    let url = "https://api.openai.com/v1/chat/completions";
    let mut req = http_request_post_json(url, &request_data)
        .map_err(|e| format!("Failed to create request: {}", e))?;

    // Add authentication and headers
    req.headers_mut().insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", api_key))
            .map_err(|e| format!("Invalid API key format: {}", e))?,
    );
    req.headers_mut().insert("Accept", HeaderValue::from_static("application/json"));
    req.headers_mut().insert("Content-Type", HeaderValue::from_static("application/json"));

    // Make API request
    let api_response: OpenAIResponse =
        fetch_json(req).await.map_err(|e| format!("Failed to fetch OpenAI response: {}", e))?;

    // Handle API errors
    if let Some(error) = api_response.error {
        return Err(format!(
            "OpenAI API error: {}",
            error.message.unwrap_or_else(|| "Unknown error".to_string())
        ));
    }

    // Extract response content
    let response_text = api_response
        .choices
        .unwrap_or_default()
        .first()
        .and_then(|choice| choice.message.as_ref())
        .and_then(|msg| msg.content.clone())
        .unwrap_or_else(|| "No response generated".to_string());

    // Get current timestamp
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string();

    Ok(ChatResponse {
        prompt: prompt.to_string(),
        response: response_text,
        model: "gpt-4o".to_string(),
        timestamp,
    })
}

/// OpenAI API request structure
#[derive(Debug, Serialize, Clone)]
struct OpenAIRequest {
    seed: u32,
    model: String,
    messages: Vec<Message>,
}

/// Message structure for OpenAI API
#[derive(Debug, Serialize, Clone)]
struct Message {
    role: String,
    content: String,
}

/// OpenAI API response structure
#[derive(Debug, Deserialize, Clone, Default)]
#[serde(default)]
struct OpenAIResponse {
    choices: Option<Vec<Choice>>,
    error: Option<ErrorInfo>,
}

/// Choice structure from OpenAI response
#[derive(Debug, Deserialize, Clone, Default)]
#[serde(default)]
struct Choice {
    message: Option<MessageContent>,
}

/// Message content from OpenAI response
#[derive(Debug, Deserialize, Clone, Default)]
#[serde(default)]
struct MessageContent {
    content: Option<String>,
}

/// Error information from OpenAI API
#[derive(Debug, Deserialize, Clone, Default)]
#[serde(default)]
struct ErrorInfo {
    message: Option<String>,
    #[serde(rename = "type")]
    error_type: Option<String>,
}

/// Final response structure returned by the component
#[derive(Debug, Serialize, Clone)]
struct ChatResponse {
    prompt: String,
    response: String,
    model: String,
    timestamp: String,
}
