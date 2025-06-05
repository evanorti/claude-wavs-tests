# OpenAI Chat Oracle Component Plan

## Overview
This WAVS component takes a text prompt as input, sends it to OpenAI's Chat API, and returns the AI response. It will use the GPT-4o model and include a system message for consistent behavior.

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

## API Structure Verified
- ✅ Tested OpenAI endpoint: `https://api.openai.com/v1/chat/completions`
- ✅ Confirmed request structure with seed, model, and messages array
- ✅ Error response structure confirmed (JSON with error object)
- ✅ API key authentication required via Bearer token

## Component Architecture

### Input
- Function: `generateResponse(string prompt)`
- Takes a string prompt via ABI-encoded input
- Will use proper ABI decoding to extract the prompt

### Processing Flow
1. Decode ABI input to extract prompt string
2. Create request payload with:
   - `seed`: Fixed value for reproducibility
   - `model`: "gpt-4o"  
   - `messages`: System + user prompt
3. Send POST request to OpenAI API
4. Parse response and extract generated text
5. Return formatted response

### Output
- JSON structure containing:
  - `prompt`: Original user prompt
  - `response`: AI-generated response
  - `model`: Model used
  - `timestamp`: Request timestamp

## Validation Checklist

### ABI Encoding Checks
- [x] ✅ ALWAYS use proper ABI decoding (functionCall::abi_decode)
- [x] ✅ NEVER use String::from_utf8 on ABI data
- [x] ✅ Define Solidity function signature for input
- [x] ✅ Handle both function call and direct string parameter formats

### Data Handling Checks  
- [x] ✅ Derive Clone for all API response structures
- [x] ✅ Use #[serde(default)] and Option<T> for API fields
- [x] ✅ Clone data before use to avoid ownership issues
- [x] ✅ Avoid &data.clone() pattern

### Error Handling Checks
- [x] ✅ Use ok_or_else() for Option types
- [x] ✅ Use map_err() for Result types
- [x] ✅ Provide descriptive error messages

### Import Checks
- [x] ✅ Import all required traits and types
- [x] ✅ Use std::str::FromStr for parsing
- [x] ✅ Import wstd::runtime::block_on for async
- [x] ✅ Import wavs_wasi_utils::http functions
- [x] ✅ Import alloy_sol_types::{SolCall, SolValue}

### Component Structure Checks
- [x] ✅ Use export!(Component with_types_in bindings)
- [x] ✅ Implement Guest trait properly
- [x] ✅ Return Result<Option<WasmResponse>, String>
- [x] ✅ Handle TriggerAction and TriggerData correctly

### Security Checks
- [x] ✅ Use environment variable WAVS_ENV_OPENAI_KEY
- [x] ✅ No hardcoded API keys or secrets

### Dependencies
- [x] ✅ Use { workspace = true } for all dependencies
- [x] ✅ Include serde, serde_json for JSON handling
- [x] ✅ Include wavs_wasi_utils for HTTP requests
- [x] ✅ Include wstd for async runtime

### Solidity Types
- [x] ✅ Define proper sol! function for input
- [x] ✅ Use solidity module structure
- [x] ✅ Use .to_string() for string literals in structs

### Network Requests
- [x] ✅ Use block_on for async functions
- [x] ✅ Use http_request_post_json for POST requests
- [x] ✅ Set proper headers (Authorization, Content-Type)

## Required Imports
```rust
use alloy_sol_types::{SolCall, SolValue};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;
use wavs_wasi_utils::http::{fetch_json, http_request_post_json};
use wstd::{http::HeaderValue, runtime::block_on};
```

## Data Structures

### API Request
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

### API Response
```rust
#[derive(Debug, Deserialize, Clone, Default)]
#[serde(default)]
struct OpenAIResponse {
    choices: Option<Vec<Choice>>,
    error: Option<ErrorInfo>,
}

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(default)]
struct Choice {
    message: Option<MessageContent>,
}

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(default)]
struct MessageContent {
    content: Option<String>,
}
```

### Component Output
```rust
#[derive(Debug, Serialize, Clone)]
struct ChatResponse {
    prompt: String,
    response: String,
    model: String,
    timestamp: String,
}
```

## Flow Chart
```
Input (ABI-encoded string) 
    ↓
Decode prompt using abi_decode
    ↓
Create OpenAI request payload
    ↓
Send POST to api.openai.com/v1/chat/completions
    ↓
Parse JSON response
    ↓
Extract generated text from choices[0].message.content
    ↓
Format output as ChatResponse
    ↓
Serialize to JSON and return
```

## Constants
- Model: "gpt-4o"
- Seed: 42 (for reproducibility)
- System message: "You are a helpful assistant."
- API endpoint: "https://api.openai.com/v1/chat/completions"

## Error Handling
- Invalid API key → descriptive error
- Network issues → retry logic not needed (single attempt)
- API rate limits → pass through error message
- Empty/invalid prompt → validation error
- Missing environment variable → clear error message

This plan ensures the component will pass all validation checks and build successfully on the first attempt.
