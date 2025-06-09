# USDT Balance Checker Component Plan

## Overview
This component takes a wallet address as input, queries the USDT (Tether) contract on Ethereum, and returns the USDT balance for that address.

## Component Details
- **Name**: usdt-balance-checker
- **Input**: Wallet address (string)
- **Output**: USDT balance data with formatted and raw amounts
- **Contract**: USDT (Tether) contract address: `0xdAC17F958D2ee523a2206206994597C13D831ec7`

## Implementation Flow Chart
```
Input: Wallet Address (string)
    ↓
Decode ABI-encoded input with hex string support
    ↓
Parse wallet address using Address::from_str
    ↓
Get Ethereum provider using get_evm_chain_config
    ↓
Call USDT.balanceOf(wallet_address)
    ↓
Call USDT.decimals() to get token decimals (6 for USDT)
    ↓
Format balance by dividing by 10^decimals
    ↓
Return UsdtBalanceData struct with raw and formatted balance
```

## Validation Checklist
- [x] ✅ ALWAYS use `{ workspace = true }` in component Cargo.toml
- [x] ✅ ALWAYS verify API response structures (no external APIs used, blockchain only)
- [x] ✅ ALWAYS Read documentation provided (CLAUDE.md reviewed)
- [x] ✅ ALWAYS implement the Guest trait and export component
- [x] ✅ ALWAYS use `export!(Component with_types_in bindings)`
- [x] ✅ ALWAYS use `clone()` before consuming data to avoid ownership issues
- [x] ✅ ALWAYS derive `Clone` for API response data structures
- [x] ✅ ALWAYS decode ABI data properly, never with `String::from_utf8`
- [x] ✅ ALWAYS use `ok_or_else()` for Option types, `map_err()` for Result types
- [x] ✅ ALWAYS use string parameters for CLI testing
- [x] ✅ ALWAYS use `.to_string()` to convert string literals to String types
- [x] ✅ NEVER edit bindings.rs - it's auto-generated

## Required Imports
```rust
// Core component imports
pub mod bindings;
mod trigger;
use trigger::{decode_trigger_event, encode_trigger_output, Destination};
use crate::bindings::{export, Guest, TriggerAction, WasmResponse};
use crate::bindings::host::get_evm_chain_config;

// Blockchain interaction imports
use alloy_network::Ethereum;
use alloy_primitives::{Address, TxKind, U256};
use alloy_provider::{Provider, RootProvider};
use alloy_rpc_types::TransactionInput;
use alloy_sol_types::{sol, SolCall, SolValue};
use std::str::FromStr;
use wavs_wasi_utils::evm::{alloy_primitives::hex, new_evm_provider};

// Async and serialization imports
use wstd::runtime::block_on;
use serde::{Deserialize, Serialize};
use anyhow::Result;
```

## Data Structures
```rust
// Response structure - MUST derive Clone
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UsdtBalanceData {
    wallet: String,
    balance_raw: String,
    balance_formatted: String,
    usdt_contract: String,
    decimals: u8,
    timestamp: String,
}

// Solidity interface for USDT (ERC20)
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

## Implementation Details
1. **Input Processing**: Handle ABI-encoded string input with hex string support
2. **Address Validation**: Parse wallet address using `Address::from_str`
3. **Blockchain Query**: Use Ethereum provider to call USDT contract
4. **Balance Formatting**: Handle USDT's 6 decimal places correctly
5. **Error Handling**: Proper error messages for invalid addresses and contract calls

## Dependencies in Cargo.toml
All dependencies will use `{ workspace = true }`:
- wit-bindgen-rt
- wavs-wasi-utils 
- serde
- serde_json
- alloy-sol-macro
- wstd
- alloy-sol-types
- anyhow
- alloy-primitives (for blockchain interactions)
- alloy-provider (for blockchain interactions)
- alloy-rpc-types (for blockchain interactions)
- alloy-network (for blockchain interactions)
- alloy-contract (for blockchain interactions)

## Testing
Component will be tested with:
```bash
export COMPONENT_FILENAME=usdt_balance_checker.wasm
export INPUT_DATA="0x742d35Cc6634C0532925a3b8D84c8C0b1b39a7d"  # Example wallet address
make wasi-exec
```

## Security Considerations
- No API keys needed (blockchain-only)
- Uses environment chain configuration
- Validates wallet address format before blockchain calls
- Uses proper error handling for invalid inputs

This plan ensures the component will build correctly, pass all validation checks, and execute successfully on the first try.