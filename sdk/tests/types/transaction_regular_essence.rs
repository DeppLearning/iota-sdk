// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_sdk::types::block::{
    address::{Address, AliasAddress, Ed25519Address},
    input::{Input, TreasuryInput, UtxoInput},
    output::{
        unlock_condition::{
            AddressUnlockCondition, GovernorAddressUnlockCondition, ImmutableAliasAddressUnlockCondition,
            StateControllerAddressUnlockCondition,
        },
        AliasId, AliasOutput, BasicOutput, ChainId, FoundryId, FoundryOutput, NativeToken, NftId, NftOutput, Output,
        SimpleTokenScheme, TokenId, TokenScheme, TreasuryOutput,
    },
    payload::{
        milestone::MilestoneId,
        transaction::{RegularTransactionEssence, TransactionId},
        Payload,
    },
    protocol::protocol_parameters,
    rand::{
        bytes::rand_bytes_array,
        output::rand_inputs_commitment,
        payload::{rand_tagged_data_payload, rand_treasury_transaction_payload},
    },
    Error,
};
use packable::bounded::TryIntoBoundedU16Error;
use primitive_types::U256;

const TRANSACTION_ID: &str = "0x52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";
const ED25519_ADDRESS_1: &str = "0xd56da1eb7726ed482dfe9d457cf548c2ae2a6ce3e053dbf82f11223be476adb9";
const ED25519_ADDRESS_2: &str = "0xefda4275375ac3675abff85235fd25a1522a2044cc6027a31b310857246f18c0";

#[test]
fn kind() {
    assert_eq!(RegularTransactionEssence::KIND, 1);
}

#[test]
fn build_valid() {
    let protocol_parameters = protocol_parameters();
    let transaction_id = TransactionId::new(prefix_hex::decode(TRANSACTION_ID).unwrap());
    let input1 = Input::Utxo(UtxoInput::new(transaction_id, 0).unwrap());
    let input2 = Input::Utxo(UtxoInput::new(transaction_id, 1).unwrap());
    let bytes: [u8; 32] = prefix_hex::decode(ED25519_ADDRESS_1).unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let output = Output::Basic(
        BasicOutput::build_with_amount(amount)
            .add_unlock_condition(AddressUnlockCondition::new(address))
            .finish(protocol_parameters.token_supply())
            .unwrap(),
    );

    let essence = RegularTransactionEssence::builder(protocol_parameters.network_id(), rand_inputs_commitment())
        .with_inputs(vec![input1, input2])
        .add_output(output)
        .finish(&protocol_parameters);

    assert!(essence.is_ok());
}

#[test]
fn build_valid_with_payload() {
    let protocol_parameters = protocol_parameters();
    let transaction_id = TransactionId::new(prefix_hex::decode(TRANSACTION_ID).unwrap());
    let input1 = Input::Utxo(UtxoInput::new(transaction_id, 0).unwrap());
    let input2 = Input::Utxo(UtxoInput::new(transaction_id, 1).unwrap());
    let bytes: [u8; 32] = prefix_hex::decode(ED25519_ADDRESS_1).unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let output = Output::Basic(
        BasicOutput::build_with_amount(amount)
            .add_unlock_condition(AddressUnlockCondition::new(address))
            .finish(protocol_parameters.token_supply())
            .unwrap(),
    );

    let essence = RegularTransactionEssence::builder(protocol_parameters.network_id(), rand_inputs_commitment())
        .with_inputs(vec![input1, input2])
        .add_output(output)
        .with_payload(rand_tagged_data_payload())
        .finish(&protocol_parameters);

    assert!(essence.is_ok());
}

