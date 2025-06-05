mod trigger;
use trigger::{decode_trigger_event, encode_trigger_output, Destination};
use wavs_wasi_utils::http::{fetch_json, http_request_post_json};
pub mod bindings;
use crate::bindings::{export, Guest, TriggerAction, WasmResponse};
use alloy_sol_types::{SolCall, SolValue};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use wstd::{http::HeaderValue, runtime::block_on};

struct Component;
export!(Component with_types_in bindings);

impl Guest for Component {
    fn run(action: TriggerAction) -> std::result::Result<Option<WasmResponse>, String> {
        let (trigger_id, req, dest) =
            decode_trigger_event(action.data).map_err(|e| e.to_string())?;

        // Clone request data to avoid ownership issues
        let req_clone = req.clone();

        // Decode the string using proper ABI decoding
        let prompt_text =
            if let Ok(decoded) = trigger::solidity::processPromptCall::abi_decode(&req_clone) {
                // If it has a function selector (from cast abi-encode "f(string)" format)
                decoded.prompt
            } else {
                // Fallback: try decoding just as a string parameter (no function selector)
                match <String as SolValue>::abi_decode(&req_clone) {
                    Ok(s) => s,
                    Err(e) => return Err(format!("Failed to decode input as ABI string: {}", e)),
                }
            };

        println!("Decoded prompt input: {}", prompt_text);

        // Process the prompt with OpenAI
        let result = block_on(async move {
            let response_data = process_openai_prompt(&prompt_text).await?;
            serde_json::to_vec(&response_data).map_err(|e| e.to_string())
        })?;

        let output = match dest {
            Destination::Ethereum => Some(encode_trigger_output(trigger_id, &result)),
            Destination::CliOutput => Some(WasmResponse { payload: result.into(), ordering: None }),
        };
        Ok(output)
    }
}

#[derive(Debug, Serialize, Clone)]
struct OpenAIRequest {
    seed: u32,
    model: String,
    messages: Vec<Message>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
struct OpenAIResponse {
    id: Option<String>,
    choices: Option<Vec<Choice>>,
    usage: Option<Usage>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
struct Choice {
    message: Option<Message>,
    finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
struct Usage {
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
    total_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PromptResponse {
    prompt: String,
    response: String,
    timestamp: String,
    tokens_used: u32,
}

async fn process_openai_prompt(prompt: &str) -> Result<PromptResponse, String> {
    // Get API key from environment
    let api_key = std::env::var("WAVS_ENV_OPENAI_KEY")
        .map_err(|_| "Failed to get OPENAI_KEY from environment variables".to_string())?;

    // Create OpenAI request
    let request = OpenAIRequest {
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

    // Create HTTP POST request with JSON data
    let mut req = http_request_post_json("https://api.openai.com/v1/chat/completions", &request)
        .map_err(|e| format!("Failed to create request: {}", e))?;

    // Add authorization header
    req.headers_mut().insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", api_key))
            .map_err(|e| format!("Invalid API key format: {}", e))?,
    );

    // Make API request
    let api_response: OpenAIResponse =
        fetch_json(req).await.map_err(|e| format!("Failed to fetch OpenAI response: {}", e))?;

    // Extract response text
    let response_text = api_response
        .choices
        .as_ref()
        .and_then(|choices| choices.first())
        .and_then(|choice| choice.message.as_ref())
        .map(|msg| msg.content.clone())
        .unwrap_or_else(|| "No response generated".to_string());

    // Get token usage
    let tokens_used = api_response.usage.as_ref().and_then(|usage| usage.total_tokens).unwrap_or(0);

    // Get current timestamp
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string();

    Ok(PromptResponse {
        prompt: prompt.to_string(),
        response: response_text,
        timestamp,
        tokens_used,
    })
}
