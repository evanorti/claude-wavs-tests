mod trigger;
use trigger::{decode_trigger_event, encode_trigger_output, Destination};
use wavs_wasi_utils::{
    evm::alloy_primitives::hex,
    http::{fetch_json, http_request_get},
};
pub mod bindings; // Never edit bindings.rs!
use crate::bindings::{export, Guest, TriggerAction, WasmResponse};
use alloy_sol_types::SolValue;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use wstd::{http::HeaderValue, runtime::block_on};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Brewery {
    pub id: Option<String>,
    pub name: Option<String>,
    pub brewery_type: Option<String>,
    pub address_1: Option<String>,
    pub address_2: Option<String>,
    pub address_3: Option<String>,
    pub city: Option<String>,
    pub state_province: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub longitude: Option<f64>,
    pub latitude: Option<f64>,
    pub phone: Option<String>,
    pub website_url: Option<String>,
    pub state: Option<String>,
    pub street: Option<String>,
}

struct Component;
export!(Component with_types_in bindings);

impl Guest for Component {
    fn run(action: TriggerAction) -> std::result::Result<Option<WasmResponse>, String> {
        let (trigger_id, req, dest) =
            decode_trigger_event(action.data).map_err(|e| e.to_string())?;

        // Decode trigger data inline - handles hex string input
        let zip_code = {
            let input_str = String::from_utf8(req.clone())
                .map_err(|e| format!("Input is not valid UTF-8: {}", e))?;
            let hex_data = if input_str.starts_with("0x") {
                hex::decode(&input_str[2..])
                    .map_err(|e| format!("Failed to decode hex string: {}", e))?
            } else {
                req.clone()
            };
            <String as SolValue>::abi_decode(&hex_data)
                .map_err(|e| format!("Failed to decode input as ABI string: {}", e))?
        };

        let res = block_on(async move {
            let breweries = fetch_breweries(&zip_code).await?;
            serde_json::to_vec(&breweries).map_err(|e| e.to_string())
        })?;

        let output = match dest {
            Destination::Ethereum => Some(encode_trigger_output(trigger_id, &res)),
            Destination::CliOutput => Some(WasmResponse { payload: res.into(), ordering: None }),
        };
        Ok(output)
    }
}

async fn fetch_breweries(zip_code: &str) -> Result<Vec<Brewery>, String> {
    let url =
        format!("https://api.openbrewerydb.org/v1/breweries?by_postal={}&per_page=3", zip_code);
    let mut req = http_request_get(&url).map_err(|e| format!("Failed to create request: {}", e))?;
    req.headers_mut().insert("Accept", HeaderValue::from_static("application/json"));
    req.headers_mut().insert("Content-Type", HeaderValue::from_static("application/json"));
    req.headers_mut().insert("User-Agent", HeaderValue::from_static("Mozilla/5.0"));
    let breweries: Vec<Brewery> = fetch_json(req)
        .await
        .map_err(|e| format!("Failed to fetch or parse brewery data: {}", e))?;
    Ok(breweries)
}
