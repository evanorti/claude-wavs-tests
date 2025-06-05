# OpenAI Prompt Processor Component Plan

## Prompt

```
Please make a component that takes a prompt as input, sends an api request to OpenAI, and returns the response.

  Use this api structure:
  {
    "seed": $SEED,
    "model": "gpt-4o",
    "messages": [
      {"role": "system", "content": "You are a helpful assistant."},
      {"role": "user", "content": "<PROMPT>"}
    ]
  }

  My api key is WAVS_ENV_OPENAI_KEY in my .env file.
```

## Overview
A WAVS component that takes a text prompt as input, sends it to OpenAI's API, and returns the AI response.

## Component Structure
✅ **Component Name**: openai-prompt-processor
✅ **Input Format**: String (user prompt)
✅ **Output Format**: JSON with prompt, response, timestamp
✅ **API**: OpenAI Chat Completions API with gpt-4o model

## Validation Checklist
✅ 1. Use `{ workspace = true }` in Cargo.toml - planned
✅ 2. Verify API response structure by using curl - completed above
✅ 3. Read documentation - completed
✅ 4. Implement Guest trait and export component - planned
✅ 5. Use `export!(Component with_types_in bindings)` - planned
✅ 6. Use `clone()` before consuming data - planned
✅ 7. Derive `Clone` for API response structures - planned
✅ 8. Decode ABI data properly with abi_decode - planned
✅ 9. Use `ok_or_else()` for Option types, `map_err()` for Result types - planned
✅ 10. Use string parameters for CLI testing - planned
✅ 11. Use `.to_string()` for string literals in struct fields - planned
✅ 12. Never edit bindings.rs - understood

## Required Imports
- `alloy_sol_types::{sol, SolCall, SolValue}` - for ABI decoding
- `serde::{Deserialize, Serialize}` - for JSON handling
- `wavs_wasi_utils::http::{fetch_json, http_request_post}` - for HTTP requests
- `wstd::{http::HeaderValue, runtime::block_on}` - for async handling
- `anyhow::Result` - for error handling

## Data Structures

### API Request Structure
```rust
#[derive(Debug, Serialize, Clone)]
struct OpenAIRequest {
    seed: u32,
    model: String,
    messages: Vec<Message>,
}

#[derive(Debug, Serialize, Clone)]
struct Message {
    role: String,
    content: String,
}
```

### API Response Structure
```rust
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
```

### Component Output Structure
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
struct PromptResponse {
    prompt: String,
    response: String,
    timestamp: String,
    tokens_used: u32,
}
```

## Solidity Function Definition
```rust
sol! {
    function processPrompt(string prompt) external;
}
```

## Component Flow
1. Receive ABI-encoded prompt string
2. Decode using proper ABI decoding (abi_decode)
3. Create OpenAI API request with fixed seed and system message
4. Send POST request to OpenAI API with authentication header
5. Parse response and extract the AI's message
6. Return formatted response data

## Security Considerations
✅ API key stored in environment variable `WAVS_ENV_OPENAI_KEY`
✅ No hardcoded secrets
✅ Proper error handling for API failures

## Dependencies Required in Cargo.toml
```toml
# Core dependencies (always needed)
wit-bindgen-rt = {workspace = true}
wavs-wasi-utils = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
alloy-sol-macro = { workspace = true }
wstd = { workspace = true }
alloy-sol-types = { workspace = true }
anyhow = { workspace = true }
```

## Error Handling Strategy
- Use `map_err()` for Result types from HTTP requests
- Use `ok_or_else()` for Option types from environment variables
- Handle missing fields in API response with Option<T> and #[serde(default)]
- Provide descriptive error messages

This component will be robust, secure, and follow all WAVS component best practices.
