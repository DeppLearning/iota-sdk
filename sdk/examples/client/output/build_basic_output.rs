// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! In this example we will build basic outputs with different feature blocks.
//!
//! `cargo run --example build_basic_output --release`

use iota_sdk::{
    client::{Client, Result},
    types::block::{
        address::Address,
        output::{
            feature::{MetadataFeature, SenderFeature, TagFeature},
            unlock_condition::{
                AddressUnlockCondition, ExpirationUnlockCondition, StorageDepositReturnUnlockCondition,
                TimelockUnlockCondition,
            },
            BasicOutputBuilder,
        },
    },
};

#[tokio::main]
async fn main() -> Result<()> {
    // This example uses secrets in environment variables for simplicity which should not be done in production.
    dotenvy::dotenv().ok();

    let node_url = std::env::var("NODE_URL").unwrap();

    // Create a client instance.
    let client = Client::builder().with_node(&node_url)?.finish().await?;

    let token_supply = client.get_token_supply().await?;

    let address = Address::try_from_bech32("rms1qpllaj0pyveqfkwxmnngz2c488hfdtmfrj3wfkgxtk4gtyrax0jaxzt70zy")?;

    let basic_output_builder =
        BasicOutputBuilder::new_with_amount(1_000_000).add_unlock_condition(AddressUnlockCondition::new(address));

    let outputs = vec![
        // most simple output
        basic_output_builder.clone().finish_output(token_supply)?,
        // with metadata feature block
        basic_output_builder
            .clone()
            .add_feature(MetadataFeature::new("Hello, World!")?)
            .finish_output(token_supply)?,
        // with storage deposit return
        basic_output_builder
            .clone()
            .add_unlock_condition(StorageDepositReturnUnlockCondition::new(
                address,
                1000000,
                token_supply,
            )?)
            .finish_output(token_supply)?,
        // with expiration
        basic_output_builder
            .clone()
            .add_unlock_condition(ExpirationUnlockCondition::new(address, 1)?)
            .finish_output(token_supply)?,
        // with timelock
        basic_output_builder
            .clone()
            .add_unlock_condition(TimelockUnlockCondition::new(1)?)
            .finish_output(token_supply)?,
        // with tag feature
        basic_output_builder
            .clone()
            .add_feature(TagFeature::new("Hello, World!")?)
            .finish_output(token_supply)?,
        // with sender feature
        basic_output_builder
            .add_feature(SenderFeature::new(address))
            .finish_output(token_supply)?,
    ];

    println!("{outputs:#?}");

    Ok(())
}
