mod trigger;
use trigger::{decode_trigger_event, encode_trigger_output, Destination};

pub mod bindings;
use crate::bindings::host::get_evm_chain_config;
use crate::bindings::{export, Guest, TriggerAction, WasmResponse};

use alloy_network::Ethereum;
use alloy_primitives::{Address, TxKind, U256};
use alloy_provider::{Provider, RootProvider};
use alloy_rpc_types::TransactionInput;
use alloy_sol_types::{sol, SolCall, SolValue};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::str::FromStr;
use wavs_wasi_utils::evm::{alloy_primitives::hex, new_evm_provider};
use wstd::runtime::block_on;

// USDT CONTRACT ADDRESS ON ETHEREUM MAINNET
const USDT_CONTRACT_ADDRESS: &str = "0xdAC17F958D2ee523a2206206994597C13D831ec7";

// ERC-20 INTERFACE
sol! {
    interface IERC20 {
        function balanceOf(address owner) external view returns (uint256);
        function decimals() external view returns (uint8);
    }
}

// RESPONSE STRUCTURE - MUST DERIVE CLONE
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UsdtBalanceData {
    wallet: String,
    balance_raw: String,
    balance_formatted: String,
    token_contract: String,
    token_symbol: String,
    decimals: u8,
}

// COMPONENT IMPLEMENTATION
struct Component;
export!(Component with_types_in bindings);

impl Guest for Component {
    fn run(action: TriggerAction) -> std::result::Result<Option<WasmResponse>, String> {
        let (trigger_id, req, dest) =
            decode_trigger_event(action.data).map_err(|e| e.to_string())?;

        // Decode trigger data inline - handles hex string input
        let wallet_address_str = {
            // First, convert the input bytes to a string to check if it's a hex string
            let input_str = String::from_utf8(req.clone())
                .map_err(|e| format!("Input is not valid UTF-8: {}", e))?;

            // Check if it's a hex string (starts with "0x")
            let hex_data = if input_str.starts_with("0x") {
                // Decode the hex string to bytes
                hex::decode(&input_str[2..])
                    .map_err(|e| format!("Failed to decode hex string: {}", e))?
            } else {
                // If it's not a hex string, assume the input is already binary data
                req.clone()
            };

            // Now ABI decode the binary data as a string parameter
            <String as SolValue>::abi_decode(&hex_data)
                .map_err(|e| format!("Failed to decode input as ABI string: {}", e))?
        };
        println!("Decoded wallet address: {}", wallet_address_str);

        // Check USDT balance
        let res = block_on(async move {
            let balance_data = get_usdt_balance(&wallet_address_str).await?;
            serde_json::to_vec(&balance_data).map_err(|e| e.to_string())
        })?;

        // Return result based on destination
        let output = match dest {
            Destination::Ethereum => Some(encode_trigger_output(trigger_id, &res)),
            Destination::CliOutput => Some(WasmResponse { payload: res.into(), ordering: None }),
        };
        Ok(output)
    }
}

// USDT BALANCE CHECKER IMPLEMENTATION
async fn get_usdt_balance(wallet_address_str: &str) -> Result<UsdtBalanceData, String> {
    // Parse wallet address
    let wallet_address = Address::from_str(wallet_address_str)
        .map_err(|e| format!("Invalid wallet address: {}", e))?;

    // Parse USDT contract address
    let usdt_address = Address::from_str(USDT_CONTRACT_ADDRESS)
        .map_err(|e| format!("Invalid USDT contract address: {}", e))?;

    // Get Ethereum provider
    let chain_config = get_evm_chain_config("ethereum")
        .ok_or_else(|| "Failed to get Ethereum chain config".to_string())?;

    let provider: RootProvider<Ethereum> =
        new_evm_provider::<Ethereum>(chain_config.http_endpoint.unwrap());

    // Get USDT balance
    let balance_call = IERC20::balanceOfCall { owner: wallet_address };
    let tx = alloy_rpc_types::eth::TransactionRequest {
        to: Some(TxKind::Call(usdt_address)),
        input: TransactionInput { input: Some(balance_call.abi_encode().into()), data: None },
        ..Default::default()
    };

    let result = provider.call(tx).await.map_err(|e| e.to_string())?;
    let balance_raw: U256 = U256::from_be_slice(&result);

    // Get USDT decimals (should be 6)
    let decimals_call = IERC20::decimalsCall {};
    let tx_decimals = alloy_rpc_types::eth::TransactionRequest {
        to: Some(TxKind::Call(usdt_address)),
        input: TransactionInput { input: Some(decimals_call.abi_encode().into()), data: None },
        ..Default::default()
    };

    let result_decimals = provider.call(tx_decimals).await.map_err(|e| e.to_string())?;
    let decimals: u8 = result_decimals[31]; // Last byte for uint8

    // Format balance - convert to human readable format
    let formatted_balance = format_token_amount(balance_raw, decimals);

    // Return data
    Ok(UsdtBalanceData {
        wallet: wallet_address_str.to_string(),
        balance_raw: balance_raw.to_string(),
        balance_formatted: formatted_balance,
        token_contract: USDT_CONTRACT_ADDRESS.to_string(),
        token_symbol: "USDT".to_string(),
        decimals,
    })
}

// Helper function to format token amount
fn format_token_amount(amount: U256, decimals: u8) -> String {
    if amount == U256::ZERO {
        return "0".to_string();
    }

    // Calculate divisor (10^decimals)
    let mut divisor = U256::from(1);
    for _ in 0..decimals {
        divisor = divisor * U256::from(10);
    }

    // Perform division
    let formatted_amount = amount / divisor;
    let remainder = amount % divisor;

    // Format with decimal places if there's a remainder
    if remainder == U256::ZERO {
        formatted_amount.to_string()
    } else {
        // Convert remainder to decimal places
        let remainder_str = remainder.to_string();
        let padding = decimals as usize - remainder_str.len();
        // SAFE: bounded by check above - decimals is u8 so max 255, and remainder_str.len() reduces this further
        let padded_remainder = "0".repeat(min(padding, 50)) + &remainder_str;

        // Remove trailing zeros
        let trimmed_remainder = padded_remainder.trim_end_matches('0');

        if trimmed_remainder.is_empty() {
            formatted_amount.to_string()
        } else {
            format!("{}.{}", formatted_amount, trimmed_remainder)
        }
    }
}
