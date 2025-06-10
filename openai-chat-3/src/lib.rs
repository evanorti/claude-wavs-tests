mod trigger;
use trigger::{decode_trigger_event, encode_trigger_output, Destination};
use wavs_wasi_utils::{
    evm::alloy_primitives::hex,
    http::{fetch_json, http_request_post_json},
};
pub mod bindings;
use crate::bindings::{export, Guest, TriggerAction, WasmResponse};
use alloy_sol_types::SolValue;
use serde::{Deserialize, Serialize};
use wstd::{http::HeaderValue, runtime::block_on};

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
        println!("Decoded prompt: {}", prompt);

        // Send prompt to OpenAI and get response
        let result = block_on(async move {
            let chat_result = send_to_openai(&prompt).await?;
            serde_json::to_vec(&chat_result).map_err(|e| e.to_string())
        })?;

        let output = match dest {
            Destination::Ethereum => Some(encode_trigger_output(trigger_id, &result)),
            Destination::CliOutput => Some(WasmResponse { payload: result.into(), ordering: None }),
        };
        Ok(output)
    }
}

async fn send_to_openai(prompt: &str) -> Result<ChatResult, String> {
    // Get API key from environment
    let api_key = std::env::var("WAVS_ENV_OPENAI_KEY")
        .map_err(|_| "Failed to get OPENAI_KEY from environment variables".to_string())?;

    // Create OpenAI request
    let openai_request = OpenAIRequest {
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
    let mut req = http_request_post_json(url, &openai_request)
        .map_err(|e| format!("Failed to create request: {}", e))?;

    // Add authorization header
    let auth_header = format!("Bearer {}", api_key);
    req.headers_mut().insert(
        "Authorization",
        HeaderValue::from_str(&auth_header)
            .map_err(|e| format!("Failed to set authorization header: {}", e))?,
    );
    req.headers_mut().insert("Content-Type", HeaderValue::from_static("application/json"));
    req.headers_mut().insert("User-Agent", HeaderValue::from_static("WAVS-OpenAI-Component/1.0"));

    // Send request and parse response
    let openai_response: OpenAIResponse =
        fetch_json(req).await.map_err(|e| format!("Failed to send request to OpenAI: {}", e))?;

    // Extract response content
    let response_content = openai_response
        .choices
        .as_ref()
        .and_then(|choices| choices.first())
        .and_then(|choice| choice.message.as_ref())
        .and_then(|message| message.content.as_ref())
        .ok_or_else(|| "No response content found in OpenAI response".to_string())?;

    let model = openai_response.model.unwrap_or_else(|| "gpt-4o".to_string());
    let tokens_used =
        openai_response.usage.as_ref().and_then(|usage| usage.total_tokens).unwrap_or(0);

    Ok(ChatResult {
        prompt: prompt.to_string(),
        response: response_content.clone(),
        model,
        tokens_used,
    })
}

#[derive(Debug, Serialize, Clone)]
pub struct OpenAIRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct OpenAIResponse {
    id: Option<String>,
    object: Option<String>,
    created: Option<u64>,
    model: Option<String>,
    choices: Option<Vec<Choice>>,
    usage: Option<Usage>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Choice {
    index: Option<u32>,
    message: Option<ResponseMessage>,
    finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct ResponseMessage {
    role: Option<String>,
    content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Usage {
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
    total_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatResult {
    prompt: String,
    response: String,
    model: String,
    tokens_used: u32,
}