#[test]
fn build_valid_add_inputs_outputs() {
    let protocol_parameters = protocol_parameters();
    let transaction_id = TransactionId::new(prefix_hex::decode(TRANSACTION_ID).unwrap());
    let input1 = Input::Utxo(UtxoInput::new(transaction_id, 0).unwrap());
    let input2 = Input::Utxo(UtxoInput::new(transaction_id, 1).unwrap());
    let bytes: [u8; 32] = prefix_hex::decode(ED25519_ADDRESS_1).unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let output = Output::Basic(
        BasicOutput::build_with_amount(amount)
            .add_unlock_condition(AddressUnlockCondition::new(address))
            .finish(protocol_parameters.token_supply())
            .unwrap(),
    );

    let essence = RegularTransactionEssence::builder(protocol_parameters.network_id(), rand_inputs_commitment())
        .with_inputs(vec![input1, input2])
        .add_output(output)
        .finish(&protocol_parameters);

    assert!(essence.is_ok());
}

#[test]
fn build_invalid_payload_kind() {
    let protocol_parameters = protocol_parameters();
    let transaction_id = TransactionId::new(prefix_hex::decode(TRANSACTION_ID).unwrap());
    let input1 = Input::Utxo(UtxoInput::new(transaction_id, 0).unwrap());
    let input2 = Input::Utxo(UtxoInput::new(transaction_id, 1).unwrap());
    let bytes: [u8; 32] = prefix_hex::decode(ED25519_ADDRESS_1).unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let output = Output::Basic(
        BasicOutput::build_with_amount(amount)
            .add_unlock_condition(AddressUnlockCondition::new(address))
            .finish(protocol_parameters.token_supply())
            .unwrap(),
    );

    let essence = RegularTransactionEssence::builder(protocol_parameters.network_id(), rand_inputs_commitment())
        .with_inputs(vec![input1, input2])
        .add_output(output)
        .with_payload(rand_treasury_transaction_payload(protocol_parameters.token_supply()))
        .finish(&protocol_parameters);

    assert!(matches!(essence, Err(Error::InvalidPayloadKind(4))));
}

#[test]
fn build_invalid_input_count_low() {
    let protocol_parameters = protocol_parameters();
    let bytes: [u8; 32] = prefix_hex::decode(ED25519_ADDRESS_1).unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let output = Output::Basic(
        BasicOutput::build_with_amount(amount)
            .add_unlock_condition(AddressUnlockCondition::new(address))
            .finish(protocol_parameters.token_supply())
            .unwrap(),
    );

    let essence = RegularTransactionEssence::builder(protocol_parameters.network_id(), rand_inputs_commitment())
        .add_output(output)
        .finish(&protocol_parameters);

    assert!(matches!(
        essence,
        Err(Error::InvalidInputCount(TryIntoBoundedU16Error::Invalid(0)))
    ));
}

#[test]
fn build_invalid_input_count_high() {
    let protocol_parameters = protocol_parameters();
    let transaction_id = TransactionId::new(prefix_hex::decode(TRANSACTION_ID).unwrap());
    let input = Input::Utxo(UtxoInput::new(transaction_id, 0).unwrap());
    let bytes: [u8; 32] = prefix_hex::decode(ED25519_ADDRESS_1).unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let output = Output::Basic(
        BasicOutput::build_with_amount(amount)
            .add_unlock_condition(AddressUnlockCondition::new(address))
            .finish(protocol_parameters.token_supply())
            .unwrap(),
    );

    let essence = RegularTransactionEssence::builder(protocol_parameters.network_id(), rand_inputs_commitment())
        .with_inputs(vec![input; 129])
        .add_output(output)
        .finish(&protocol_parameters);

    assert!(matches!(
        essence,
        Err(Error::InvalidInputCount(TryIntoBoundedU16Error::Invalid(129)))
    ));
}

#[test]
fn build_invalid_output_count_low() {
    let protocol_parameters = protocol_parameters();
    let transaction_id = TransactionId::new(prefix_hex::decode(TRANSACTION_ID).unwrap());
    let input = Input::Utxo(UtxoInput::new(transaction_id, 0).unwrap());

    let essence = RegularTransactionEssence::builder(protocol_parameters.network_id(), rand_inputs_commitment())
        .add_input(input)
        .finish(&protocol_parameters);

    assert!(matches!(
        essence,
        Err(Error::InvalidOutputCount(TryIntoBoundedU16Error::Invalid(0)))
    ));
}

