// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_sdk::{
    types::block::output::{
        unlock_condition::{AddressUnlockCondition, ExpirationUnlockCondition},
        BasicOutputBuilder, NativeToken, NftId, NftOutputBuilder, UnlockCondition,
    },
    wallet::{
        account::{OutputsToClaim, TransactionOptions},
        MintNativeTokenParams, Result, SendAmountParams, SendNativeTokensParams,
    },
    U256,
};

use crate::wallet::common::{create_accounts_with_funds, make_wallet, setup, tear_down};

#[ignore]
#[tokio::test]
async fn claim_2_basic_micro_outputs() -> Result<()> {
    let storage_path = "test-storage/claim_2_basic_micro_outputs";
    setup(storage_path)?;

    let wallet = make_wallet(storage_path, None, None).await?;

    let accounts = create_accounts_with_funds(&wallet, 2).await?;

    let micro_amount = 1;
    let tx = accounts[1]
        .send_amount(
            vec![
                SendAmountParams::new(*accounts[0].addresses().await?[0].address(), micro_amount),
                SendAmountParams::new(*accounts[0].addresses().await?[0].address(), micro_amount),
            ],
            TransactionOptions {
                allow_micro_amount: true,
                ..Default::default()
            },
        )
        .await?;

    accounts[1]
        .retry_transaction_until_included(&tx.transaction_id, None, None)
        .await?;

    // Claim with account 0
    let balance = accounts[0].sync(None).await.unwrap();
    assert_eq!(balance.potentially_locked_outputs().len(), 2);
    let base_coin_amount_before_claiming = balance.base_coin().available();

    let tx = accounts[0]
        .claim_outputs(
            accounts[0]
                .get_unlockable_outputs_with_additional_unlock_conditions(OutputsToClaim::MicroTransactions)
                .await?,
        )
        .await?;
    accounts[0]
        .retry_transaction_until_included(&tx.transaction_id, None, None)
        .await?;

    let balance = accounts[0].sync(None).await.unwrap();
    assert_eq!(balance.potentially_locked_outputs().len(), 0);
    assert_eq!(
        balance.base_coin().available(),
        base_coin_amount_before_claiming + 2 * micro_amount
    );

    tear_down(storage_path)
}

#[ignore]
#[tokio::test]
async fn claim_1_of_2_basic_outputs() -> Result<()> {
    let storage_path = "test-storage/claim_1_of_2_basic_outputs";
    setup(storage_path)?;

    let wallet = make_wallet(storage_path, None, None).await?;

    let accounts = create_accounts_with_funds(&wallet, 2).await?;

    let amount = 10;
    let tx = accounts[1]
        .send_amount(
            vec![
                SendAmountParams::new(*accounts[0].addresses().await?[0].address(), amount),
                SendAmountParams::new(*accounts[0].addresses().await?[0].address(), 0),
            ],
            TransactionOptions {
                allow_micro_amount: true,
                ..Default::default()
            },
        )
        .await?;

    accounts[1]
        .retry_transaction_until_included(&tx.transaction_id, None, None)
        .await?;

    // Claim with account 0
    let balance = accounts[0].sync(None).await.unwrap();
    assert_eq!(balance.potentially_locked_outputs().len(), 2);
    let base_coin_amount_before_claiming = balance.base_coin().available();

    let tx = accounts[0]
        .claim_outputs(
            accounts[0]
                .get_unlockable_outputs_with_additional_unlock_conditions(OutputsToClaim::Amount)
                .await?,
        )
        .await?;
    accounts[0]
        .retry_transaction_until_included(&tx.transaction_id, None, None)
        .await?;

    let balance = accounts[0].sync(None).await.unwrap();
    assert_eq!(balance.potentially_locked_outputs().len(), 1);
    assert_eq!(
        balance.base_coin().available(),
        base_coin_amount_before_claiming + amount
    );

    tear_down(storage_path)
}

