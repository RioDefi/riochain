use frame_support::traits::{Currency, Imbalance, OnUnbalanced};
use sp_runtime::traits::Convert;

use pallet_balances::NegativeImbalance;

use rio_payment::{Multiplier, TargetedFeeAdjustment};
use rio_primitives::Balance;

use super::*;
use crate::Balances;

/// Logic for the author to get a portion of fees.
pub struct ToAuthor<R>(sp_std::marker::PhantomData<R>);

impl<R> OnUnbalanced<NegativeImbalance<R>> for ToAuthor<R>
where
    R: pallet_balances::Trait + pallet_authorship::Trait,
    <R as frame_system::Trait>::AccountId: From<AccountId>,
    <R as frame_system::Trait>::AccountId: Into<AccountId>,
    <R as frame_system::Trait>::Event: From<
        pallet_balances::RawEvent<
            <R as frame_system::Trait>::AccountId,
            <R as pallet_balances::Trait>::Balance,
            pallet_balances::DefaultInstance,
        >,
    >,
{
    fn on_nonzero_unbalanced(amount: NegativeImbalance<R>) {
        let numeric_amount = amount.peek();
        let author = <pallet_authorship::Module<R>>::author();
        <pallet_balances::Module<R>>::resolve_creating(
            &<pallet_authorship::Module<R>>::author(),
            amount,
        );
        <frame_system::Module<R>>::deposit_event(pallet_balances::RawEvent::Deposit(
            author,
            numeric_amount,
        ));
    }
}

/// Struct that handles the conversion of Balance -> `u64`. This is used for staking's election
/// calculation.
pub struct CurrencyToVoteHandler;

impl CurrencyToVoteHandler {
    fn factor() -> Balance {
        (Balances::total_issuance() / u64::max_value() as Balance).max(1)
    }
}

impl Convert<Balance, u64> for CurrencyToVoteHandler {
    fn convert(x: Balance) -> u64 {
        (x / Self::factor()) as u64
    }
}

impl Convert<u128, Balance> for CurrencyToVoteHandler {
    fn convert(x: u128) -> Balance {
        x * Self::factor()
    }
}

parameter_types! {
    /// The portion of the `AvailableBlockRatio` that we adjust the fees with. Blocks filled less
    /// than this will decrease the weight and more will increase.
    pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
    /// The adjustment variable of the runtime. Higher values will cause `TargetBlockFullness` to
    /// change the fees more rapidly.
    pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(3, 100_000);
    /// Minimum amount of the multiplier. This value cannot be too low. A test case should ensure
    /// that combined with `AdjustmentVariable`, we can recover from the minimum.
    /// See `multiplier_can_grow_from_zero`.
    pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 1_000_000_000u128);
}
/// Parameterized slow adjusting fee updated based on
/// https://w3f-research.readthedocs.io/en/latest/polkadot/Token%20Economics.html#-2.-slow-adjusting-mechanism
pub type SlowAdjustingFeeUpdate<R> =
    TargetedFeeAdjustment<R, TargetBlockFullness, AdjustmentVariable, MinimumMultiplier>;
