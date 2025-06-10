# USDT Balance Checker Component Plan

## Overview
This component takes a wallet address as input, queries the USDT (Tether) contract on Ethereum, and returns the USDT balance of that address along with formatted balance information.

## Component Design

### Input
- **Type**: String (wallet address)
- **Format**: Ethereum address (0x...)
- **Encoding**: ABI-encoded string parameter

### Output
- **Type**: JSON object containing:
  - `wallet`: Input wallet address
  - `balance_raw`: Raw USDT balance (in smallest units)
  - `balance_formatted`: Formatted balance (divided by decimals)
  - `token_contract`: USDT contract address
  - `token_symbol`: "USDT"
  - `decimals`: Token decimals (6 for USDT)
  - `timestamp`: Current timestamp

### Processing Flow
1. Decode ABI-encoded wallet address from trigger data
2. Parse wallet address using `Address::from_str`
3. Connect to Ethereum provider using chain config
4. Query USDT contract for:
   - Balance using `balanceOf(address)`
   - Decimals using `decimals()`
5. Format balance by dividing by 10^decimals
6. Return structured data

### Contract Details
- **USDT Contract Address**: `0xdAC17F958D2ee523a2206206994597C13D831ec7`
- **Network**: Ethereum Mainnet
- **Standard**: ERC-20
- **Decimals**: 6

### Required Imports
```rust
// Core WAVS imports
mod trigger;
use trigger::{decode_trigger_event, encode_trigger_output, Destination};
pub mod bindings;
use crate::bindings::{export, Guest, TriggerAction, WasmResponse};
use crate::bindings::host::get_evm_chain_config;

// Alloy blockchain interaction imports
use alloy_network::Ethereum;
use alloy_primitives::{Address, TxKind, U256};
use alloy_provider::{Provider, RootProvider};
use alloy_rpc_types::TransactionInput;
use alloy_sol_types::{sol, SolCall, SolValue};

// Utility imports
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use wavs_wasi_utils::{
    evm::{alloy_primitives::hex, new_evm_provider},
};
use wstd::runtime::block_on;
```

### Required Dependencies (Cargo.toml)
All with `{ workspace = true }`:
- `wit-bindgen-rt`
- `wavs-wasi-utils`
- `serde`
- `serde_json`
- `alloy-sol-macro`
- `wstd`
- `alloy-sol-types`
- `anyhow`
- `alloy-primitives`
- `alloy-provider`
- `alloy-rpc-types`
- `alloy-network`

## Validation Checklist

### Common Errors
- [x] ✅ Use `{ workspace = true }` in component Cargo.toml
- [x] ✅ No API endpoints to verify with curl (blockchain interaction only)
- [x] ✅ Read CLAUDE.md documentation
- [x] ✅ Implement Guest trait and export component
- [x] ✅ Use `export!(Component with_types_in bindings)`
- [x] ✅ Use `clone()` before consuming data
- [x] ✅ Derive `Clone` for response data structures
- [x] ✅ Decode ABI data properly with hex string support
- [x] ✅ Use `ok_or_else()` for Option types, `map_err()` for Result types
- [x] ✅ Use string parameters for CLI testing
- [x] ✅ Use `.to_string()` for string literals in struct fields
- [x] ✅ NEVER edit bindings.rs

### Component Structure
- [x] Implements Guest trait
- [x] Exports component correctly with `export!(Component with_types_in bindings)`
- [x] Properly handles TriggerAction and TriggerData

### ABI Handling
- [x] Properly decodes function calls with hex string support
- [x] Avoids String::from_utf8 on ABI data
- [x] Uses `<String as SolValue>::abi_decode(&hex_data)`

### Data Ownership
- [x] All response structures derive Clone
- [x] Clones data before use
- [x] Avoids moving out of collections
- [x] Avoids ownership issues

### Error Handling
- [x] Uses `ok_or_else()` for Option types (chain config)
- [x] Uses `map_err()` for Result types (provider calls)
- [x] Provides descriptive error messages

### Imports
- [x] Includes all required traits and types
- [x] Uses correct import paths
- [x] Imports SolCall for encoding
- [x] All methods and types properly imported
- [x] Both structs and traits imported
- [x] All dependencies in Cargo.toml with `{workspace = true}`
- [x] No unused imports

### Component Structure
- [x] Uses proper sol! macro with correct syntax
- [x] Correctly defines Solidity types in solidity module
- [x] Implements IERC20 interface functions

### Security
- [x] No hardcoded API keys or secrets (blockchain interaction only)
- [x] No sensitive data exposure

### Dependencies
- [x] Uses workspace dependencies correctly
- [x] Includes all required dependencies

### Solidity Types
- [x] Properly imports sol macro
- [x] Uses solidity module correctly
- [x] Handles numeric conversions safely with string parsing
- [x] Uses .to_string() for string literals in struct initialization

### Network Requests
- [x] Uses block_on for async functions
- [x] Blockchain interaction instead of HTTP requests
- [x] No external API endpoints

## Implementation Notes

1. **Token Decimals**: USDT uses 6 decimals, so raw balance must be divided by 1,000,000
2. **Numeric Conversion**: Use string parsing method for U256 conversions
3. **Address Parsing**: Use `Address::from_str()` with proper error handling
4. **Provider Setup**: Use `get_evm_chain_config("ethereum")` and `new_evm_provider`
5. **Balance Formatting**: Implement proper decimal division for user-friendly display

## Testing Command
```bash
export COMPONENT_FILENAME=usdt_balance_checker.wasm
export INPUT_DATA="0xA0b86a33E6441479a46E2C52D073b12b79a2aB59"
make wasi-exec
```

This plan ensures the component will build correctly and pass all validation checks on the first attempt.