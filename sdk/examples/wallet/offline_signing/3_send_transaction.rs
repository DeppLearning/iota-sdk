// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! In this example we send the signed transaction in a block.
//!
//! `cargo run --example 3_send_transaction --release`.

use std::{fs::File, io::prelude::*, path::Path};

use iota_sdk::{
    client::{
        api::{SignedTransactionData, SignedTransactionDataDto},
        Client,
    },
    wallet::{Result, Wallet},
};

const SIGNED_TRANSACTION_FILE_NAME: &str = "examples/wallet/offline_signing/signed_transaction.json";

#[tokio::main]
async fn main() -> Result<()> {
    // Create the wallet with the secret_manager and client options
    let wallet = Wallet::builder()
        .with_storage_path("examples/wallet/offline_signing/online_walletdb")
        .finish()
        .await?;

    // Create a new account
    let account = wallet.get_account("Alice").await?;

    let signed_transaction_data =
        read_signed_transaction_from_file(account.client(), SIGNED_TRANSACTION_FILE_NAME).await?;

    // Sends offline signed transaction online.
    let transaction = account.submit_and_store_transaction(signed_transaction_data).await?;
    println!("Transaction sent: {}", transaction.transaction_id);

    let block_id = account
        .retry_transaction_until_included(&transaction.transaction_id, None, None)
        .await?;
    println!(
        "Block included: {}/block/{}",
        std::env::var("EXPLORER_URL").unwrap(),
        block_id
    );

    Ok(())
}

async fn read_signed_transaction_from_file<P: AsRef<Path> + Send>(
    client: &Client,
    path: P,
) -> Result<SignedTransactionData> {
    let mut file = File::open(&path)?;
    let mut json = String::new();
    file.read_to_string(&mut json)?;

    let dto = serde_json::from_str::<SignedTransactionDataDto>(&json)?;

    Ok(SignedTransactionData::try_from_dto(
        &dto,
        &client.get_protocol_parameters().await?,
    )?)
}