#[ignore]
#[tokio::test]
async fn claim_2_basic_outputs_no_outputs_in_claim_account() -> Result<()> {
    let storage_path = "test-storage/claim_2_basic_outputs_no_outputs_in_claim_account";
    setup(storage_path)?;

    let wallet = make_wallet(storage_path, None, None).await?;

    let account_0 = &create_accounts_with_funds(&wallet, 1).await?[0];
    let account_1 = wallet.create_account().finish().await?;

    let token_supply = account_0.client().get_token_supply().await?;
    let rent_structure = account_0.client().get_rent_structure().await?;
    let expiration_time = account_0.client().get_time_checked().await? + 86400; // 1 Day from now

    let output = BasicOutputBuilder::new_with_minimum_storage_deposit(rent_structure)
        .add_unlock_condition(AddressUnlockCondition::new(
            *account_1.addresses().await?[0].address().as_ref(),
        ))
        .add_unlock_condition(ExpirationUnlockCondition::new(
            *account_0.addresses().await?[0].address().as_ref(),
            expiration_time,
        )?)
        .finish_output(token_supply)?;
    let amount = output.amount();

    let outputs = vec![output; 2];

    let tx = account_0.send(outputs, None).await?;

    account_0
        .retry_transaction_until_included(&tx.transaction_id, None, None)
        .await?;

    // Claim with account 1
    let balance = account_1.sync(None).await.unwrap();
    assert_eq!(balance.potentially_locked_outputs().len(), 2);
    let base_coin_amount_before_claiming = balance.base_coin().available();

    let tx = account_1
        .claim_outputs(
            account_1
                .get_unlockable_outputs_with_additional_unlock_conditions(OutputsToClaim::All)
                .await?,
        )
        .await?;
    account_1
        .retry_transaction_until_included(&tx.transaction_id, None, None)
        .await?;

    let balance = account_1.sync(None).await.unwrap();
    assert_eq!(balance.potentially_locked_outputs().len(), 0);
    assert_eq!(
        balance.base_coin().available(),
        base_coin_amount_before_claiming + 2 * amount
    );

    tear_down(storage_path)
}

#[ignore]
#[tokio::test]
async fn claim_2_native_tokens() -> Result<()> {
    let storage_path = "test-storage/claim_2_native_tokens";
    setup(storage_path)?;

    let wallet = make_wallet(storage_path, None, None).await?;

    let accounts = create_accounts_with_funds(&wallet, 2).await?;

    let native_token_amount = U256::from(100);

    let tx = accounts[1].create_alias_output(None, None).await?;
    accounts[1]
        .retry_transaction_until_included(&tx.transaction_id, None, None)
        .await?;
    accounts[1].sync(None).await?;

    let mint_tx_0 = accounts[1]
        .mint_native_token(
            MintNativeTokenParams {
                alias_id: None,
                circulating_supply: native_token_amount,
                maximum_supply: native_token_amount,
                foundry_metadata: None,
            },
            None,
        )
        .await?;
    accounts[1]
        .retry_transaction_until_included(&mint_tx_0.transaction.transaction_id, None, None)
        .await?;
    accounts[1].sync(None).await?;

    let mint_tx_1 = accounts[1]
        .mint_native_token(
            MintNativeTokenParams {
                alias_id: None,
                circulating_supply: native_token_amount,
                maximum_supply: native_token_amount,
                foundry_metadata: None,
            },
            None,
        )
        .await?;
    accounts[1]
        .retry_transaction_until_included(&mint_tx_1.transaction.transaction_id, None, None)
        .await?;
    accounts[1].sync(None).await?;

    let tx = accounts[1]
        .send_native_tokens(
            vec![
                SendNativeTokensParams {
                    address: *accounts[0].addresses().await?[0].address(),
                    native_tokens: vec![(mint_tx_0.token_id, native_token_amount)],
                    expiration: None,
                    return_address: None,
                },
                SendNativeTokensParams {
                    address: *accounts[0].addresses().await?[0].address(),
                    native_tokens: vec![(mint_tx_1.token_id, native_token_amount)],
                    expiration: None,
                    return_address: None,
                },
            ],
            None,
        )
        .await?;
    accounts[1]
        .retry_transaction_until_included(&tx.transaction_id, None, None)
        .await?;

    // Claim with account 0
    let balance = accounts[0].sync(None).await.unwrap();
    assert_eq!(balance.potentially_locked_outputs().len(), 2);

    let tx = accounts[0]
        .claim_outputs(
            accounts[0]
                .get_unlockable_outputs_with_additional_unlock_conditions(OutputsToClaim::NativeTokens)
                .await?,
        )
        .await?;
    accounts[0]
        .retry_transaction_until_included(&tx.transaction_id, None, None)
        .await?;

    let balance = accounts[0].sync(None).await.unwrap();
    assert_eq!(balance.potentially_locked_outputs().len(), 0);
    assert_eq!(balance.native_tokens().len(), 2);
    let native_token_0 = balance
        .native_tokens()
        .iter()
        .find(|t| t.token_id() == &mint_tx_0.token_id)
        .unwrap();
    assert_eq!(native_token_0.total(), native_token_amount);
    let native_token_1 = balance
        .native_tokens()
        .iter()
        .find(|t| t.token_id() == &mint_tx_1.token_id)
        .unwrap();
    assert_eq!(native_token_1.total(), native_token_amount);

    tear_down(storage_path)
}