#[test]
fn build_invalid_output_count_high() {
    let protocol_parameters = protocol_parameters();
    let transaction_id = TransactionId::new(prefix_hex::decode(TRANSACTION_ID).unwrap());
    let input = Input::Utxo(UtxoInput::new(transaction_id, 0).unwrap());
    let bytes: [u8; 32] = prefix_hex::decode(ED25519_ADDRESS_1).unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let output = Output::Basic(
        BasicOutput::build_with_amount(amount)
            .add_unlock_condition(AddressUnlockCondition::new(address))
            .finish(protocol_parameters.token_supply())
            .unwrap(),
    );

    let essence = RegularTransactionEssence::builder(protocol_parameters.network_id(), rand_inputs_commitment())
        .add_input(input)
        .with_outputs(vec![output; 129])
        .finish(&protocol_parameters);

    assert!(matches!(
        essence,
        Err(Error::InvalidOutputCount(TryIntoBoundedU16Error::Invalid(129)))
    ));
}

#[test]
fn build_invalid_duplicate_utxo() {
    let protocol_parameters = protocol_parameters();
    let transaction_id = TransactionId::new(prefix_hex::decode(TRANSACTION_ID).unwrap());
    let input = Input::Utxo(UtxoInput::new(transaction_id, 0).unwrap());
    let bytes: [u8; 32] = prefix_hex::decode(ED25519_ADDRESS_1).unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let output = Output::Basic(
        BasicOutput::build_with_amount(amount)
            .add_unlock_condition(AddressUnlockCondition::new(address))
            .finish(protocol_parameters.token_supply())
            .unwrap(),
    );

    let essence = RegularTransactionEssence::builder(protocol_parameters.network_id(), rand_inputs_commitment())
        .with_inputs(vec![input; 2])
        .add_output(output)
        .finish(&protocol_parameters);

    assert!(matches!(essence, Err(Error::DuplicateUtxo(_))));
}

#[test]
fn build_invalid_input_kind() {
    let protocol_parameters = protocol_parameters();
    let input = Input::Treasury(TreasuryInput::new(MilestoneId::new(rand_bytes_array())));
    let bytes: [u8; 32] = prefix_hex::decode(ED25519_ADDRESS_1).unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let output = Output::Basic(
        BasicOutput::build_with_amount(amount)
            .add_unlock_condition(AddressUnlockCondition::new(address))
            .finish(protocol_parameters.token_supply())
            .unwrap(),
    );

    let essence = RegularTransactionEssence::builder(protocol_parameters.network_id(), rand_inputs_commitment())
        .add_input(input)
        .add_output(output)
        .finish(&protocol_parameters);

    assert!(matches!(essence, Err(Error::InvalidInputKind(1))));
}

#[test]
fn build_invalid_output_kind() {
    let protocol_parameters = protocol_parameters();
    let transaction_id = TransactionId::new(prefix_hex::decode(TRANSACTION_ID).unwrap());
    let input = Input::Utxo(UtxoInput::new(transaction_id, 0).unwrap());
    let amount = 1_000_000;
    let output = Output::Treasury(TreasuryOutput::new(amount, protocol_parameters.token_supply()).unwrap());

    let essence = RegularTransactionEssence::builder(protocol_parameters.network_id(), rand_inputs_commitment())
        .add_input(input)
        .add_output(output)
        .finish(&protocol_parameters);

    assert!(matches!(essence, Err(Error::InvalidOutputKind(2))));
}

