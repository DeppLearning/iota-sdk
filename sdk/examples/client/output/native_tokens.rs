// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! In this example we will send basic outputs with native tokens in two ways:
//! 1. receiver gets the full output amount + native tokens
//! 2. receiver needs to claim the output to get the native tokens, but has to send the amount back
//!
//! `cargo run --example native_tokens --release`

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use iota_sdk::{
    client::{secret::SecretManager, utils::request_funds_from_faucet, Client, Result},
    types::block::output::{
        unlock_condition::{AddressUnlockCondition, ExpirationUnlockCondition, StorageDepositReturnUnlockCondition},
        BasicOutputBuilder, NativeToken, TokenId,
    },
};
use primitive_types::U256;

#[tokio::main]
async fn main() -> Result<()> {
    // This example uses secrets in environment variables for simplicity which should not be done in production.
    // Configure your own mnemonic in the ".env" file. Since the output amount cannot be zero, the seed must contain
    // non-zero balance.
    dotenvy::dotenv().ok();

    let node_url = std::env::var("NODE_URL").unwrap();
    let explorer_url = std::env::var("EXPLORER_URL").unwrap();
    let faucet_url = std::env::var("FAUCET_URL").unwrap();

    // Create a client instance.
    let client = Client::builder().with_node(&node_url)?.finish().await?;

    let secret_manager =
        SecretManager::try_from_mnemonic(&std::env::var("NON_SECURE_USE_OF_DEVELOPMENT_MNEMONIC_1").unwrap())?;

    let addresses = client.get_addresses(&secret_manager).with_range(0..2).finish().await?;
    let sender_address = addresses[0];
    let receiver_address = addresses[1];

    let token_supply = client.get_token_supply().await?;

    request_funds_from_faucet(&faucet_url, &sender_address).await?;
    tokio::time::sleep(std::time::Duration::from_secs(15)).await;

    let tomorrow = (SystemTime::now() + Duration::from_secs(24 * 3600))
        .duration_since(UNIX_EPOCH)
        .expect("clock went backwards")
        .as_secs()
        .try_into()
        .unwrap();

    // Replace with the token ID of native tokens you own.
    let token_id: [u8; 38] =
        prefix_hex::decode("0x08e68f7616cd4948efebc6a77c4f935eaed770ac53869cba56d104f2b472a8836d0100000000")?;

    let outputs = vec![
        // Without StorageDepositReturnUnlockCondition, the receiver will get the amount of the output and the native
        // tokens
        BasicOutputBuilder::new_with_amount(1_000_000)
            .add_unlock_condition(AddressUnlockCondition::new(receiver_address))
            .add_native_token(NativeToken::new(TokenId::new(token_id), U256::from(10))?)
            .finish_output(token_supply)?,
        // With StorageDepositReturnUnlockCondition, the receiver can consume the output to get the native tokens, but
        // he needs to send the amount back
        BasicOutputBuilder::new_with_amount(1_000_000)
            .add_unlock_condition(AddressUnlockCondition::new(receiver_address))
            .add_native_token(NativeToken::new(TokenId::new(token_id), U256::from(10))?)
            // Return the full amount.
            .add_unlock_condition(StorageDepositReturnUnlockCondition::new(
                sender_address,
                1_000_000,
                token_supply,
            )?)
            // If the receiver does not consume this output, we unlock after a day to avoid
            // locking our funds forever.
            .add_unlock_condition(ExpirationUnlockCondition::new(sender_address, tomorrow)?)
            .finish_output(token_supply)?,
    ];

    let block = client
        .block()
        .with_secret_manager(&secret_manager)
        .with_outputs(outputs)?
        .finish()
        .await?;

    println!("Block with native tokens sent: {explorer_url}/block/{}", block.id());

    Ok(())
}
