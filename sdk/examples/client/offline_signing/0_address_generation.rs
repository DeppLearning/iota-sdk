// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! In this example we generate an address which will be used later to find inputs.
//!
//! `cargo run --example 0_address_generation --release`

use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use iota_sdk::{
    client::{constants::SHIMMER_TESTNET_BECH32_HRP, secret::SecretManager, Client, Result},
    types::block::address::Bech32Address,
};

const ADDRESS_FILE_NAME: &str = "examples/client/offline_signing/address.json";

#[tokio::main]
async fn main() -> Result<()> {
    // This example uses secrets in environment variables for simplicity which should not be done in production.
    dotenvy::dotenv().ok();

    // Creates a client instance.
    let offline_client = Client::builder().finish().await?;
    let secret_manager =
        SecretManager::try_from_mnemonic(&std::env::var("NON_SECURE_USE_OF_DEVELOPMENT_MNEMONIC_1").unwrap())?;

    // Generates an address offline.
    let address = offline_client
        .get_addresses(&secret_manager)
        // Currently only index 0 is supported for offline signing.
        .with_range(0..1)
        .with_bech32_hrp(SHIMMER_TESTNET_BECH32_HRP)
        .finish()
        .await?;

    write_address_to_file(ADDRESS_FILE_NAME, &address)
}

fn write_address_to_file<P: AsRef<Path>>(path: P, address: &[Bech32Address]) -> Result<()> {
    let json = serde_json::to_string_pretty(&address)?;
    let mut file = BufWriter::new(File::create(path).unwrap());

    println!("{json}");

    file.write_all(json.as_bytes()).unwrap();

    Ok(())
}
