// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! In this example we will try to send transactions from multiple threads simultaneously to the first 300 addresses of
//! the first account (ping_account).
//!
//! `cargo run --example pong --release`

use iota_sdk::{
    client::{
        constants::SHIMMER_COIN_TYPE,
        request_funds_from_faucet,
        secret::{mnemonic::MnemonicSecretManager, SecretManager},
    },
    types::block::output::{unlock_condition::AddressUnlockCondition, BasicOutputBuilder},
    wallet::{ClientOptions, Result, Wallet},
};

#[tokio::main]
async fn main() -> Result<()> {
    // This example uses secrets in environment variables for simplicity which should not be done in production.
    dotenvy::dotenv().ok();

    let client_options = ClientOptions::new().with_node(&std::env::var("NODE_URL").unwrap())?;

    let secret_manager =
        MnemonicSecretManager::try_from_mnemonic(&std::env::var("NON_SECURE_USE_OF_DEVELOPMENT_MNEMONIC_1").unwrap())?;

    let wallet = Wallet::builder()
        .with_secret_manager(SecretManager::Mnemonic(secret_manager))
        .with_client_options(client_options)
        .with_coin_type(SHIMMER_COIN_TYPE)
        .with_storage_path("pongdb")
        .finish()
        .await?;

    // Get account or create a new one
    let account_alias = "ping";
    let ping_account = match wallet.get_account(account_alias.to_string()).await {
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
    let account_alias = "pong";
    let pong_account = match wallet.get_account(account_alias.to_string()).await {
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

    let amount_addresses = 5;
    // generate addresses so we find all funds
    if pong_account.addresses().await?.len() < amount_addresses {
        pong_account
            .generate_addresses((amount_addresses - pong_account.addresses().await?.len()) as u32, None)
            .await?;
    }
    let balance = ping_account.sync(None).await?;
    println!("Balance: {balance:?}");
    // generate addresses from the second account to which we will send funds
    let ping_addresses = {
        let mut addresses = ping_account.addresses().await?;
        if addresses.len() < amount_addresses {
            addresses = ping_account
                .generate_addresses((amount_addresses - addresses.len()) as u32, None)
                .await?
        };
        println!(
            "{}",
            request_funds_from_faucet(&std::env::var("FAUCET_URL").unwrap(), addresses[0].address()).await?
        );
        addresses
    };

    for address_index in 0..1000 {
        let mut threads = Vec::new();
        for n in 1..4 {
            let pong_account_ = pong_account.clone();
            let ping_addresses_ = ping_addresses.clone();
            threads.push(async move {
                tokio::spawn(async move {
                    // send transaction
                    let outputs = vec![
                        // send one or two Mi for more different transactions
                        BasicOutputBuilder::new_with_amount(n * 1_000_000)
                            .add_unlock_condition(AddressUnlockCondition::new(
                                ping_addresses_[address_index % amount_addresses].address(),
                            ))
                            .finish_output(pong_account_.client().get_token_supply().await?)?,
                    ];
                    let tx = pong_account_.send(outputs, None).await?;
                    println!(
                        "Block from thread {} sent: {}/block/{}",
                        n,
                        std::env::var("EXPLORER_URL").unwrap(),
                        tx.block_id.expect("no block created yet")
                    );
                    iota_sdk::wallet::Result::Ok(n)
                })
                .await
            });
        }

        let results = futures::future::try_join_all(threads).await?;
        for thread in results {
            if let Err(e) = thread {
                println!("{e}");
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    // wait until user press enter so background tasks keep running
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    Ok(())
}
