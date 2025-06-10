# OpenAI Chat Component Plan

## Overview
This component will:
1. Take a prompt as input
2. Send it to OpenAI's API
3. Return the response

## Validation Checklist

1. Common errors:
   - [x] Use `{ workspace = true }` in Cargo.toml
   - [x] Verify API response structure with curl
   - [x] Implement Guest trait and export component
   - [x] Use `export!(Component with_types_in bindings)`
   - [x] Use `clone()` before consuming data
   - [x] Derive `Clone` for API response structures
   - [x] Proper ABI decoding
   - [x] Use `ok_or_else()` for Option types
   - [x] Use string parameters for CLI testing
   - [x] Use `.to_string()` for string literals
   - [x] Never edit bindings.rs

2. Component structure:
   - [x] Implements Guest trait
   - [x] Exports component correctly
   - [x] Properly handles TriggerAction and TriggerData

3. ABI handling:
   - [x] Properly decodes function calls
   - [x] Avoids String::from_utf8 on ABI data

4. Data ownership:
   - [x] All API structures derive Clone
   - [x] Clones data before use
   - [x] Avoids ownership issues

5. Error handling:
   - [x] Uses ok_or_else() for Option types
   - [x] Uses map_err() for Result types
   - [x] Provides descriptive error messages

6. Imports:
   - [x] Required traits and types
   - [x] Correct import paths
   - [x] Proper SolCall imports
   - [x] All methods and types used properly
   - [x] Both structs and traits imported
   - [x] All dependencies in Cargo.toml

7. Component structure:
   - [x] Proper sol! macro usage
   - [x] Correct Solidity types
   - [x] Required functions implemented

8. Security:
   - [x] No hardcoded API keys
   - [x] Uses environment variables

9. Dependencies:
   - [x] Uses workspace dependencies
   - [x] All required dependencies included

10. Solidity types:
    - [x] Proper sol macro usage
    - [x] Correct solidity module usage
    - [x] Safe numeric conversions
    - [x] Proper string handling

11. Network requests:
    - [x] Uses block_on for async
    - [x] Uses fetch_json with headers
    - [x] API endpoints tested with curl
    - [x] Uses #[serde(default)] and Option<T>

## Required Imports
```rust
use alloy_sol_types::{sol, SolCall, SolValue};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use wavs_wasi_utils::{
    evm::alloy_primitives::hex,
    http::{fetch_json, http_request_post_json},
};
use wstd::{http::HeaderValue, runtime::block_on};
```

## API Response Structure
```rust
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
```

## Component Flow
1. Receive prompt input
2. Create OpenAI API request with proper headers
3. Send request to OpenAI API
4. Parse response
5. Return formatted response

## Testing
Test with curl:
```bash
curl -X POST https://api.openai.com/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -d '{
    "model": "gpt-4",
    "messages": [
      {"role": "system", "content": "You are a helpful assistant."},
      {"role": "user", "content": "Hello!"}
    ]
  }'
``` 
