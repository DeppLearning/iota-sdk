// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

use crate::{
    client::api::PreparedTransactionData,
    types::block::{
        address::Bech32Address,
        output::{
            feature::MetadataFeature,
            unlock_condition::{GovernorAddressUnlockCondition, StateControllerAddressUnlockCondition},
            AliasId, AliasOutputBuilder, Output,
        },
        Error,
    },
    wallet::account::{types::Transaction, Account, OutputData, TransactionOptions},
};

/// Params `create_alias_output()`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAliasParams {
    /// Bech32 encoded address which will control the alias. Default will use the first
    /// address of the account
    pub address: Option<Bech32Address>,
    /// Immutable alias metadata
    pub immutable_metadata: Option<Vec<u8>>,
    /// Alias metadata
    pub metadata: Option<Vec<u8>>,
    /// Alias state metadata
    pub state_metadata: Option<Vec<u8>>,
}

/// Dto for aliasOptions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAliasParamsDto {
    /// Bech32 encoded address which will control the alias. Default will use the first
    /// address of the account
    pub address: Option<Bech32Address>,
    /// Immutable alias metadata, hex encoded bytes
    pub immutable_metadata: Option<String>,
    /// Alias metadata, hex encoded bytes
    pub metadata: Option<String>,
    /// Alias state metadata
    pub state_metadata: Option<String>,
}

impl TryFrom<&CreateAliasParamsDto> for CreateAliasParams {
    type Error = crate::wallet::Error;

    fn try_from(value: &CreateAliasParamsDto) -> crate::wallet::Result<Self> {
        Ok(Self {
            address: value.address,
            immutable_metadata: match &value.immutable_metadata {
                Some(metadata) => {
                    Some(prefix_hex::decode(metadata).map_err(|_| Error::InvalidField("immutable_metadata"))?)
                }
                None => None,
            },
            metadata: match &value.metadata {
                Some(metadata) => Some(prefix_hex::decode(metadata).map_err(|_| Error::InvalidField("metadata"))?),
                None => None,
            },
            state_metadata: match &value.state_metadata {
                Some(metadata) => {
                    Some(prefix_hex::decode(metadata).map_err(|_| Error::InvalidField("state_metadata"))?)
                }
                None => None,
            },
        })
    }
}

impl Account {
    /// Function to create an alias output.
    /// ```ignore
    /// let params = CreateAliasParams {
    ///     address: None,
    ///     immutable_metadata: Some(b"some immutable alias metadata".to_vec()),
    ///     metadata: Some(b"some alias metadata".to_vec()),
    ///     state_metadata: Some(b"some alias state metadata".to_vec()),
    /// };
    ///
    /// let transaction = account.create_alias_output(params, None).await?;
    /// println!(
    ///     "Transaction sent: {}/transaction/{}",
    ///     std::env::var("EXPLORER_URL").unwrap(),
    ///     transaction.transaction_id
    /// );
    /// ```
    pub async fn create_alias_output(
        &self,
        params: Option<CreateAliasParams>,
        options: impl Into<Option<TransactionOptions>> + Send,
    ) -> crate::wallet::Result<Transaction> {
        let prepared_transaction = self.prepare_create_alias_output(params, options).await?;
        self.sign_and_submit_transaction(prepared_transaction).await
    }

    /// Function to prepare the transaction for
    /// [Account.create_alias_output()](crate::account::Account.create_alias_output)
    pub async fn prepare_create_alias_output(
        &self,
        params: Option<CreateAliasParams>,
        options: impl Into<Option<TransactionOptions>> + Send,
    ) -> crate::wallet::Result<PreparedTransactionData> {
        log::debug!("[TRANSACTION] prepare_create_alias_output");
        let rent_structure = self.client().get_rent_structure().await?;
        let token_supply = self.client().get_token_supply().await?;

        let controller_address = match params.as_ref().and_then(|options| options.address.as_ref()) {
            Some(bech32_address) => {
                self.client().bech32_hrp_matches(bech32_address.hrp()).await?;
                *bech32_address.inner()
            }
            None => {
                self.public_addresses()
                    .await
                    .first()
                    .expect("first address is generated during account creation")
                    .address
                    .inner
            }
        };

        let mut alias_output_builder =
            AliasOutputBuilder::new_with_minimum_storage_deposit(rent_structure, AliasId::null())
                .with_state_index(0)
                .with_foundry_counter(0)
                .add_unlock_condition(StateControllerAddressUnlockCondition::new(controller_address))
                .add_unlock_condition(GovernorAddressUnlockCondition::new(controller_address));
        if let Some(CreateAliasParams {
            immutable_metadata,
            metadata,
            state_metadata,
            ..
        }) = params
        {
            if let Some(immutable_metadata) = immutable_metadata {
                alias_output_builder =
                    alias_output_builder.add_immutable_feature(MetadataFeature::new(immutable_metadata)?);
            }
            if let Some(metadata) = metadata {
                alias_output_builder = alias_output_builder.add_feature(MetadataFeature::new(metadata)?);
            }
            if let Some(state_metadata) = state_metadata {
                alias_output_builder = alias_output_builder.with_state_metadata(state_metadata);
            }
        }

        let outputs = vec![alias_output_builder.finish_output(token_supply)?];

        self.prepare_transaction(outputs, options).await
    }

    /// Get an existing alias output
    pub(crate) async fn get_alias_output(&self, alias_id: Option<AliasId>) -> Option<(AliasId, OutputData)> {
        log::debug!("[get_alias_output]");
        self.details()
            .await
            .unspent_outputs()
            .values()
            .find_map(|output_data| match &output_data.output {
                Output::Alias(alias_output) => {
                    let output_alias_id = alias_output.alias_id_non_null(&output_data.output_id);

                    alias_id.map_or_else(
                        || Some((output_alias_id, output_data.clone())),
                        |alias_id| {
                            if output_alias_id == alias_id {
                                Some((output_alias_id, output_data.clone()))
                            } else {
                                None
                            }
                        },
                    )
                }
                _ => None,
            })
    }
}
