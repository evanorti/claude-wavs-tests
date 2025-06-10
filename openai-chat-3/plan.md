# OpenAI Chat Component Plan

## Overview
This WAVS component takes a text prompt as input, sends it to OpenAI's chat completions API, and returns the AI-generated response.

## Component Flow
```
Input: String (prompt) 
  ↓
Decode ABI input to extract prompt
  ↓  
Send POST request to OpenAI API with prompt
  ↓
Parse JSON response and extract content
  ↓
Return formatted response based on destination
```

## API Integration
- **Endpoint**: `https://api.openai.com/v1/chat/completions`
- **Method**: POST
- **Authentication**: Bearer token from `WAVS_ENV_OPENAI_KEY`
- **Request Structure**:
  ```json
  {
    "model": "gpt-4o",
    "messages": [
      {"role": "system", "content": "You are a helpful assistant."},
      {"role": "user", "content": "<PROMPT>"}
    ]
  }
  ```

## Response Structure
OpenAI API returns JSON with these key fields:
- `choices[0].message.content` - The AI response content
- `id` - Unique response ID
- `model` - Model used
- `usage.total_tokens` - Token count

## Required Imports
```rust
// Core WAVS dependencies
use alloy_sol_types::SolValue;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use wavs_wasi_utils::{
    evm::alloy_primitives::hex,
    http::{fetch_json, http_request_post_json},
};
use wstd::{http::HeaderValue, runtime::block_on};

// Component bindings
pub mod bindings;
mod trigger;
use trigger::{decode_trigger_event, encode_trigger_output, Destination};
use crate::bindings::{export, Guest, TriggerAction, WasmResponse};
```

## Data Structures

### Request Structure
```rust
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
```

### Response Structure  
```rust
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
```

### Result Structure
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatResult {
    prompt: String,
    response: String,
    model: String,
    tokens_used: u32,
}
```

## Validation Checklist

### Common Errors
- [x] Use `{ workspace = true }` in component Cargo.toml
- [x] Verify API response structures (done with curl test)
- [x] Read documentation and requirements from claude.md
- [x] Implement Guest trait and export component correctly  
- [x] Use `export!(Component with_types_in bindings)`
- [x] Use `clone()` before consuming data
- [x] Derive `Clone` for API response data structures
- [x] Decode ABI data properly with hex string handling
- [x] Use `ok_or_else()` for Option types, `map_err()` for Result types
- [x] Use string parameters for CLI testing
- [x] Use `.to_string()` for string literal to String conversions
- [x] Never edit bindings.rs

### Component Structure
- [x] Implements Guest trait
- [x] Exports component correctly with `export!(Component with_types_in bindings)`
- [x] Properly handles TriggerAction and TriggerData

### ABI Handling
- [x] Properly decodes function calls with hex string support
- [x] Avoids String::from_utf8 on ABI data
- [x] Uses `<String as SolValue>::abi_decode(&hex_data)`

### Data Ownership
- [x] All API structures derive Clone
- [x] Clone data before use to avoid ownership issues
- [x] Avoid moving out of collections

### Error Handling
- [x] Use ok_or_else() for Option types
- [x] Use map_err() for Result types
- [x] Provide descriptive error messages

### Imports
- [x] Include all required traits and types
- [x] Use correct import paths
- [x] Import SolValue for ABI decoding
- [x] Import http functions from wavs_wasi_utils
- [x] Import block_on from wstd::runtime
- [x] All dependencies in Cargo.toml with `{workspace = true}`

### Security
- [x] No hardcoded API keys
- [x] Use environment variable `WAVS_ENV_OPENAI_KEY`

### Network Requests
- [x] Use block_on for async functions
- [x] Use http_request_post_json for POST requests
- [x] Use proper headers including Authorization bearer token
- [x] Use #[serde(default)] and Option<T> for external API fields

## Implementation Details

### Input Processing
- Receive TriggerAction with ABI-encoded string
- Handle both hex strings ("0x...") and binary data
- Decode using `<String as SolValue>::abi_decode(&hex_data)`

### API Request
- Create OpenAI request with system and user messages
- Add Authorization header with bearer token from environment
- Use `http_request_post_json` for POST request
- Handle API response with proper error checking

### Output Processing  
- Extract response content from choices[0].message.content
- Create ChatResult with prompt, response, model, and token usage
- Serialize to JSON and return based on destination (Ethereum vs CLI)

## Component Ready for Implementation
All planning steps completed, validation checklist verified, API structure confirmed. Ready to proceed with implementation.