# OpenAI Chat Component Plan

Made by Cursor agent

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
This component will:
1. Take a prompt as input via ABI-encoded string
2. Send a request to OpenAI's API with the prompt
3. Return the API response

## Flow
```
Input (ABI string) -> Decode -> OpenAI API Request -> Process Response -> Return
```

## Required Imports
```rust
use alloy_sol_types::{sol, SolCall, SolValue};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use wavs_wasi_utils::http::{fetch_json, http_request_post_json};
use wstd::{http::HeaderValue, runtime::block_on};
```

## Validation Checklist

1. Common errors:
   - [x] Use `{ workspace = true }` in Cargo.toml
   - [x] Verify API response structure (checked with curl)
   - [x] Implement Guest trait and export component
   - [x] Use `export!(Component with_types_in bindings)`
   - [x] Use `clone()` before consuming data
   - [x] Derive `Clone` for API response structures
   - [x] Proper ABI decoding
   - [x] Use `ok_or_else()` for Option types
   - [x] Use string parameters for CLI testing
   - [x] Use `.to_string()` for string literals
   - [x] Don't edit bindings.rs

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
   - [x] Includes all required traits and types
   - [x] Uses correct import paths
   - [x] Properly imports SolCall for encoding
   - [x] All methods and types used properly
   - [x] All dependencies in Cargo.toml with `{workspace = true}`

7. Component structure:
   - [x] Uses proper sol! macro
   - [x] Correctly defines Solidity types
   - [x] Implements required functions

8. Security:
   - [x] No hardcoded API keys
   - [x] Uses environment variables for API key

9. Dependencies:
   - [x] Uses workspace dependencies correctly
   - [x] Includes all required dependencies

10. Solidity types:
    - [x] Properly imports sol macro
    - [x] Uses solidity module correctly
    - [x] Handles string conversions safely

11. Network requests:
    - [x] Uses block_on for async functions
    - [x] Uses fetch_json with correct headers
    - [x] API endpoint tested with curl
    - [x] Uses #[serde(default)] and Option<T> for API fields

## API Response Structure
```json
{
  "id": "chatcmpl-123",
  "object": "chat.completion",
  "created": 1677652288,
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "The response text here"
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 9,
    "completion_tokens": 12,
    "total_tokens": 21
  }
}
```

## Implementation Plan
1. Create component directory and copy necessary files
2. Create Cargo.toml with required dependencies
3. Implement trigger.rs with proper ABI handling
4. Implement lib.rs with OpenAI API integration
5. Add component to workspace members
6. Validate and build component 
