// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "ledger_nano")]
use crate::client::secret::SecretManager;
use crate::{
    client::api::PreparedTransactionData,
    types::block::{
        input::INPUT_COUNT_MAX,
        output::{
            unlock_condition::AddressUnlockCondition, BasicOutputBuilder, NativeTokens, NativeTokensBuilder, Output,
        },
    },
};

// Constants for the calculation of the amount of inputs we can use with a ledger nano
#[cfg(feature = "ledger_nano")]
const ESSENCE_SIZE_WITHOUT_IN_AND_OUTPUTS: usize = 49;
#[cfg(feature = "ledger_nano")]
// Input size in essence (35) + LedgerBIP32Index (8)
const INPUT_SIZE: usize = 43;
#[cfg(feature = "ledger_nano")]
const MIN_OUTPUT_SIZE_IN_ESSENCE: usize = 46;

#[cfg(feature = "ledger_nano")]
use crate::wallet::account::constants::DEFAULT_LEDGER_OUTPUT_CONSOLIDATION_THRESHOLD;
use crate::wallet::{
    account::{
        constants::DEFAULT_OUTPUT_CONSOLIDATION_THRESHOLD,
        operations::{helpers::time::can_output_be_unlocked_now, output_claiming::get_new_native_token_count},
        types::{OutputData, Transaction},
        Account, AddressWithUnspentOutputs, TransactionOptions,
    },
    Result,
};

impl Account {
    fn should_consolidate_output(
        &self,
        output_data: &OutputData,
        current_time: u32,
        account_addresses: &[AddressWithUnspentOutputs],
    ) -> Result<bool> {
        Ok(if let Output::Basic(basic_output) = &output_data.output {
            let unlock_conditions = basic_output.unlock_conditions();

            let is_time_locked = unlock_conditions.is_time_locked(current_time);
            if is_time_locked {
                // If the output is timelocked, then it cannot be consolidated.
                return Ok(false);
            }

            let has_storage_deposit_return = unlock_conditions.storage_deposit_return().is_some();
            let has_expiration = unlock_conditions.expiration().is_some();
            let is_expired = unlock_conditions.is_expired(current_time);
            if has_storage_deposit_return && (!has_expiration || !is_expired) {
                // If the output has not expired and must return a storage deposit, then it cannot be consolidated.
                return Ok(false);
            }

            can_output_be_unlocked_now(account_addresses, &[], output_data, current_time, None)?
        } else {
            false
        })
    }

    /// Consolidate basic outputs with only an [AddressUnlockCondition] from an account by sending them to an own
    /// address again if the output amount is >= the output_consolidation_threshold. When `force` is set to `true`, the
    /// threshold is ignored. Only consolidates the amount of outputs that fit into a single transaction.
    pub async fn consolidate_outputs(
        &self,
        force: bool,
        output_consolidation_threshold: Option<usize>,
    ) -> Result<Transaction> {
        let prepared_transaction = self
            .prepare_consolidate_outputs(force, output_consolidation_threshold)
            .await?;
        let consolidation_tx = self.sign_and_submit_transaction(prepared_transaction).await?;

        log::debug!(
            "[OUTPUT_CONSOLIDATION] consolidation transaction created: block_id: {:?} tx_id: {:?}",
            consolidation_tx.block_id,
            consolidation_tx.transaction_id
        );

        Ok(consolidation_tx)
    }

