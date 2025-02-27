// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! In this example we will create addresses with a ledger nano hardware wallet.
//! To use the ledger nano simulator clone https://github.com/iotaledger/ledger-shimmer-app, run `git submodule init && git submodule update --recursive`,
//! then `./build.sh -m nanos|nanox|nanosplus -s` and use `true` in `LedgerSecretManager::new(true)`.
//!
//! `cargo run --example ledger_nano --release --features=ledger_nano`

use std::time::Instant;

use iota_sdk::{
    client::{
        constants::SHIMMER_COIN_TYPE,
        secret::{ledger_nano::LedgerSecretManager, SecretManager},
    },
    types::block::address::Bech32Address,
    wallet::{ClientOptions, Result, SendAmountParams, Wallet},
};

#[tokio::main]
async fn main() -> Result<()> {
    // This example uses secrets in environment variables for simplicity which should not be done in production.
    dotenvy::dotenv().ok();

    let client_options = ClientOptions::new().with_node(&std::env::var("NODE_URL").unwrap())?;

    let secret_manager = LedgerSecretManager::new(true);

    let wallet = Wallet::builder()
        .with_secret_manager(SecretManager::LedgerNano(secret_manager))
        .with_storage_path("ledger_nano_walletdb")
        .with_client_options(client_options)
        .with_coin_type(SHIMMER_COIN_TYPE)
        .finish()
        .await?;

    println!("{:?}", wallet.get_ledger_nano_status().await?);

    // Get account or create a new one
    let account_alias = "ledger";
    let account = match wallet.get_account(account_alias).await {
        Ok(account) => account,
        _ => {
            // first we'll create an example account and store it
            wallet
                .create_account()
                .with_alias(account_alias.to_string())
                .finish()
                .await?
        }
    };

    let address = account.generate_addresses(1, None).await?;

    println!("{address:?}");

    let now = Instant::now();
    let balance = account.sync(None).await?;
    println!("Syncing took: {:.2?}", now.elapsed());
    println!("Balance: {balance:?}");

    // send transaction
    let outputs = vec![SendAmountParams::new(
        Bech32Address::try_from_str("rms1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluaw60xu")?,
        1_000_000,
    )];
    let transaction = account.send_amount(outputs, None).await?;
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
