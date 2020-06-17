// Copyright 2018-2020 Parity Technologies (UK) Ltd. and Centrality Investments Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! # Transaction Payment Module
//!
//! This module provides the basic logic needed to pay the absolute minimum amount needed for a
//! transaction to be included. This includes:
//!   - _weight fee_: A fee proportional to amount of weight a transaction consumes.
//!   - _length fee_: A fee proportional to the encoded length of the transaction.
//!   - _tip_: An optional tip. Tip increases the priority of the transaction, giving it a higher
//!     chance to be included by the transaction queue.
//!
//! Additionally, this module allows one to configure:
//!   - The mapping between one unit of weight to one unit of fee via [`WeightToFee`].
//!   - A means of updating the fee for the next block, via defining a multiplier, based on the
//!     final state of the chain at the end of the previous block. This can be configured via
//!     [`FeeMultiplierUpdate`]

#![cfg_attr(not(feature = "std"), no_std)]
// pub mod constants;

use codec::{Decode, Encode};
use rstd::prelude::*;
use sp_runtime::{
    traits::{
        Bounded, Convert, Member, SaturatedConversion, Saturating, SignedExtension,
        SimpleArithmetic, Zero,
    },
    transaction_validity::{
        InvalidTransaction, TransactionPriority, TransactionValidity, TransactionValidityError,
        ValidTransaction,
    },
    Fixed64,
};
use support::{
    decl_module, decl_storage,
    traits::{Currency, ExistenceRequirement, Get, OnUnbalanced, WithdrawReason},
    weights::{DispatchInfo, GetDispatchInfo, Weight},
    Parameter,
};
use transaction_payment_rpc_runtime_api::RuntimeDispatchInfo;

use rio_primitives::{traits::BuyFeeAsset, types::FeeExchange};
use rio_support::debug;

type Multiplier = Fixed64;
type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;
type NegativeImbalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;

pub trait Trait: system::Trait {
    /// The arithmetic type of asset identifier.
    type AssetId: Parameter + Member + SimpleArithmetic + Default + Copy;

    /// The currency type in which fees will be paid.
    type Currency: Currency<Self::AccountId> + Send + Sync;

    /// Handler for the unbalanced reduction when taking transaction fees.
    type OnTransactionPayment: OnUnbalanced<NegativeImbalanceOf<Self>>;

    /// The fee to be paid for making a transaction; the base.
    type TransactionBaseFee: Get<BalanceOf<Self>>;

    /// The fee to be paid for making a transaction; the per-byte portion.
    type TransactionByteFee: Get<BalanceOf<Self>>;

    /// Convert a weight value into a deductible fee based on the currency type.
    type WeightToFee: Convert<Weight, BalanceOf<Self>>;

    /// Update the multiplier of the next block, based on the previous block's weight.
    type FeeMultiplierUpdate: Convert<(Weight, Multiplier), Multiplier>;

    /// A service which will buy fee assets if signalled by the extrinsic.
    type BuyFeeAsset: BuyFeeAsset<
        AccountId = Self::AccountId,
        Balance = BalanceOf<Self>,
        FeeExchange = FeeExchange<BalanceOf<Self>>,
    >;
}

decl_storage! {
    trait Store for Module<T: Trait> as RioTransactionPayment {
        NextFeeMultiplier get(fn next_fee_multiplier): Multiplier = Multiplier::from_parts(0);
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        /// The fee to be paid for making a transaction; the base.
        const TransactionBaseFee: BalanceOf<T> = T::TransactionBaseFee::get();

        /// The fee to be paid for making a transaction; the per-byte portion.
        const TransactionByteFee: BalanceOf<T> = T::TransactionByteFee::get();

        fn on_finalize() {
            let current_weight = <system::Module<T>>::all_extrinsics_weight();
            NextFeeMultiplier::mutate(|fm| {
                *fm = T::FeeMultiplierUpdate::convert((current_weight, *fm))
            });
        }
    }
}

impl<T: Trait> Module<T> {
    /// Query the data that we know about the fee of a given `call`.
    ///
    /// As this module is not and cannot be aware of the internals of a signed extension, it only
    /// interprets them as some encoded value and takes their length into account.
    ///
    /// All dispatchables must be annotated with weight and will have some fee info. This function
    /// always returns.
    // NOTE: we can actually make it understand `ChargeTransactionPayment`, but would be some hassle
    // for sure. We have to make it aware of the index of `ChargeTransactionPayment` in `Extra`.
    // Alternatively, we could actually execute the tx's per-dispatch and record the balance of the
    // sender before and after the pipeline.. but this is way too much hassle for a very very little
    // potential gain in the future.
    pub fn query_info<Extrinsic: GetDispatchInfo>(
        unchecked_extrinsic: Extrinsic,
        len: u32,
    ) -> RuntimeDispatchInfo<BalanceOf<T>>
    where
        T: Send + Sync,
        BalanceOf<T>: Send + Sync,
    {
        let dispatch_info = <Extrinsic as GetDispatchInfo>::get_dispatch_info(&unchecked_extrinsic);

        let partial_fee =
            <ChargeTransactionPayment<T>>::compute_fee(len, dispatch_info, 0u32.into());
        let DispatchInfo { weight, class, .. } = dispatch_info;

        RuntimeDispatchInfo {
            weight,
            class,
            partial_fee,
        }
    }
}

/// Require the transactor pay for themselves and maybe include a tip to gain additional priority
/// in the queue.
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct ChargeTransactionPayment<T: Trait + Send + Sync> {
    #[codec(compact)]
    tip: BalanceOf<T>,
    // fee_exchange: Option<FeeExchange<BalanceOf<T>>>,
}