    /// Function to prepare the transaction for
    /// [Account.consolidate_outputs()](crate::account::Account.consolidate_outputs)
    pub async fn prepare_consolidate_outputs(
        &self,
        force: bool,
        output_consolidation_threshold: Option<usize>,
    ) -> Result<PreparedTransactionData> {
        log::debug!("[OUTPUT_CONSOLIDATION] prepare consolidating outputs if needed");
        #[cfg(feature = "participation")]
        let voting_output = self.get_voting_output().await?;
        let current_time = self.client().get_time_checked().await?;
        let token_supply = self.client().get_token_supply().await?;
        let mut outputs_to_consolidate = Vec::new();
        let account_details = self.details().await;
        let account_addresses = &account_details.addresses_with_unspent_outputs[..];

        for (output_id, output_data) in account_details.unspent_outputs() {
            #[cfg(feature = "participation")]
            if let Some(ref voting_output) = voting_output {
                // Remove voting output from inputs, because we want to keep its features and not consolidate it.
                if output_data.output_id == voting_output.output_id {
                    continue;
                }
            }
            let is_locked_output = account_details.locked_outputs.contains(output_id);
            let should_consolidate_output =
                self.should_consolidate_output(output_data, current_time, account_addresses)?;
            if !is_locked_output && should_consolidate_output {
                outputs_to_consolidate.push(output_data.clone());
            }
        }

        drop(account_details);

        let output_consolidation_threshold = output_consolidation_threshold.unwrap_or({
            match &*self.wallet.secret_manager.read().await {
                #[cfg(feature = "ledger_nano")]
                SecretManager::LedgerNano(_) => DEFAULT_LEDGER_OUTPUT_CONSOLIDATION_THRESHOLD,
                _ => DEFAULT_OUTPUT_CONSOLIDATION_THRESHOLD,
            }
        });

        // only consolidate if the unlocked outputs are >= output_consolidation_threshold
        if outputs_to_consolidate.is_empty()
            || (!force && outputs_to_consolidate.len() < output_consolidation_threshold)
        {
            log::debug!(
                "[OUTPUT_CONSOLIDATION] no consolidation needed, available_outputs: {}, consolidation_threshold: {}",
                outputs_to_consolidate.len(),
                output_consolidation_threshold
            );
            return Err(crate::wallet::Error::NoOutputsToConsolidate {
                available_outputs: outputs_to_consolidate.len(),
                consolidation_threshold: output_consolidation_threshold,
            });
        }

        let max_inputs = match &*self.wallet.secret_manager.read().await {
            #[cfg(feature = "ledger_nano")]
            SecretManager::LedgerNano(ledger) => {
                let ledger_nano_status = ledger.get_ledger_nano_status().await;
                // With blind signing we are only limited by the protocol
                if ledger_nano_status.blind_signing_enabled() {
                    INPUT_COUNT_MAX
                } else {
                    ledger_nano_status
                        .buffer_size()
                        .map(|buffer_size| {
                            // Calculate how many inputs we can have with this ledger, buffer size is different for
                            // different ledger types
                            let available_buffer_size_for_inputs =
                                buffer_size - ESSENCE_SIZE_WITHOUT_IN_AND_OUTPUTS - MIN_OUTPUT_SIZE_IN_ESSENCE;
                            (available_buffer_size_for_inputs / INPUT_SIZE) as u16
                        })
                        .unwrap_or(INPUT_COUNT_MAX)
                }
            }
            _ => INPUT_COUNT_MAX,
        };

        let mut total_amount = 0;
        let mut custom_inputs = Vec::with_capacity(max_inputs.into());
        let mut total_native_tokens = NativeTokensBuilder::new();

        for output_data in outputs_to_consolidate.iter().take(max_inputs.into()) {
            if let Some(native_tokens) = output_data.output.native_tokens() {
                // Skip output if the max native tokens count would be exceeded
                if get_new_native_token_count(&total_native_tokens, native_tokens)? > NativeTokens::COUNT_MAX.into() {
                    log::debug!("[OUTPUT_CONSOLIDATION] skipping output to not exceed the max native tokens count");
                    continue;
                }
                total_native_tokens.add_native_tokens(native_tokens.clone())?;
            };
            total_amount += output_data.output.amount();

            custom_inputs.push(output_data.output_id);
        }

        let consolidation_output = vec![
            BasicOutputBuilder::new_with_amount(total_amount)
                .add_unlock_condition(AddressUnlockCondition::new(outputs_to_consolidate[0].address))
                .with_native_tokens(total_native_tokens.finish()?)
                .finish_output(token_supply)?,
        ];

        let options = Some(TransactionOptions {
            custom_inputs: Some(custom_inputs),
            ..Default::default()
        });

        self.prepare_transaction(consolidation_output, options).await
    }
}
