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
use std::str::FromStr;
use wavs_wasi_utils::evm::new_evm_provider;
use wstd::runtime::block_on;

const USDT_CONTRACT_ADDRESS: &str = "0xdAC17F958D2ee523a2206206994597C13D831ec7";
const USDT_DECIMALS: u8 = 6;

sol! {
    interface IERC20 {
        function balanceOf(address owner) external view returns (uint256);
        function decimals() external view returns (uint8);
    }
}

sol! {
    function checkUsdtBalance(string wallet) external;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UsdtBalanceData {
    wallet: String,
    balance_raw: String,
    balance_formatted: String,
    contract_address: String,
    decimals: u8,
    timestamp: String,
}

struct Component;
export!(Component with_types_in bindings);

impl Guest for Component {
    fn run(action: TriggerAction) -> std::result::Result<Option<WasmResponse>, String> {
        let (trigger_id, req, dest) =
            decode_trigger_event(action.data).map_err(|e| e.to_string())?;

        let req_clone = req.clone();

        // Decode the string using proper ABI decoding
        let wallet_address_str =
            if let Ok(decoded) = trigger::solidity::checkUsdtBalanceCall::abi_decode(&req_clone) {
                // If it has a function selector (from cast abi-encode "f(string)" format)
                decoded.wallet
            } else {
                // Fallback: try decoding just as a string parameter (no function selector)
                match <String as SolValue>::abi_decode(&req_clone) {
                    Ok(s) => s,
                    Err(e) => return Err(format!("Failed to decode input as ABI string: {}", e)),
                }
            };

        let res = block_on(async move {
            let balance_data = get_usdt_balance(&wallet_address_str).await?;
            serde_json::to_vec(&balance_data).map_err(|e| e.to_string())
        })?;

        let output = match dest {
            Destination::Ethereum => Some(encode_trigger_output(trigger_id, &res)),
            Destination::CliOutput => Some(WasmResponse { payload: res.into(), ordering: None }),
        };
        Ok(output)
    }
}

async fn get_usdt_balance(wallet_address_str: &str) -> Result<UsdtBalanceData, String> {
    let wallet_address = Address::from_str(wallet_address_str)
        .map_err(|e| format!("Invalid wallet address: {}", e))?;

    let usdt_address = Address::from_str(USDT_CONTRACT_ADDRESS)
        .map_err(|e| format!("Invalid USDT contract address: {}", e))?;

    let chain_config = get_evm_chain_config("ethereum")
        .ok_or_else(|| "Failed to get Ethereum chain config".to_string())?;

    let provider: RootProvider<Ethereum> =
        new_evm_provider::<Ethereum>(chain_config.http_endpoint.unwrap());

    let balance_call = IERC20::balanceOfCall { owner: wallet_address };
    let tx = alloy_rpc_types::eth::TransactionRequest {
        to: Some(TxKind::Call(usdt_address)),
        input: TransactionInput { input: Some(balance_call.abi_encode().into()), data: None },
        ..Default::default()
    };

    let result = provider.call(tx).await.map_err(|e| e.to_string())?;
    let balance_raw: U256 = U256::from_be_slice(&result);

    let formatted_balance = format_usdt_amount(balance_raw, USDT_DECIMALS);

    Ok(UsdtBalanceData {
        wallet: wallet_address_str.to_string(),
        balance_raw: balance_raw.to_string(),
        balance_formatted: formatted_balance,
        contract_address: USDT_CONTRACT_ADDRESS.to_string(),
        decimals: USDT_DECIMALS,
        timestamp: get_current_timestamp(),
    })
}

fn format_usdt_amount(amount: U256, decimals: u8) -> String {
    let mut divisor = U256::from(1);
    for _ in 0..decimals {
        divisor = divisor * U256::from(10);
    }
    let formatted_amount = amount / divisor;
    formatted_amount.to_string()
}

fn get_current_timestamp() -> String {
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(duration) => duration.as_secs().to_string(),
        Err(_) => "0".to_string(),
    }
}
