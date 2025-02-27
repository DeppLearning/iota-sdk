// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the receipt milestone option.

mod migrated_funds_entry;
mod tail_transaction_hash;

use alloc::vec::Vec;
use core::ops::RangeInclusive;

use hashbrown::HashMap;
use iterator_sorted::is_unique_sorted;
use packable::{bounded::BoundedU16, prefix::VecPrefix, Packable, PackableExt};

pub use self::{migrated_funds_entry::MigratedFundsEntry, tail_transaction_hash::TailTransactionHash};
use crate::types::block::{
    output::OUTPUT_COUNT_RANGE,
    payload::{milestone::MilestoneIndex, Payload, TreasuryTransactionPayload},
    protocol::ProtocolParameters,
    Error,
};

const MIGRATED_FUNDS_ENTRY_RANGE: RangeInclusive<u16> = OUTPUT_COUNT_RANGE;

pub(crate) type ReceiptFundsCount =
    BoundedU16<{ *MIGRATED_FUNDS_ENTRY_RANGE.start() }, { *MIGRATED_FUNDS_ENTRY_RANGE.end() }>;

/// Receipt is a listing of migrated funds.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error)]
#[packable(unpack_visitor = ProtocolParameters)]
pub struct ReceiptMilestoneOption {
    migrated_at: MilestoneIndex,
    last: bool,
    #[packable(unpack_error_with = |e| e.unwrap_item_err_or_else(|p| Error::InvalidReceiptFundsCount(p.into())))]
    #[packable(verify_with = verify_funds_packable)]
    funds: VecPrefix<MigratedFundsEntry, ReceiptFundsCount>,
    #[packable(verify_with = verify_transaction_packable)]
    transaction: Payload,
}

impl ReceiptMilestoneOption {
    /// The milestone option kind of a [`ReceiptMilestoneOption`].
    pub const KIND: u8 = 0;

    /// Creates a new [`ReceiptMilestoneOption`].
    pub fn new(
        migrated_at: MilestoneIndex,
        last: bool,
        funds: Vec<MigratedFundsEntry>,
        transaction: TreasuryTransactionPayload,
        token_supply: u64,
    ) -> Result<Self, Error> {
        let funds = VecPrefix::<MigratedFundsEntry, ReceiptFundsCount>::try_from(funds)
            .map_err(Error::InvalidReceiptFundsCount)?;

        verify_funds::<true>(&funds, &token_supply)?;

        Ok(Self {
            migrated_at,
            last,
            funds,
            transaction: transaction.into(),
        })
    }

    /// Returns the milestone index at which the funds of a [`ReceiptMilestoneOption`] were migrated at in the legacy
    /// network.
    pub fn migrated_at(&self) -> MilestoneIndex {
        self.migrated_at
    }

    /// Returns whether a [`ReceiptMilestoneOption`] is the final one for a given migrated at index.
    pub fn last(&self) -> bool {
        self.last
    }

    /// The funds which were migrated with a [`ReceiptMilestoneOption`].
    pub fn funds(&self) -> &[MigratedFundsEntry] {
        &self.funds
    }

    /// The [`TreasuryTransactionPayload`](crate::types::block::payload::treasury_transaction::TreasuryTransactionPayload) used
    /// to fund the funds of a [`ReceiptMilestoneOption`].
    pub fn transaction(&self) -> &TreasuryTransactionPayload {
        if let Payload::TreasuryTransaction(ref transaction) = self.transaction {
            transaction
        } else {
            // It has already been validated at construction that `transaction` is a `TreasuryTransactionPayload`.
            unreachable!()
        }
    }

    /// Returns the sum of all [`MigratedFundsEntry`] items within a [`ReceiptMilestoneOption`].
    pub fn amount(&self) -> u64 {
        self.funds.iter().map(|f| f.amount()).sum()
    }
}

