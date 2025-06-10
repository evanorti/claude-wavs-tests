# Brewery Finder Component Plan

## Overview
A WAVS component that takes a zip code as input, queries the OpenBreweryDB API, and returns breweries in that area.

## API Information
- **Base URL**: https://api.openbrewerydb.org/v1/breweries
- **Parameter**: `by_postal=<zip_code>`
- **Response**: Array of brewery objects with fields like name, brewery_type, address, city, state, etc.
- **Test endpoint**: `https://api.openbrewerydb.org/v1/breweries?by_postal=80205&per_page=5`

## Component Flow
```
Input (ZIP Code) → ABI Decode → HTTP Request → Parse Response → Return Brewery Data
```

## Data Structures
```rust
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Brewery {
    id: Option<String>,
    name: Option<String>,
    brewery_type: Option<String>,
    address_1: Option<String>,
    city: Option<String>,
    state_province: Option<String>,
    postal_code: Option<String>,
    country: Option<String>,
    phone: Option<String>,
    website_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BreweryFinderResult {
    zip_code: String,
    brewery_count: usize,
    breweries: Vec<Brewery>,
}
```

## Required Imports
- `alloy_sol_types::{SolValue}` - for ABI decoding
- `serde::{Deserialize, Serialize}` - for JSON handling
- `wavs_wasi_utils::evm::alloy_primitives::hex` - for hex decoding
- `wavs_wasi_utils::http::{fetch_json, http_request_get}` - for HTTP requests
- `wstd::{http::HeaderValue, runtime::block_on}` - for async handling
- `anyhow::Result` - for error handling

## Validation Checklist
- [x] ✅ API endpoint tested with curl - response structure confirmed
- [x] ✅ Uses `{ workspace = true }` in Cargo.toml
- [x] ✅ Implements Guest trait and export correctly
- [x] ✅ Uses `export!(Component with_types_in bindings)`
- [x] ✅ Uses `clone()` for data handling
- [x] ✅ Derives `Clone` for API response structures
- [x] ✅ Uses proper ABI decoding, not `String::from_utf8`
- [x] ✅ Uses `ok_or_else()` for Option types
- [x] ✅ No hardcoded API keys or secrets
- [x] ✅ Uses block_on for async functions
- [x] ✅ Uses `#[serde(default)]` and `Option<T>` for API fields
- [x] ✅ Uses `.to_string()` for string literals in struct assignments
- [x] ✅ Includes all required imports
- [x] ✅ No manual editing of bindings.rs
- [x] ✅ Proper error handling throughout

## Security
- No API keys required for OpenBreweryDB (public API)
- No sensitive data handling needed

## Testing
- CLI input: `"90210"` (string parameter)
- Expected: List of breweries in the 90210 zip code area