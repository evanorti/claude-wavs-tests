# USDT Balance Checker Component Plan (Using alloy-contract)

## Overview
This component demonstrates using the `alloy-contract` crate to check USDT balances. It provides the same functionality as the original USDT balance checker but uses the higher-level `alloy-contract` API instead of direct provider calls.

## Component Details
- **Component Name**: `usdt-balance-checker-contract`
- **Input**: Wallet address (string)
- **Output**: USDT balance data (raw balance, formatted balance, wallet address, timestamp)
- **Blockchain**: Ethereum mainnet
- **Contract**: USDT (0xdAC17F958D2ee523a2206206994597C13D831ec7)
- **Key Difference**: Uses `alloy-contract` for contract interactions

## Technical Implementation

### Input Handling
- Receives ABI-encoded string containing wallet address
- Supports both function call format (`checkUsdtBalance(string wallet)`) and raw string format
- Uses proper ABI decoding with fallback mechanism

### Blockchain Interaction (alloy-contract approach)
- Uses `alloy-contract` to create a contract instance
- Calls contract methods directly: `contract.balanceOf(owner).call().await`
- Demonstrates the difference in API ergonomics compared to manual provider calls

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
Create Contract Instance with alloy-contract
    ↓
Call contract.balanceOf(address).call().await
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

// Alloy blockchain imports (contract-focused)
use alloy_contract::Contract;
use alloy_network::Ethereum;
use alloy_primitives::{Address, U256};
use alloy_provider::{Provider, RootProvider};
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

## Key Differences from Direct Provider Approach

### Direct Provider (Original)
```rust
let balance_call = IERC20::balanceOfCall { owner: wallet_address };
let tx = alloy_rpc_types::eth::TransactionRequest {
    to: Some(TxKind::Call(usdt_address)),
    input: TransactionInput { input: Some(balance_call.abi_encode().into()), data: None },
    ..Default::default()
};
let result = provider.call(&tx).await?;
let balance_raw: U256 = U256::from_be_slice(&result);
```

### alloy-contract Approach (This Component)
```rust
let contract = Contract::new(usdt_address, IERC20::abi(), provider);
let balance_raw: U256 = contract.balanceOf(wallet_address).call().await?;
```

## Dependencies Required
```rust
// Additional dependency for this version
alloy-contract = { workspace = true }

// Standard dependencies
alloy-primitives = { workspace = true }
alloy-provider = { workspace = true }
alloy-network = { workspace = true }
alloy-sol-types = { workspace = true }
alloy-sol-macro = { workspace = true }
wavs-wasi-utils = { workspace = true }
wstd = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
anyhow = { workspace = true }
wit-bindgen-rt = { workspace = true }
```

## Validation Checklist
- [x] ✅ Component demonstrates alloy-contract usage
- [x] ✅ Maintains same functionality as original USDT checker
- [x] ✅ Uses proper error handling
- [x] ✅ Follows WAVS component structure
- [x] ✅ Derives Clone for data structures
- [x] ✅ Uses workspace dependencies
- [x] ✅ Implements Guest trait correctly

## Testing Command
```bash
export COMPONENT_FILENAME=usdt_balance_checker_contract.wasm
export USDT_WALLET_ADDRESS=`cast abi-encode "f(string)" "0x742d35Cc6D6C5C5c0B2dE8B8b2F8A8d9dF7e4C2B"`
make wasi-exec
```

This component serves as a comparison to show when and how `alloy-contract` can be used, even though for this simple use case, the direct provider approach is more appropriate.