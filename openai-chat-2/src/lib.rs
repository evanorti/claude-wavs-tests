mod trigger;
use trigger::{decode_trigger_event, encode_trigger_output, Destination};
use wavs_wasi_utils::{
    evm::alloy_primitives::hex,
    http::{fetch_json, http_request_post_json},
};
pub mod bindings;
use crate::bindings::{export, Guest, TriggerAction, WasmResponse};
use alloy_sol_types::SolValue;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use wstd::{http::HeaderValue, runtime::block_on};

// API Response Structures
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct OpenAiResponse {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    choices: Option<Vec<Choice>>,
    #[serde(default)]
    created: Option<u64>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    usage: Option<Usage>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Choice {
    #[serde(default)]
    message: Option<Message>,
    #[serde(default)]
    finish_reason: Option<String>,
    #[serde(default)]
    index: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Message {
    #[serde(default)]
    role: Option<String>,
    #[serde(default)]
    content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Usage {
    #[serde(default)]
    prompt_tokens: Option<u64>,
    #[serde(default)]
    completion_tokens: Option<u64>,
    #[serde(default)]
    total_tokens: Option<u64>,
}

// Request Structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenAiRequest {
    model: String,
    messages: Vec<Message>,
}

// Component Implementation
struct Component;
export!(Component with_types_in bindings);

impl Guest for Component {
    fn run(action: TriggerAction) -> std::result::Result<Option<WasmResponse>, String> {
        let (trigger_id, req, dest) =
            decode_trigger_event(action.data).map_err(|e| e.to_string())?;

        // Decode trigger data inline - handles hex string input
        let prompt = {
            // First, convert the input bytes to a string to check if it's a hex string
            let input_str = String::from_utf8(req.clone())
                .map_err(|e| format!("Input is not valid UTF-8: {}", e))?;

            // Check if it's a hex string (starts with "0x")
            let hex_data = if input_str.starts_with("0x") {
                // Decode the hex string to bytes
                hex::decode(&input_str[2..])
                    .map_err(|e| format!("Failed to decode hex string: {}", e))?
            } else {
                // If it's not a hex string, assume the input is already binary data
                req.clone()
            };

            // Now ABI decode the binary data as a string parameter
            <String as SolValue>::abi_decode(&hex_data)
                .map_err(|e| format!("Failed to decode input as ABI string: {}", e))?
        };

        // Process the prompt with OpenAI
        let res = block_on(async move {
            let response = call_openai(&prompt).await?;
            serde_json::to_vec(&response).map_err(|e| e.to_string())
        })?;

        // Return result based on destination
        let output = match dest {
            Destination::Ethereum => Some(encode_trigger_output(trigger_id, &res)),
            Destination::CliOutput => Some(WasmResponse { payload: res.into(), ordering: None }),
        };
        Ok(output)
    }
}

async fn call_openai(prompt: &str) -> Result<OpenAiResponse, String> {
    // Get API key from environment
    let api_key = std::env::var("WAVS_ENV_OPENAI_KEY")
        .map_err(|_| "Failed to get OPENAI_KEY from environment variables".to_string())?;

    // Create request
    let request = OpenAiRequest {
        model: "gpt-4".to_string(),
        messages: vec![
            Message {
                role: Some("system".to_string()),
                content: Some("You are a helpful assistant.".to_string()),
            },
            Message { role: Some("user".to_string()), content: Some(prompt.to_string()) },
        ],
    };

    // Create HTTP request
    let url = "https://api.openai.com/v1/chat/completions";
    let mut req = http_request_post_json(url, &request)
        .map_err(|e| format!("Failed to create request: {}", e))?;

    // Add headers
    req.headers_mut().insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", api_key))
            .map_err(|e| format!("Failed to create Authorization header: {}", e))?,
    );
    req.headers_mut().insert("Content-Type", HeaderValue::from_static("application/json"));

    // Make request
    let response: OpenAiResponse =
        fetch_json(req).await.map_err(|e| format!("Failed to fetch data: {}", e))?;

    Ok(response)
}