#[ignore]
#[tokio::test]
async fn claim_2_native_tokens_no_outputs_in_claim_account() -> Result<()> {
    let storage_path = "test-storage/claim_2_native_tokens_no_outputs_in_claim_account";
    setup(storage_path)?;

    let wallet = make_wallet(storage_path, None, None).await?;

    let account_0 = &create_accounts_with_funds(&wallet, 1).await?[0];
    let account_1 = wallet.create_account().finish().await?;

    let native_token_amount = U256::from(100);

    let tx = account_0.create_alias_output(None, None).await?;
    account_0
        .retry_transaction_until_included(&tx.transaction_id, None, None)
        .await?;
    account_0.sync(None).await?;

    let mint_tx_0 = account_0
        .mint_native_token(
            MintNativeTokenParams {
                alias_id: None,
                circulating_supply: native_token_amount,
                maximum_supply: native_token_amount,
                foundry_metadata: None,
            },
            None,
        )
        .await?;
    account_0
        .retry_transaction_until_included(&mint_tx_0.transaction.transaction_id, None, None)
        .await?;
    account_0.sync(None).await?;

    let mint_tx_1 = account_0
        .mint_native_token(
            MintNativeTokenParams {
                alias_id: None,
                circulating_supply: native_token_amount,
                maximum_supply: native_token_amount,
                foundry_metadata: None,
            },
            None,
        )
        .await?;
    account_0
        .retry_transaction_until_included(&mint_tx_1.transaction.transaction_id, None, None)
        .await?;
    account_0.sync(None).await?;

    let rent_structure = account_0.client().get_rent_structure().await?;
    let token_supply = account_0.client().get_token_supply().await?;

    let tx = account_0
        .send(
            vec![
                BasicOutputBuilder::new_with_minimum_storage_deposit(rent_structure)
                    .add_unlock_condition(AddressUnlockCondition::new(
                        *account_1.addresses().await?[0].address().as_ref(),
                    ))
                    .add_unlock_condition(ExpirationUnlockCondition::new(
                        *account_0.addresses().await?[0].address().as_ref(),
                        account_0.client().get_time_checked().await? + 5000,
                    )?)
                    .add_native_token(NativeToken::new(mint_tx_0.token_id, native_token_amount)?)
                    .finish_output(token_supply)?,
                BasicOutputBuilder::new_with_minimum_storage_deposit(rent_structure)
                    .add_unlock_condition(AddressUnlockCondition::new(
                        *account_1.addresses().await?[0].address().as_ref(),
                    ))
                    .add_unlock_condition(ExpirationUnlockCondition::new(
                        *account_0.addresses().await?[0].address().as_ref(),
                        account_0.client().get_time_checked().await? + 5000,
                    )?)
                    .add_native_token(NativeToken::new(mint_tx_1.token_id, native_token_amount)?)
                    .finish_output(token_supply)?,
            ],
            None,
        )
        .await?;
    account_0
        .retry_transaction_until_included(&tx.transaction_id, None, None)
        .await?;

    // Claim with account 1
    let balance = account_1.sync(None).await.unwrap();
    assert_eq!(balance.potentially_locked_outputs().len(), 2);

    let tx = account_1
        .claim_outputs(
            account_1
                .get_unlockable_outputs_with_additional_unlock_conditions(OutputsToClaim::NativeTokens)
                .await?,
        )
        .await?;
    account_1
        .retry_transaction_until_included(&tx.transaction_id, None, None)
        .await?;

    let balance = account_1.sync(None).await.unwrap();
    assert_eq!(balance.potentially_locked_outputs().len(), 0);
    assert_eq!(balance.native_tokens().len(), 2);
    let native_token_0 = balance
        .native_tokens()
        .iter()
        .find(|t| t.token_id() == &mint_tx_0.token_id)
        .unwrap();
    assert_eq!(native_token_0.total(), native_token_amount);
    let native_token_1 = balance
        .native_tokens()
        .iter()
        .find(|t| t.token_id() == &mint_tx_1.token_id)
        .unwrap();
    assert_eq!(native_token_1.total(), native_token_amount);

    tear_down(storage_path)
}

