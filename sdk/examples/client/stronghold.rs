// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! In this example we will create an address with a stronghold secret manager.
//!
//! `cargo run --example stronghold --features=stronghold --release`


use async_trait::async_trait;
use iota_sdk::{client::{
    api::{GetAddressesBuilder, PreparedTransactionData},
    constants::{SHIMMER_COIN_TYPE, SHIMMER_TESTNET_BECH32_HRP},
    secret::{stronghold::StrongholdSecretManager, SecretManager, SecretM, SignTransactionEssence, SecretManage, GenerateAddressOptions},
    Result, stronghold::StrongholdAdapter,
}, types::block::{unlock::{Unlocks, Unlock, SignatureUnlock}, address::Address, signature::{Ed25519Signature, Signature}}};
use core::ops::Range;

pub struct GenericSecretManager(StrongholdAdapter);

use crypto::keys::slip10::Chain;

#[async_trait]
impl SecretManage for GenericSecretManager {
    type Error = iota_sdk::client::Error;

    /// Generates addresses.
    ///
    /// For `coin_type`, see also <https://github.com/satoshilabs/slips/blob/master/slip-0044.md>.
    async fn generate_addresses(
        &self,
        coin_type: u32,
        account_index: u32,
        address_indexes: Range<u32>,
        options: Option<GenerateAddressOptions>,
    ) -> core::result::Result<Vec<Address>, Self::Error> {
        Ok(self.0.generate_addresses(coin_type, account_index, address_indexes, options).await?)
    }

    /// Signs `msg` using the given [`Chain`].
    async fn sign_ed25519(&self, msg: &[u8], chain: &Chain) -> core::result::Result<Ed25519Signature, Self::Error> {
        Ok(self.0.sign_ed25519(msg, chain).await?)
    }

    /// Signs `essence_hash` using the given `chain`, returning an [`Unlock`].
    async fn signature_unlock(&self, essence_hash: &[u8; 32], chain: &Chain) -> core::result::Result<Unlock, Self::Error> {
        Ok(Unlock::Signature(SignatureUnlock::new(Signature::Ed25519(
            self.sign_ed25519(essence_hash, chain).await?,
        ))))
    }
    
}
impl SecretM for GenericSecretManager {}

#[async_trait]
impl SignTransactionEssence for GenericSecretManager {
    async fn sign_transaction_essence(
        &self,
        prepared_transaction_data: &PreparedTransactionData,
        time: Option<u32>,
    ) -> core::result::Result<Unlocks, <Self as SecretManage>::Error> {
        // Ok(SecretManager::Stronghold(self.0).sign_transaction_essence(prepared_transaction_data, time).await?)
        todo!()
    }
}
#[tokio::main]
async fn main() -> Result<()> {
    let stronghold_secret_manager = StrongholdSecretManager::builder()
        .password("some_hopefully_secure_password")
        .build("test.stronghold")?;

    // This example uses secrets in environment variables for simplicity which should not be done in production.
    dotenvy::dotenv().ok();
    // let mnemonic = std::env::var("NON_SECURE_USE_OF_DEVELOPMENT_MNEMONIC_1").unwrap();
    // The mnemonic only needs to be stored the first time
    stronghold_secret_manager.store_mnemonic("endorse answer radar about source reunion marriage tag sausage weekend frost daring base attack because joke dream slender leisure group reason prepare broken river".to_string()).await?;

    // Generate addresses with custom account index and range
    let addresses = GetAddressesBuilder::new(&SecretManager::Generic(Box::new(GenericSecretManager(stronghold_secret_manager))))
        .with_bech32_hrp(SHIMMER_TESTNET_BECH32_HRP)
        .with_coin_type(SHIMMER_COIN_TYPE)
        .with_account_index(0)
        .with_range(0..1)
        .finish()
        .await?;

    println!("First public address: {}", addresses[0]);

    Ok(())
}
