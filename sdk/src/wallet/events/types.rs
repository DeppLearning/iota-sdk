// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use getset::Getters;
use serde::{Deserialize, Serialize};

use crate::{
    client::api::PreparedTransactionDataDto,
    types::{
        api::core::response::OutputWithMetadataResponse,
        block::{
            address::Bech32Address,
            payload::transaction::{dto::TransactionPayloadDto, TransactionId},
        },
    },
    wallet::account::types::{InclusionState, OutputDataDto},
};
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    /// Associated account index.
    pub account_index: u32,
    /// The event
    pub event: WalletEvent,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum WalletEvent {
    ConsolidationRequired,
    #[cfg(feature = "ledger_nano")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ledger_nano")))]
    LedgerAddressGeneration(AddressData),
    NewOutput(Box<NewOutputEvent>),
    SpentOutput(Box<SpentOutputEvent>),
    TransactionInclusion(TransactionInclusionEvent),
    TransactionProgress(TransactionProgressEvent),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum WalletEventType {
    ConsolidationRequired,
    #[cfg(feature = "ledger_nano")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ledger_nano")))]
    LedgerAddressGeneration,
    NewOutput,
    SpentOutput,
    TransactionInclusion,
    TransactionProgress,
}

impl TryFrom<&str> for WalletEventType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let event_type = match value {
            "ConsolidationRequired" => Self::ConsolidationRequired,
            #[cfg(feature = "ledger_nano")]
            "LedgerAddressGeneration" => Self::LedgerAddressGeneration,
            "NewOutput" => Self::NewOutput,
            "SpentOutput" => Self::SpentOutput,
            "TransactionInclusion" => Self::TransactionInclusion,
            "TransactionProgress" => Self::TransactionProgress,
            _ => return Err(format!("invalid event type {value}")),
        };
        Ok(event_type)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewOutputEvent {
    /// The new output.
    pub output: OutputDataDto,
    /// The transaction that created the output. Might be pruned and not available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction: Option<TransactionPayloadDto>,
    /// The inputs for the transaction that created the output. Might be pruned and not available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_inputs: Option<Vec<OutputWithMetadataResponse>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SpentOutputEvent {
    /// The spent output.
    pub output: OutputDataDto,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionInclusionEvent {
    pub transaction_id: TransactionId,
    pub inclusion_state: InclusionState,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TransactionProgressEvent {
    /// Performing input selection.
    SelectingInputs,
    /// Generating remainder value deposit address.
    GeneratingRemainderDepositAddress(AddressData),
    /// Prepared transaction.
    PreparedTransaction(Box<PreparedTransactionDataDto>),
    /// Prepared transaction essence hash hex encoded, required for blindsigning with a ledger nano
    PreparedTransactionEssenceHash(String),
    /// Signing the transaction.
    SigningTransaction,
    /// Performing PoW.
    PerformingPow,
    /// Broadcasting.
    Broadcasting,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AddressConsolidationNeeded {
    /// The associated address.
    pub address: Bech32Address,
}

/// Address event data.
#[derive(Debug, Clone, Serialize, Deserialize, Getters, PartialEq, Eq, Hash)]
#[getset(get = "pub")]
pub struct AddressData {
    /// The address.
    #[getset(get = "pub")]
    pub address: Bech32Address,
}