#[ignore]
#[tokio::test]
async fn claim_2_nft_outputs() -> Result<()> {
    let storage_path = "test-storage/claim_2_nft_outputs";
    setup(storage_path)?;

    let wallet = make_wallet(storage_path, None, None).await?;

    let accounts = create_accounts_with_funds(&wallet, 2).await?;

    let token_supply = accounts[1].client().get_token_supply().await?;
    let outputs = vec![
        // address of the owner of the NFT
        NftOutputBuilder::new_with_amount(1_000_000, NftId::null())
            .with_unlock_conditions(vec![
                UnlockCondition::Address(AddressUnlockCondition::new(
                    *accounts[0].addresses().await?[0].address().as_ref(),
                )),
                UnlockCondition::Expiration(ExpirationUnlockCondition::new(
                    *accounts[1].addresses().await?[0].address().as_ref(),
                    accounts[1].client().get_time_checked().await? + 5000,
                )?),
            ])
            .finish_output(token_supply)?,
        NftOutputBuilder::new_with_amount(1_000_000, NftId::null())
            .with_unlock_conditions(vec![
                UnlockCondition::Address(AddressUnlockCondition::new(
                    *accounts[0].addresses().await?[0].address().as_ref(),
                )),
                UnlockCondition::Expiration(ExpirationUnlockCondition::new(
                    *accounts[1].addresses().await?[0].address().as_ref(),
                    accounts[1].client().get_time_checked().await? + 5000,
                )?),
            ])
            .finish_output(token_supply)?,
    ];

    let tx = accounts[1].send(outputs, None).await?;
    accounts[1]
        .retry_transaction_until_included(&tx.transaction_id, None, None)
        .await?;

    // Claim with account 0
    let balance = accounts[0].sync(None).await.unwrap();
    assert_eq!(balance.potentially_locked_outputs().len(), 2);

    let tx = accounts[0]
        .claim_outputs(
            accounts[0]
                .get_unlockable_outputs_with_additional_unlock_conditions(OutputsToClaim::Nfts)
                .await?,
        )
        .await?;
    accounts[0]
        .retry_transaction_until_included(&tx.transaction_id, None, None)
        .await?;

    let balance = accounts[0].sync(None).await.unwrap();
    assert_eq!(balance.potentially_locked_outputs().len(), 0);
    assert_eq!(balance.nfts().len(), 2);

    tear_down(storage_path)
}

#[ignore]
#[tokio::test]
async fn claim_2_nft_outputs_no_outputs_in_claim_account() -> Result<()> {
    let storage_path = "test-storage/claim_2_nft_outputs_no_outputs_in_claim_account";
    setup(storage_path)?;

    let wallet = make_wallet(storage_path, None, None).await?;

    let account_0 = &create_accounts_with_funds(&wallet, 1).await?[0];
    let account_1 = wallet.create_account().finish().await?;

    let token_supply = account_0.client().get_token_supply().await?;
    let outputs = vec![
        // address of the owner of the NFT
        NftOutputBuilder::new_with_amount(1_000_000, NftId::null())
            .with_unlock_conditions(vec![
                UnlockCondition::Address(AddressUnlockCondition::new(
                    *account_1.addresses().await?[0].address().as_ref(),
                )),
                UnlockCondition::Expiration(ExpirationUnlockCondition::new(
                    *account_0.addresses().await?[0].address().as_ref(),
                    account_0.client().get_time_checked().await? + 5000,
                )?),
            ])
            .finish_output(token_supply)?,
        NftOutputBuilder::new_with_amount(1_000_000, NftId::null())
            .with_unlock_conditions(vec![
                UnlockCondition::Address(AddressUnlockCondition::new(
                    *account_1.addresses().await?[0].address().as_ref(),
                )),
                UnlockCondition::Expiration(ExpirationUnlockCondition::new(
                    *account_0.addresses().await?[0].address().as_ref(),
                    account_0.client().get_time_checked().await? + 5000,
                )?),
            ])
            .finish_output(token_supply)?,
    ];

    let tx = account_0.send(outputs, None).await?;
    account_0
        .retry_transaction_until_included(&tx.transaction_id, None, None)
        .await?;

    // Claim with account 1
    let balance = account_1.sync(None).await.unwrap();
    assert_eq!(balance.potentially_locked_outputs().len(), 2);

    let tx = account_1
        .claim_outputs(
            account_1
                .get_unlockable_outputs_with_additional_unlock_conditions(OutputsToClaim::Nfts)
                .await?,
        )
        .await?;
    account_1
        .retry_transaction_until_included(&tx.transaction_id, None, None)
        .await?;

    let balance = account_1.sync(None).await.unwrap();
    assert_eq!(balance.potentially_locked_outputs().len(), 0);
    assert_eq!(balance.nfts().len(), 2);

    tear_down(storage_path)
}
