mod trigger;
use trigger::{decode_trigger_event, encode_trigger_output, Destination};
pub mod bindings;
use crate::bindings::{export, Guest, TriggerAction, WasmResponse};
use alloy_sol_types::{SolCall, SolValue};
use anyhow::Result;
use serde::{Deserialize, Serialize};

struct Component;
export!(Component with_types_in bindings);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SquareResult {
    input: String,
    squared: String,
}

impl Guest for Component {
    fn run(action: TriggerAction) -> std::result::Result<Option<WasmResponse>, String> {
        let (trigger_id, req, dest) =
            decode_trigger_event(action.data).map_err(|e| e.to_string())?;

        let req_clone = req.clone();

        // Decode the string using proper ABI decoding
        let input_str =
            if let Ok(decoded) = trigger::solidity::squareNumberCall::abi_decode(&req_clone) {
                decoded.input
            } else {
                match <String as SolValue>::abi_decode(&req_clone) {
                    Ok(s) => s,
                    Err(e) => return Err(format!("Failed to decode input as ABI string: {}", e)),
                }
            };

        println!("Decoded input string: {}", input_str);

        // Parse as u64, square, and prepare result
        let n: u64 =
            input_str.parse().map_err(|e| format!("Failed to parse input as u64: {}", e))?;
        let squared =
            n.checked_mul(n).ok_or_else(|| "Overflow when squaring number".to_string())?;
        let result = SquareResult { input: n.to_string(), squared: squared.to_string() };
        let res = serde_json::to_vec(&result).map_err(|e| e.to_string())?;

        println!("Squared result: {}", squared);

        let output = match dest {
            Destination::Ethereum => Some(encode_trigger_output(trigger_id, &res)),
            Destination::CliOutput => Some(WasmResponse { payload: res.into(), ordering: None }),
        };
        Ok(output)
    }
}
