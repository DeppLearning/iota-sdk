// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use crypto::keys::slip10::Chain;
use iota_sdk::{
    client::{
        api::{transaction::validate_transaction_payload_length, verify_semantic, PreparedTransactionData},
        constants::{HD_WALLET_TYPE, SHIMMER_COIN_TYPE, SHIMMER_TESTNET_BECH32_HRP},
        secret::{SecretManage, SecretManager, SignTransactionEssence},
        Client, Result,
    },
    types::block::{
        address::{Address, NftAddress},
        input::{Input, UtxoInput},
        output::{InputsCommitment, NftId},
        payload::{
            transaction::{RegularTransactionEssence, TransactionEssence},
            TransactionPayload,
        },
        protocol::protocol_parameters,
        semantic::ConflictReason,
        unlock::{SignatureUnlock, Unlock},
    },
};

use crate::client::{
    build_inputs, build_outputs,
    Build::{Basic, Nft},
    NFT_ID_1,
};

#[tokio::test]
async fn nft_reference_unlocks() -> Result<()> {
    let secret_manager = SecretManager::try_from_mnemonic(&Client::generate_mnemonic()?)?;

    let bech32_address_0 = &secret_manager
        .generate_addresses(SHIMMER_COIN_TYPE, 0, 0..1, None)
        .await?[0]
        .to_bech32(SHIMMER_TESTNET_BECH32_HRP);

    let protocol_parameters = protocol_parameters();
    let nft_id = NftId::from_str(NFT_ID_1)?;
    let nft_bech32_address = &Address::Nft(NftAddress::new(nft_id)).to_bech32(SHIMMER_TESTNET_BECH32_HRP);

    let inputs = build_inputs(vec![
        Nft(
            1_000_000,
            nft_id,
            &bech32_address_0.to_string(),
            None,
            None,
            None,
            None,
            None,
            Some(Chain::from_u32_hardened(vec![
                HD_WALLET_TYPE,
                SHIMMER_COIN_TYPE,
                0,
                0,
                0,
            ])),
        ),
        Basic(
            1_000_000,
            &nft_bech32_address.to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
        ),
        Basic(
            1_000_000,
            &nft_bech32_address.to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
        ),
    ]);

    let outputs = build_outputs(vec![
        Nft(
            1_000_000,
            nft_id,
            &bech32_address_0.to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
        ),
        Basic(
            2_000_000,
            &nft_bech32_address.to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
        ),
    ]);

    let essence = TransactionEssence::Regular(
        RegularTransactionEssence::builder(
            protocol_parameters.network_id(),
            InputsCommitment::new(inputs.iter().map(|i| &i.output)),
        )
        .with_inputs(
            inputs
                .iter()
                .map(|i| Input::Utxo(UtxoInput::from(*i.output_metadata.output_id())))
                .collect(),
        )
        .with_outputs(outputs)
        .finish(&protocol_parameters)?,
    );

    let prepared_transaction_data = PreparedTransactionData {
        essence,
        inputs_data: inputs,
        remainder: None,
    };

    let current_time = 100;

    let unlocks = secret_manager
        .sign_transaction_essence(&prepared_transaction_data, Some(current_time))
        .await?;

    assert_eq!(unlocks.len(), 3);
    assert_eq!((*unlocks).get(0).unwrap().kind(), SignatureUnlock::KIND);
    match (*unlocks).get(1).unwrap() {
        Unlock::Nft(a) => {
            assert_eq!(a.index(), 0);
        }
        _ => panic!("Invalid unlock"),
    }
    match (*unlocks).get(2).unwrap() {
        Unlock::Nft(a) => {
            assert_eq!(a.index(), 0);
        }
        _ => panic!("Invalid unlock"),
    }

    let tx_payload = TransactionPayload::new(prepared_transaction_data.essence.clone(), unlocks)?;

    validate_transaction_payload_length(&tx_payload)?;

    let conflict = verify_semantic(&prepared_transaction_data.inputs_data, &tx_payload, current_time)?;

    if conflict != ConflictReason::None {
        panic!("{conflict:?}, with {tx_payload:#?}");
    }

    Ok(())
}
