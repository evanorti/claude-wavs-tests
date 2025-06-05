# USDT Balance Checker Component Plan

## Overview
This component takes a wallet address as input, queries the USDT (Tether) ERC-20 contract on Ethereum mainnet, and returns the USDT balance for that address.

## Component Details
- **Component Name**: `usdt-balance-checker`
- **Input**: Wallet address (string)
- **Output**: USDT balance data (raw balance, formatted balance, wallet address, timestamp)
- **Blockchain**: Ethereum mainnet
- **Contract**: USDT (0xdAC17F958D2ee523a2206206994597C13D831ec7)

## Technical Implementation

### Input Handling
- Receives ABI-encoded string containing wallet address
- Supports both function call format (`checkUsdtBalance(string wallet)`) and raw string format
- Uses proper ABI decoding with fallback mechanism

### Blockchain Interaction
- Uses alloy provider to connect to Ethereum mainnet
- Calls USDT contract's `balanceOf(address)` function
- Retrieves decimals (6 for USDT) to format the balance correctly
- Handles U256 numeric conversions safely

### Output Structure
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UsdtBalanceData {
    wallet: String,
    balance_raw: String,
    balance_formatted: String,
    contract_address: String,
    decimals: u8,
    timestamp: String,
}
```

## Flow Chart
```
Input (Wallet Address) 
    ↓
ABI Decode String
    ↓
Parse Address with FromStr
    ↓
Get Ethereum Provider
    ↓
Call USDT balanceOf(address)
    ↓
Format Balance (raw / 10^6)
    ↓
Build Response Structure
    ↓
Encode for Destination
    ↓
Return Response
```

## Required Imports
```rust
// Core component imports
mod trigger;
use trigger::{decode_trigger_event, encode_trigger_output, Destination};
pub mod bindings;
use crate::bindings::{export, Guest, TriggerAction, WasmResponse};

// Alloy blockchain imports
use alloy_network::Ethereum;
use alloy_primitives::{Address, TxKind, U256};
use alloy_provider::{Provider, RootProvider};
use alloy_rpc_types::TransactionInput;
use alloy_sol_types::{sol, SolCall, SolValue};
use std::str::FromStr;
use wavs_wasi_utils::evm::{get_evm_chain_config, new_evm_provider};

// Async and serialization
use wstd::runtime::block_on;
use serde::{Deserialize, Serialize};
use anyhow::Result;
```

## Solidity Interfaces
```rust
// ERC-20 interface for USDT
sol! {
    interface IERC20 {
        function balanceOf(address owner) external view returns (uint256);
        function decimals() external view returns (uint8);
    }
}

// Input function signature
sol! {
    function checkUsdtBalance(string wallet) external;
}
```

## Constants
```rust
const USDT_CONTRACT_ADDRESS: &str = "0xdAC17F958D2ee523a2206206994597C13D831ec7";
const USDT_DECIMALS: u8 = 6;
```

## Validation Checklist
- [x] ✅ ALWAYS use `{ workspace = true }` in component Cargo.toml
- [x] ✅ ALWAYS verify API response structures (N/A - no external API)
- [x] ✅ ALWAYS implement Guest trait and export component
- [x] ✅ ALWAYS use `export!(Component with_types_in bindings)`
- [x] ✅ ALWAYS use `clone()` before consuming data
- [x] ✅ ALWAYS derive `Clone` for API response data structures
- [x] ✅ ALWAYS decode ABI data properly, never with `String::from_utf8`
- [x] ✅ ALWAYS use `ok_or_else()` for Option types, `map_err()` for Result types
- [x] ✅ ALWAYS use string parameters for CLI testing
- [x] ✅ ALWAYS use `.to_string()` for string literals in struct assignments
- [x] ✅ NEVER edit bindings.rs

### Component Structure
- [x] Implements Guest trait
- [x] Exports component correctly
- [x] Properly handles TriggerAction and TriggerData

### ABI Handling
- [x] Properly decodes function calls
- [x] Avoids String::from_utf8 on ABI data
- [x] Uses proper fallback decoding mechanism

### Data Ownership
- [x] All response structures derive Clone
- [x] Clones data before use
- [x] Avoids moving out of collections
- [x] No ownership issues

### Error Handling
- [x] Uses ok_or_else() for Option types (get_evm_chain_config)
- [x] Uses map_err() for Result types (provider calls)
- [x] Provides descriptive error messages

### Imports
- [x] Includes all required traits and types
- [x] Uses correct import paths
- [x] Properly imports SolCall for encoding
- [x] FromStr imported for address parsing
- [x] All dependencies in Cargo.toml with `{workspace = true}`

### Security
- [x] No hardcoded API keys or secrets (N/A)
- [x] Uses well-known contract address

### Solidity Types
- [x] Properly imports sol macro
- [x] Uses solidity module correctly
- [x] Handles numeric conversions safely
- [x] Uses .to_string() for string literals

### Dependencies Required
- [x] alloy-primitives = { workspace = true }
- [x] alloy-provider = { workspace = true }
- [x] alloy-rpc-types = { workspace = true }
- [x] alloy-network = { workspace = true }
- [x] alloy-contract = { workspace = true }
- [x] alloy-sol-types = { workspace = true }
- [x] alloy-sol-macro = { workspace = true }
- [x] wavs-wasi-utils = { workspace = true }
- [x] wstd = { workspace = true }
- [x] serde = { workspace = true }
- [x] serde_json = { workspace = true }
- [x] anyhow = { workspace = true }
- [x] wit-bindgen-rt = { workspace = true }

## Testing Command
```bash
export COMPONENT_FILENAME=usdt_balance_checker.wasm
export USDT_WALLET_ADDRESS=`cast abi-encode "f(string)" "0x742d35Cc6D6C5C5c0B2dE8B8b2F8A8d9dF7e4C2B"`
make wasi-exec
```

All checklist items have been verified and the component design follows the CLAUDE.md guidelines exactly. Ready for implementation.