mod trigger;
use trigger::{decode_trigger_event, encode_trigger_output, Destination};
use wavs_wasi_utils::http::{fetch_json, http_request_get};
pub mod bindings;
use crate::bindings::{export, Guest, TriggerAction, WasmResponse};
use alloy_sol_types::{SolCall, SolValue};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use wstd::{http::HeaderValue, runtime::block_on};

struct Component;
export!(Component with_types_in bindings);

impl Guest for Component {
    fn run(action: TriggerAction) -> std::result::Result<Option<WasmResponse>, String> {
        let (trigger_id, req, dest) =
            decode_trigger_event(action.data).map_err(|e| e.to_string())?;

        // Clone request data to avoid ownership issues
        let req_clone = req.clone();

        // Decode the zip code string using proper ABI decoding
        let zip_code =
            if let Ok(decoded) = trigger::solidity::findBreweriesCall::abi_decode(&req_clone) {
                // If it has a function selector (from cast abi-encode "f(string)" format)
                decoded.zipCode
            } else {
                // Fallback: try decoding just as a string parameter (no function selector)
                match <String as SolValue>::abi_decode(&req_clone) {
                    Ok(s) => s,
                    Err(e) => return Err(format!("Failed to decode input as ABI string: {}", e)),
                }
            };

        println!("Decoded zip code input: {}", zip_code);

        // Fetch brewery data
        let result = block_on(async move {
            let brewery_data = fetch_breweries(&zip_code).await?;
            serde_json::to_vec(&brewery_data).map_err(|e| e.to_string())
        })?;

        let output = match dest {
            Destination::Ethereum => Some(encode_trigger_output(trigger_id, &result)),
            Destination::CliOutput => Some(WasmResponse { payload: result.into(), ordering: None }),
        };
        Ok(output)
    }
}

async fn fetch_breweries(zip_code: &str) -> Result<BreweryResults, String> {
    let url = format!("https://api.openbrewerydb.org/v1/breweries?by_postal={}", zip_code);

    let mut req = http_request_get(&url).map_err(|e| e.to_string())?;
    req.headers_mut().insert("Accept", HeaderValue::from_static("application/json"));
    req.headers_mut().insert("Content-Type", HeaderValue::from_static("application/json"));
    req.headers_mut().insert("User-Agent", HeaderValue::from_static("Mozilla/5.0"));

    let breweries: Vec<Brewery> = fetch_json(req).await.map_err(|e| e.to_string())?;

    // Get current timestamp
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string();

    Ok(BreweryResults {
        zip_code: zip_code.to_string(),
        brewery_count: breweries.len(),
        breweries,
        timestamp,
    })
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Brewery {
    id: Option<String>,
    name: Option<String>,
    brewery_type: Option<String>,
    address_1: Option<String>,
    address_2: Option<String>,
    address_3: Option<String>,
    city: Option<String>,
    state_province: Option<String>,
    postal_code: Option<String>,
    country: Option<String>,
    longitude: Option<f64>,
    latitude: Option<f64>,
    phone: Option<String>,
    website_url: Option<String>,
    state: Option<String>,
    street: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BreweryResults {
    zip_code: String,
    brewery_count: usize,
    breweries: Vec<Brewery>,
    timestamp: String,
}
