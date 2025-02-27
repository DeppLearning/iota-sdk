// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! In this example we will get inputs and prepare a transaction.
//!
//! `cargo run --example 1_prepare_transaction --release`

use std::{
    fs::File,
    io::{BufWriter, Read, Write},
    path::Path,
};

use iota_sdk::{
    client::{
        api::{PreparedTransactionData, PreparedTransactionDataDto},
        constants::SHIMMER_COIN_TYPE,
        secret::{placeholder::PlaceholderSecretManager, SecretManager},
    },
    types::block::address::Bech32Address,
    wallet::{account::types::AccountAddress, ClientOptions, Result, SendAmountParams, Wallet},
};

const ADDRESS_FILE_NAME: &str = "examples/wallet/offline_signing/addresses.json";
const PREPARED_TRANSACTION_FILE_NAME: &str = "examples/wallet/offline_signing/prepared_transaction.json";

#[tokio::main]
async fn main() -> Result<()> {
    // This example uses secrets in environment variables for simplicity which should not be done in production.
    dotenvy::dotenv().ok();

    let outputs = vec![SendAmountParams::new(
        // Address to which we want to send the amount.
        Bech32Address::try_from_str("rms1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluaw60xu")?,
        // The amount to send.
        1_000_000,
    )];

    // Recovers addresses from example `0_address_generation`.
    let addresses = read_addresses_from_file(ADDRESS_FILE_NAME)?;

    let client_options = ClientOptions::new().with_node(&std::env::var("NODE_URL").unwrap())?;

    // Create the wallet with the secret_manager and client options
    let wallet = Wallet::builder()
        .with_secret_manager(SecretManager::Placeholder(PlaceholderSecretManager))
        .with_client_options(client_options.clone())
        .with_coin_type(SHIMMER_COIN_TYPE)
        .with_storage_path("examples/wallet/offline_signing/online_walletdb")
        .finish()
        .await?;

    // Create a new account
    let account = wallet
        .create_account()
        .with_alias("Alice".to_string())
        .with_addresses(addresses)
        .finish()
        .await?;

    // Sync the account to get the outputs for the addresses
    account.sync(None).await?;

    let prepared_transaction = account.prepare_send_amount(outputs.clone(), None).await?;

    println!("Prepared transaction sending {outputs:?}");

    write_transaction_to_file(PREPARED_TRANSACTION_FILE_NAME, prepared_transaction)
}

fn read_addresses_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<AccountAddress>> {
    let mut file = File::open(&path)?;
    let mut json = String::new();
    file.read_to_string(&mut json)?;

    Ok(serde_json::from_str(&json)?)
}

fn write_transaction_to_file<P: AsRef<Path>>(path: P, prepared_transaction: PreparedTransactionData) -> Result<()> {
    let json = serde_json::to_string_pretty(&PreparedTransactionDataDto::from(&prepared_transaction))?;
    let mut file = BufWriter::new(File::create(path)?);

    println!("{json}");

    file.write_all(json.as_bytes())?;

    Ok(())
}
