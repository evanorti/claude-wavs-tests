use crate::bindings::wavs::worker::layer_types::{
    TriggerData, TriggerDataEvmContractEvent, WasmResponse,
};
use alloy_sol_types::SolValue;
use anyhow::Result;
use wavs_wasi_utils::decode_event_log_data;

/// Represents the destination where the trigger output should be sent
pub enum Destination {
    Ethereum,
    CliOutput,
}

/// Decodes incoming trigger event data into its components
pub fn decode_trigger_event(trigger_data: TriggerData) -> Result<(u64, Vec<u8>, Destination)> {
    match trigger_data {
        TriggerData::EvmContractEvent(TriggerDataEvmContractEvent { log, .. }) => {
            let event: solidity::NewTrigger = decode_event_log_data!(log)?;
            let trigger_info =
                <solidity::TriggerInfo as SolValue>::abi_decode(&event._triggerInfo)?;
            Ok((trigger_info.triggerId, trigger_info.data.to_vec(), Destination::Ethereum))
        }
        TriggerData::Raw(data) => Ok((0, data.clone(), Destination::CliOutput)),
        _ => Err(anyhow::anyhow!("Unsupported trigger data type")),
    }
}

/// Encodes the output data for submission back to Ethereum
pub fn encode_trigger_output(trigger_id: u64, output: impl AsRef<[u8]>) -> WasmResponse {
    WasmResponse {
        payload: solidity::DataWithId {
            triggerId: trigger_id,
            data: output.as_ref().to_vec().into(),
        }
        .abi_encode(),
        ordering: None,
    }
}

/// Solidity type definitions for the OpenAI chat component
pub mod solidity {
    use alloy_sol_macro::sol;
    pub use ITypes::*;

    // Import the main interface types
    sol!("../../src/interfaces/ITypes.sol");

    // Define the function signature for generating AI responses
    sol! {
        function generateResponse(string prompt) external;
    }
}