impl<T: Trait + Send + Sync> ChargeTransactionPayment<T> {
    /// utility constructor. Used only in client/factory code.
    pub fn from(tip: BalanceOf<T>, fee_exchange: Option<FeeExchange<BalanceOf<T>>>) -> Self {
        Self {
            tip, /*fee_exchange*/
        }
    }

    /// Compute the final fee value for a particular transaction.
    ///
    /// The final fee is composed of:
    ///   - _base_fee_: This is the minimum amount a user pays for a transaction.
    ///   - _len_fee_: This is the amount paid merely to pay for size of the transaction.
    ///   - _weight_fee_: This amount is computed based on the weight of the transaction. Unlike
    ///      size-fee, this is not input dependent and reflects the _complexity_ of the execution
    ///      and the time it consumes.
    ///   - _targeted_fee_adjustment_: This is a multiplier that can tune the final fee based on
    ///     the congestion of the network.
    ///   - (optional) _tip_: if included in the transaction, it will be added on top. Only signed
    ///      transactions can have a tip.
    ///
    /// final_fee = base_fee + targeted_fee_adjustment(len_fee + weight_fee) + tip;
    pub fn compute_fee(
        len: u32,
        info: <Self as SignedExtension>::DispatchInfo,
        tip: BalanceOf<T>,
    ) -> BalanceOf<T>
    where
        BalanceOf<T>: Sync + Send,
    {
        if info.pays_fee {
            let len = <BalanceOf<T>>::from(len);
            let per_byte = T::TransactionByteFee::get();
            let len_fee = per_byte.saturating_mul(len);

            let weight_fee = {
                // cap the weight to the maximum defined in runtime, otherwise it will be the `Bounded`
                // maximum of its data type, which is not desired.
                let capped_weight = info
                    .weight
                    .min(<T as system::Trait>::MaximumBlockWeight::get());
                T::WeightToFee::convert(capped_weight)
            };

            // the adjustable part of the fee
            let adjustable_fee = len_fee.saturating_add(weight_fee);
            let targeted_fee_adjustment = NextFeeMultiplier::get();
            // adjusted_fee = adjustable_fee + (adjustable_fee * targeted_fee_adjustment)
            let adjusted_fee =
                targeted_fee_adjustment.saturated_multiply_accumulate(adjustable_fee);

            let base_fee = T::TransactionBaseFee::get();
            let final_fee = base_fee.saturating_add(adjusted_fee).saturating_add(tip);
            debug!("[compute_fee]|final_fee:{:?}|base_fee:{:?}|adjusted_fee:{:?}|tip:{:?}, len_fee:{:?}, weight_fee:{:?}, adjustable_fee:{:?}",
                        final_fee, base_fee, adjusted_fee, tip, len_fee, weight_fee, adjustable_fee);

            final_fee
        } else {
            tip
        }
    }
}

impl<T: Trait + Send + Sync> rstd::fmt::Debug for ChargeTransactionPayment<T> {
    #[cfg(feature = "std")]
    fn fmt(&self, f: &mut rstd::fmt::Formatter) -> rstd::fmt::Result {
        write!(f, "ChargeTransactionPayment<{:?}>", self)
    }
    #[cfg(not(feature = "std"))]
    fn fmt(&self, _: &mut rstd::fmt::Formatter) -> rstd::fmt::Result {
        Ok(())
    }
}

impl<T> SignedExtension for ChargeTransactionPayment<T>
where
    T: Trait + Send + Sync,
    BalanceOf<T>: Send + Sync,
{
    type AccountId = T::AccountId;
    type Call = T::Call;
    type AdditionalSigned = ();
    type Pre = ();
    type DispatchInfo = DispatchInfo;
    fn additional_signed(&self) -> rstd::result::Result<(), TransactionValidityError> {
        Ok(())
    }

    fn validate(
        &self,
        who: &Self::AccountId,
        _call: &Self::Call,
        info: Self::DispatchInfo,
        len: usize,
    ) -> TransactionValidity {
        let fee = Self::compute_fee(len as u32, info, self.tip);

        // Only mess with balances if the fee is not zero.
        if !fee.is_zero() {
            // if let Some(exchange) = &self.fee_exchange {
            // Buy the locked fee currency paying with the user's nominated fee currency
            let exchange = FeeExchange::new_v1(BalanceOf::<T>::max_value());
            let _ = T::BuyFeeAsset::buy_fee_asset(who, fee, &exchange).map_err(|_| {
                // TODO change this after update substrate
                TransactionValidityError::Invalid(InvalidTransaction::Custom(100))
            })?;
            // }

            // Pay for the transaction `fee` in the native fee currency
            let imbalance = match T::Currency::withdraw(
                who,
                fee,
                if self.tip.is_zero() {
                    WithdrawReason::TransactionPayment.into()
                } else {
                    WithdrawReason::TransactionPayment | WithdrawReason::Tip
                },
                ExistenceRequirement::KeepAlive,
            ) {
                Ok(imbalance) => imbalance,
                Err(_) => return InvalidTransaction::Payment.into(),
            };

            T::OnTransactionPayment::on_unbalanced(imbalance);
        }

        // The transaction is valid
        let mut r = ValidTransaction::default();
        // NOTE: we probably want to maximize the _fee (of any type) per weight unit_ here, which
        // will be a bit more than setting the priority to tip. For now, this is enough.
        r.priority = fee.saturated_into::<TransactionPriority>();
        Ok(r)
    }
}
