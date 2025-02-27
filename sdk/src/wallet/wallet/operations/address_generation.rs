// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::sync::atomic::Ordering;

#[cfg(all(feature = "events", feature = "ledger_nano"))]
use crate::wallet::events::types::{AddressData, WalletEvent};
use crate::{
    client::secret::{GenerateAddressOptions, SecretManage, SecretManager},
    types::block::address::{Address, Hrp},
    wallet::Wallet,
};

impl Wallet {
    /// Generate an address without storing it
    /// ```ignore
    /// let public_addresses = wallet
    ///     .generate_address(
    ///         0,
    ///         false,
    ///         0,
    ///         None,
    ///     )
    ///     .await?;
    /// ```
    pub async fn generate_address(
        &self,
        account_index: u32,
        address_index: u32,
        options: Option<GenerateAddressOptions>,
    ) -> crate::wallet::Result<Address> {
        let address = match &*self.secret_manager.read().await {
            #[cfg(feature = "ledger_nano")]
            SecretManager::LedgerNano(ledger_nano) => {
                // If we don't sync, then we want to display the prompt on the ledger with the address. But the user
                // needs to have it visible on the computer first, so we need to generate it without the
                // prompt first
                if options.as_ref().map_or(false, |o| o.ledger_nano_prompt) {
                    #[cfg(feature = "events")]
                    {
                        let changed_options = options.map(|mut options| {
                            // Change options so ledger will not show the prompt the first time
                            options.ledger_nano_prompt = false;
                            options
                        });
                        // Generate without prompt to be able to display it
                        let address = ledger_nano
                            .generate_addresses(
                                self.coin_type.load(Ordering::Relaxed),
                                account_index,
                                address_index..address_index + 1,
                                changed_options,
                            )
                            .await?;

                        let bech32_hrp = self.get_bech32_hrp().await?;

                        self.emit(
                            account_index,
                            WalletEvent::LedgerAddressGeneration(AddressData {
                                address: address[0].to_bech32(bech32_hrp),
                            }),
                        )
                        .await;
                    }

                    // Generate with prompt so the user can verify
                    ledger_nano
                        .generate_addresses(
                            self.coin_type.load(Ordering::Relaxed),
                            account_index,
                            address_index..address_index + 1,
                            options,
                        )
                        .await?
                } else {
                    ledger_nano
                        .generate_addresses(
                            self.coin_type.load(Ordering::Relaxed),
                            account_index,
                            address_index..address_index + 1,
                            options,
                        )
                        .await?
                }
            }
            #[cfg(feature = "stronghold")]
            SecretManager::Stronghold(stronghold) => {
                stronghold
                    .generate_addresses(
                        self.coin_type.load(Ordering::Relaxed),
                        account_index,
                        address_index..address_index + 1,
                        options,
                    )
                    .await?
            }
            SecretManager::Mnemonic(mnemonic) => {
                mnemonic
                    .generate_addresses(
                        self.coin_type.load(Ordering::Relaxed),
                        account_index,
                        address_index..address_index + 1,
                        options,
                    )
                    .await?
            }
            SecretManager::Placeholder(_) => return Err(crate::client::Error::PlaceholderSecretManager.into()),
        };

        Ok(*address
            .first()
            .ok_or(crate::wallet::Error::MissingParameter("address"))?)
    }

    /// Get the bech32 hrp from the first account address or if not existent, from the client
    pub async fn get_bech32_hrp(&self) -> crate::wallet::Result<Hrp> {
        Ok(match self.get_accounts().await?.first() {
            Some(account) => {
                account
                    .public_addresses()
                    .await
                    .first()
                    .expect("missing first public address")
                    .address
                    .hrp
            }
            None => self.client().get_bech32_hrp().await?,
        })
    }
}
