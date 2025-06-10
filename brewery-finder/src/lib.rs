mod trigger;
use trigger::{decode_trigger_event, encode_trigger_output, Destination};
use wavs_wasi_utils::{
    evm::alloy_primitives::hex,
    http::{fetch_json, http_request_get},
};
pub mod bindings;
use crate::bindings::{export, Guest, TriggerAction, WasmResponse};
use alloy_sol_types::SolValue;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use wstd::{http::HeaderValue, runtime::block_on};

struct Component;
export!(Component with_types_in bindings);

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
pub struct BreweryFinderResult {
    zip_code: String,
    brewery_count: usize,
    breweries: Vec<Brewery>,
}

impl Guest for Component {
    fn run(action: TriggerAction) -> std::result::Result<Option<WasmResponse>, String> {
        let (trigger_id, req, dest) =
            decode_trigger_event(action.data).map_err(|e| e.to_string())?;

        // Decode trigger data inline - handles hex string input
        let zip_code = {
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
        println!("Looking up breweries for zip code: {}", zip_code);

        // Find breweries in the zip code
        let result = block_on(async move {
            let brewery_data = find_breweries(&zip_code).await?;
            serde_json::to_vec(&brewery_data).map_err(|e| e.to_string())
        })?;

        let output = match dest {
            Destination::Ethereum => Some(encode_trigger_output(trigger_id, &result)),
            Destination::CliOutput => Some(WasmResponse { payload: result.into(), ordering: None }),
        };
        Ok(output)
    }
}

async fn find_breweries(zip_code: &str) -> Result<BreweryFinderResult, String> {
    // Create API URL for OpenBreweryDB
    let url =
        format!("https://api.openbrewerydb.org/v1/breweries?by_postal={}&per_page=20", zip_code);

    // Create request with headers
    let mut req = http_request_get(&url).map_err(|e| format!("Failed to create request: {}", e))?;

    req.headers_mut().insert("Accept", HeaderValue::from_static("application/json"));
    req.headers_mut().insert("Content-Type", HeaderValue::from_static("application/json"));
    req.headers_mut().insert("User-Agent", HeaderValue::from_static("Mozilla/5.0"));

    // Make API request
    let breweries: Vec<Brewery> =
        fetch_json(req).await.map_err(|e| format!("Failed to fetch breweries: {}", e))?;

    // Create result data
    let brewery_count = breweries.len();
    let breweries_clone = breweries.clone();

    Ok(BreweryFinderResult {
        zip_code: zip_code.to_string(),
        brewery_count,
        breweries: breweries_clone,
    })
}