#[test]
fn build_invalid_accumulated_output() {
    let protocol_parameters = protocol_parameters();
    let transaction_id = TransactionId::new(prefix_hex::decode(TRANSACTION_ID).unwrap());
    let input = Input::Utxo(UtxoInput::new(transaction_id, 0).unwrap());

    let bytes1: [u8; 32] = prefix_hex::decode(ED25519_ADDRESS_1).unwrap();
    let address1 = Address::from(Ed25519Address::new(bytes1));
    let amount1 = protocol_parameters.token_supply() - 1_000_000;
    let output1 = Output::Basic(
        BasicOutput::build_with_amount(amount1)
            .add_unlock_condition(AddressUnlockCondition::new(address1))
            .finish(protocol_parameters.token_supply())
            .unwrap(),
    );

    let bytes2: [u8; 32] = prefix_hex::decode(ED25519_ADDRESS_2).unwrap();
    let address2 = Address::from(Ed25519Address::new(bytes2));
    let amount2 = 2_000_000;
    let output2 = Output::Basic(
        BasicOutput::build_with_amount(amount2)
            .add_unlock_condition(AddressUnlockCondition::new(address2))
            .finish(protocol_parameters.token_supply())
            .unwrap(),
    );

    let essence = RegularTransactionEssence::builder(protocol_parameters.network_id(), rand_inputs_commitment())
        .add_input(input)
        .with_outputs(vec![output1, output2])
        .finish(&protocol_parameters);

    assert!(matches!(essence, Err(Error::InvalidTransactionAmountSum(_))));
}

#[test]
fn getters() {
    let protocol_parameters = protocol_parameters();
    let transaction_id = TransactionId::new(prefix_hex::decode(TRANSACTION_ID).unwrap());
    let input1 = Input::Utxo(UtxoInput::new(transaction_id, 0).unwrap());
    let input2 = Input::Utxo(UtxoInput::new(transaction_id, 1).unwrap());
    let bytes: [u8; 32] = prefix_hex::decode(ED25519_ADDRESS_1).unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let outputs = vec![Output::Basic(
        BasicOutput::build_with_amount(amount)
            .add_unlock_condition(AddressUnlockCondition::new(address))
            .finish(protocol_parameters.token_supply())
            .unwrap(),
    )];
    let payload = Payload::from(rand_tagged_data_payload());

    let essence = RegularTransactionEssence::builder(protocol_parameters.network_id(), rand_inputs_commitment())
        .with_inputs(vec![input1, input2])
        .with_outputs(outputs.clone())
        .with_payload(payload.clone())
        .finish(&protocol_parameters)
        .unwrap();

    assert_eq!(essence.outputs(), outputs.as_slice());
    assert_eq!(essence.payload().unwrap(), &payload);
}

#[test]
fn duplicate_output_nft() {
    let protocol_parameters = protocol_parameters();
    let transaction_id = TransactionId::new(prefix_hex::decode(TRANSACTION_ID).unwrap());
    let input1 = Input::Utxo(UtxoInput::new(transaction_id, 0).unwrap());
    let input2 = Input::Utxo(UtxoInput::new(transaction_id, 1).unwrap());
    let bytes: [u8; 32] = prefix_hex::decode(ED25519_ADDRESS_1).unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let basic = BasicOutput::build_with_amount(amount)
        .add_unlock_condition(AddressUnlockCondition::new(address))
        .finish_output(protocol_parameters.token_supply())
        .unwrap();
    let nft_id = NftId::from(bytes);
    let nft = NftOutput::build_with_amount(1_000_000, nft_id)
        .add_unlock_condition(AddressUnlockCondition::new(address))
        .finish_output(protocol_parameters.token_supply())
        .unwrap();

    let essence = RegularTransactionEssence::builder(protocol_parameters.network_id(), rand_inputs_commitment())
        .with_inputs(vec![input1, input2])
        .with_outputs(vec![basic, nft.clone(), nft])
        .finish(&protocol_parameters);

    assert!(matches!(
        essence,
        Err(Error::DuplicateOutputChain(ChainId::Nft(nft_id_0))) if nft_id_0 == nft_id
    ));
}

