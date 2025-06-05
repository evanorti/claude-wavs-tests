mod trigger;
use trigger::{decode_trigger_event, encode_trigger_output, Destination};
use wavs_wasi_utils::http::{fetch_json, http_request_post_json};
pub mod bindings;
use crate::bindings::{export, Guest, TriggerAction, WasmResponse};
use alloy_sol_types::{SolCall, SolValue};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use wstd::{http::HeaderValue, runtime::block_on};

// API request and response structures
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct ChatResponse {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    object: Option<String>,
    #[serde(default)]
    created: Option<u64>,
    #[serde(default)]
    choices: Vec<Choice>,
    #[serde(default)]
    usage: Option<Usage>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct Choice {
    #[serde(default)]
    index: Option<u64>,
    #[serde(default)]
    message: Option<Message>,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct Usage {
    #[serde(default)]
    prompt_tokens: Option<u64>,
    #[serde(default)]
    completion_tokens: Option<u64>,
    #[serde(default)]
    total_tokens: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ResultData {
    prompt: String,
    response: String,
    timestamp: String,
}

struct Component;
export!(Component with_types_in bindings);

impl Guest for Component {
    fn run(action: TriggerAction) -> std::result::Result<Option<WasmResponse>, String> {
        let (trigger_id, req, dest) =
            decode_trigger_event(action.data).map_err(|e| e.to_string())?;

        // Clone request data to avoid ownership issues
        let req_clone = req.clone();

        // Decode the prompt string using proper ABI decoding
        let prompt = if let Ok(decoded) = trigger::solidity::sendPromptCall::abi_decode(&req_clone)
        {
            // If it has a function selector (from cast abi-encode "f(string)" format)
            decoded.prompt
        } else {
            // Fallback: try decoding just as a string parameter (no function selector)
            match <String as SolValue>::abi_decode(&req_clone) {
                Ok(s) => s,
                Err(e) => return Err(format!("Failed to decode input as ABI string: {}", e)),
            }
        };

        println!("Decoded prompt: {}", prompt);

        // Process the prompt with OpenAI
        let result = block_on(async move {
            let response = send_to_openai(&prompt).await?;
            serde_json::to_vec(&response).map_err(|e| e.to_string())
        })?;

        let output = match dest {
            Destination::Ethereum => Some(encode_trigger_output(trigger_id, &result)),
            Destination::CliOutput => Some(WasmResponse { payload: result.into(), ordering: None }),
        };
        Ok(output)
    }
}

async fn send_to_openai(prompt: &str) -> Result<ResultData, String> {
    // Get API key from environment
    let api_key = std::env::var("WAVS_ENV_OPENAI_KEY")
        .map_err(|_| "Failed to get OPENAI_KEY from environment variables".to_string())?;

    // Create request
    let request = ChatRequest {
        model: "gpt-4".to_string(),
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
    let mut req = http_request_post_json(url, &request)
        .map_err(|e| format!("Failed to create request: {}", e))?;

    // Add headers
    req.headers_mut().insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", api_key))
            .map_err(|e| format!("Failed to create Authorization header: {}", e))?,
    );
    req.headers_mut().insert("Content-Type", HeaderValue::from_static("application/json"));

    // Make API request
    let response: ChatResponse =
        fetch_json(req).await.map_err(|e| format!("Failed to fetch data: {}", e))?;

    // Extract response text
    let response_text = response
        .choices
        .first()
        .and_then(|choice| choice.message.as_ref())
        .map(|msg| msg.content.clone())
        .unwrap_or_else(|| "No response received".to_string());

    // Return result
    Ok(ResultData {
        prompt: prompt.to_string(),
        response: response_text,
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}