fn verify_funds<const VERIFY: bool>(funds: &[MigratedFundsEntry], token_supply: &u64) -> Result<(), Error> {
    if VERIFY {
        // Funds must be lexicographically sorted and unique in their serialised forms.
        if !is_unique_sorted(funds.iter().map(PackableExt::pack_to_vec)) {
            return Err(Error::ReceiptFundsNotUniqueSorted);
        }

        let mut tail_transaction_hashes = HashMap::with_capacity(funds.len());
        let mut funds_sum: u64 = 0;

        for (index, funds) in funds.iter().enumerate() {
            if let Some(previous) = tail_transaction_hashes.insert(funds.tail_transaction_hash().as_ref(), index) {
                return Err(Error::TailTransactionHashNotUnique {
                    previous,
                    current: index,
                });
            }

            funds_sum = funds_sum
                .checked_add(funds.amount())
                .ok_or_else(|| Error::InvalidReceiptFundsSum(funds_sum as u128 + funds.amount() as u128))?;

            if funds_sum > *token_supply {
                return Err(Error::InvalidReceiptFundsSum(funds_sum as u128));
            }
        }
    }

    Ok(())
}

fn verify_funds_packable<const VERIFY: bool>(
    funds: &[MigratedFundsEntry],
    protocol_parameters: &ProtocolParameters,
) -> Result<(), Error> {
    verify_funds::<VERIFY>(funds, &protocol_parameters.token_supply())
}

fn verify_transaction<const VERIFY: bool>(transaction: &Payload) -> Result<(), Error> {
    if VERIFY && !matches!(transaction, Payload::TreasuryTransaction(_)) {
        Err(Error::InvalidPayloadKind(transaction.kind()))
    } else {
        Ok(())
    }
}

fn verify_transaction_packable<const VERIFY: bool>(transaction: &Payload, _: &ProtocolParameters) -> Result<(), Error> {
    verify_transaction::<VERIFY>(transaction)
}

#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize};

    pub use super::migrated_funds_entry::dto::MigratedFundsEntryDto;
    use super::*;
    use crate::types::block::{
        payload::dto::{PayloadDto, TreasuryTransactionPayloadDto},
        Error,
    };

    ///
    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ReceiptMilestoneOptionDto {
        #[serde(rename = "type")]
        pub kind: u8,
        pub migrated_at: u32,
        pub funds: Vec<MigratedFundsEntryDto>,
        pub transaction: PayloadDto,
        #[serde(rename = "final")]
        pub last: bool,
    }

    impl From<&ReceiptMilestoneOption> for ReceiptMilestoneOptionDto {
        fn from(value: &ReceiptMilestoneOption) -> Self {
            Self {
                kind: ReceiptMilestoneOption::KIND,
                migrated_at: *value.migrated_at(),
                last: value.last(),
                funds: value.funds().iter().map(Into::into).collect::<_>(),
                transaction: PayloadDto::TreasuryTransaction(
                    TreasuryTransactionPayloadDto::from(value.transaction()).into(),
                ),
            }
        }
    }

    impl ReceiptMilestoneOption {
        pub fn try_from_dto(value: &ReceiptMilestoneOptionDto, token_supply: u64) -> Result<Self, Error> {
            Self::new(
                MilestoneIndex(value.migrated_at),
                value.last,
                value
                    .funds
                    .iter()
                    .map(|f| MigratedFundsEntry::try_from_dto(f, token_supply))
                    .collect::<Result<_, _>>()?,
                if let PayloadDto::TreasuryTransaction(ref transaction) = value.transaction {
                    TreasuryTransactionPayload::try_from_dto(transaction.as_ref(), token_supply)?
                } else {
                    return Err(Error::InvalidField("transaction"));
                },
                token_supply,
            )
        }

        pub fn try_from_dto_unverified(value: &ReceiptMilestoneOptionDto) -> Result<Self, Error> {
            let funds = value
                .funds
                .iter()
                .map(MigratedFundsEntry::try_from_dto_unverified)
                .collect::<Result<Vec<_>, _>>()?;

            Ok(Self {
                migrated_at: MilestoneIndex(value.migrated_at),
                last: value.last,
                funds: VecPrefix::<MigratedFundsEntry, ReceiptFundsCount>::try_from(funds)
                    .map_err(Error::InvalidReceiptFundsCount)?,
                transaction: if let PayloadDto::TreasuryTransaction(ref transaction) = value.transaction {
                    TreasuryTransactionPayload::try_from_dto_unverified(transaction.as_ref())?.into()
                } else {
                    return Err(Error::InvalidField("transaction"));
                },
            })
        }
    }
}