#[test]
fn duplicate_output_nft_null() {
    let protocol_parameters = protocol_parameters();
    let transaction_id = TransactionId::new(prefix_hex::decode(TRANSACTION_ID).unwrap());
    let input1 = Input::Utxo(UtxoInput::new(transaction_id, 0).unwrap());
    let input2 = Input::Utxo(UtxoInput::new(transaction_id, 1).unwrap());
    let bytes: [u8; 32] = prefix_hex::decode(ED25519_ADDRESS_1).unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let basic = BasicOutput::build_with_amount(amount)
        .add_unlock_condition(AddressUnlockCondition::new(address))
        .finish_output(protocol_parameters.token_supply())
        .unwrap();
    let nft_id = NftId::null();
    let nft = NftOutput::build_with_amount(1_000_000, nft_id)
        .add_unlock_condition(AddressUnlockCondition::new(address))
        .finish_output(protocol_parameters.token_supply())
        .unwrap();

    let essence = RegularTransactionEssence::builder(protocol_parameters.network_id(), rand_inputs_commitment())
        .with_inputs(vec![input1, input2])
        .with_outputs(vec![basic, nft.clone(), nft])
        .finish(&protocol_parameters);

    assert!(essence.is_ok());
}

#[test]
fn duplicate_output_alias() {
    let protocol_parameters = protocol_parameters();
    let transaction_id = TransactionId::new(prefix_hex::decode(TRANSACTION_ID).unwrap());
    let input1 = Input::Utxo(UtxoInput::new(transaction_id, 0).unwrap());
    let input2 = Input::Utxo(UtxoInput::new(transaction_id, 1).unwrap());
    let bytes: [u8; 32] = prefix_hex::decode(ED25519_ADDRESS_1).unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let basic = BasicOutput::build_with_amount(amount)
        .add_unlock_condition(AddressUnlockCondition::new(address))
        .finish_output(protocol_parameters.token_supply())
        .unwrap();
    let alias_id = AliasId::from(bytes);
    let alias = AliasOutput::build_with_amount(1_000_000, alias_id)
        .add_unlock_condition(StateControllerAddressUnlockCondition::new(address))
        .add_unlock_condition(GovernorAddressUnlockCondition::new(address))
        .finish_output(protocol_parameters.token_supply())
        .unwrap();

    let essence = RegularTransactionEssence::builder(protocol_parameters.network_id(), rand_inputs_commitment())
        .with_inputs(vec![input1, input2])
        .with_outputs(vec![basic, alias.clone(), alias])
        .finish(&protocol_parameters);

    assert!(matches!(
        essence,
        Err(Error::DuplicateOutputChain(ChainId::Alias(alias_id_0))) if alias_id_0 == alias_id
    ));
}

#[test]
fn duplicate_output_foundry() {
    let protocol_parameters = protocol_parameters();
    let transaction_id = TransactionId::new(prefix_hex::decode(TRANSACTION_ID).unwrap());
    let input1 = Input::Utxo(UtxoInput::new(transaction_id, 0).unwrap());
    let input2 = Input::Utxo(UtxoInput::new(transaction_id, 1).unwrap());
    let bytes: [u8; 32] = prefix_hex::decode(ED25519_ADDRESS_1).unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let basic = BasicOutput::build_with_amount(amount)
        .add_unlock_condition(AddressUnlockCondition::new(address))
        .finish_output(protocol_parameters.token_supply())
        .unwrap();
    let alias_id = AliasId::from(bytes);
    let token_scheme =
        TokenScheme::Simple(SimpleTokenScheme::new(U256::from(70u8), U256::from(0u8), U256::from(100u8)).unwrap());
    let foundry_id = FoundryId::build(&AliasAddress::from(alias_id), 1, token_scheme.kind());
    let token_id = TokenId::from(foundry_id);
    let foundry = FoundryOutput::build_with_amount(1_000_000, 1, token_scheme)
        .add_native_token(NativeToken::new(token_id, U256::from(70u8)).unwrap())
        .add_unlock_condition(ImmutableAliasAddressUnlockCondition::new(AliasAddress::from(alias_id)))
        .finish_output(protocol_parameters.token_supply())
        .unwrap();

    let essence = RegularTransactionEssence::builder(protocol_parameters.network_id(), rand_inputs_commitment())
        .with_inputs(vec![input1, input2])
        .with_outputs(vec![basic, foundry.clone(), foundry])
        .finish(&protocol_parameters);

    assert!(matches!(
        essence,
        Err(Error::DuplicateOutputChain(ChainId::Foundry(foundry_id_0))) if foundry_id_0 == foundry_id
    ));
}
