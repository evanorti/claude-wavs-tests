# Brewery Locator Component Plan

## Overview
This WAVS component takes a zip code as input, queries the OpenBreweryDB API, and returns brewery information for that area.

## API Research
- **Endpoint**: `https://api.openbrewerydb.org/v1/breweries?by_postal={zip_code}`
- **No API key required**: Free public API
- **Response**: Array of brewery objects with fields like name, address, brewery_type, phone, website_url, etc.
- **Testing**: Verified with NYC zip 10001 - returns 3+ breweries

## Component Flow
```
Input (zip code string) 
    ↓ 
ABI decode zip code
    ↓
HTTP request to OpenBreweryDB
    ↓
Parse JSON response
    ↓
Format brewery data
    ↓
Return JSON result
```

## Required Imports
- `alloy_sol_types::{SolCall, SolValue}` - ABI decoding
- `serde::{Deserialize, Serialize}` - JSON handling
- `wavs_wasi_utils::http::{fetch_json, http_request_get}` - HTTP requests
- `wstd::{http::HeaderValue, runtime::block_on}` - Async handling
- Standard component imports (bindings, trigger, Guest, etc.)

## Data Structures

### Input Function
```rust
sol! {
    function findBreweries(string zipCode) external;
}
```

### API Response Structure
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
```

### Output Structure
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BreweryResults {
    zip_code: String,
    brewery_count: usize,
    breweries: Vec<Brewery>,
    timestamp: String,
}
```

## Validation Checklist

### ABI Handling
- [x] ✅ Use proper ABI decoding with `abi_decode()`
- [x] ✅ NEVER use `String::from_utf8` on ABI data
- [x] ✅ Handle both function call and string parameter decoding

### Data Ownership
- [x] ✅ All API structures derive `Clone`
- [x] ✅ Clone data before use to avoid ownership issues
- [x] ✅ Use `#[serde(default)]` and `Option<T>` for API fields

### Error Handling
- [x] ✅ Use `map_err()` for Result types
- [x] ✅ Provide descriptive error messages

### Imports
- [x] ✅ All required traits and types imported
- [x] ✅ Proper import paths for HTTP functions
- [x] ✅ SolCall for encoding/decoding

### Component Structure
- [x] ✅ Implements Guest trait
- [x] ✅ Uses `export!(Component with_types_in bindings)`
- [x] ✅ Proper solidity module for types

### Security
- [x] ✅ No hardcoded API keys (OpenBreweryDB is free)
- [x] ✅ No sensitive data exposure

### Dependencies
- [x] ✅ Use `{ workspace = true }` for all dependencies
- [x] ✅ Include all required dependencies

### Network Requests
- [x] ✅ Use `block_on` for async functions
- [x] ✅ Proper headers for HTTP requests
- [x] ✅ API endpoint tested with curl

## Implementation Strategy
1. Copy required files from existing component
2. Create trigger.rs with proper ABI decoding for zip code strings
3. Create lib.rs with brewery fetching logic
4. Test with various zip codes to ensure reliability
5. Validate and build component

## Test Commands
```bash
# Test with NYC zip code
export COMPONENT_FILENAME=brewery_locator.wasm
export ZIP_CODE=`cast abi-encode "f(string)" "10001"`
make wasi-exec
```