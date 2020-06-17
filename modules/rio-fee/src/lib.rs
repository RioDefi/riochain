//! this module provides a simple account to aggregate the transaction fee
//! this account is under control of the RIO team
//! this is done by applying the "transaction_payment" strategy

#![cfg_attr(not(feature = "std"), no_std)]

use support::{
    decl_event, decl_module, decl_storage, dispatch,
    dispatch::DispatchError,
    traits::{Currency, Imbalance, OnUnbalanced, WithdrawReason},
};
use system::ensure_root;

use rio_primitives::{traits::BuyFeeAsset, types::FeeExchange};
use rio_support::debug;

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;
// type PositiveImbalanceOf<T> =
//     <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::PositiveImbalance;
type NegativeImbalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;

/// The module's configuration trait.
pub trait Trait: system::Trait + rio_assets::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type Currency: Currency<Self::AccountId>;
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as RioFee {
        AccountId get(account_id) config() : T::AccountId;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        pub fn set_account(origin, who: T::AccountId) -> dispatch::Result {
            ensure_root(origin)?;
            <AccountId<T>>::put(who.clone());
            Self::deposit_event(RawEvent::AccountChanged(who));
            Ok(())
        }
    }
}

#[rustfmt::skip]
decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        Balance = BalanceOf<T>,
    {
        AccountChanged(AccountId),
        FeeDeposit(Balance),
    }
);

impl<T: Trait> OnUnbalanced<NegativeImbalanceOf<T>> for Module<T> {
    fn on_nonzero_unbalanced(amount: NegativeImbalanceOf<T>) {
        let numeric_amount = amount.peek();
        T::Currency::deposit_creating(&Self::account_id(), numeric_amount);
        Self::deposit_event(RawEvent::FeeDeposit(numeric_amount));
        // consume the imbalance by doing nothing
    }
}

impl<T: Trait> BuyFeeAsset for Module<T> {
    type AccountId = T::AccountId;
    type Balance = T::Balance;
    type FeeExchange = FeeExchange<T::Balance>;

    fn buy_fee_asset(
        who: &Self::AccountId,
        amount: Self::Balance,
        exchange_op: &Self::FeeExchange,
    ) -> Result<Self::Balance, DispatchError> {
        let locked_id = rio_assets::protocol::LOCKED_RFUEL.into();
        let to_id = rio_assets::protocol::RFUEL.into();

        let require = amount; // just for log
        let balance = rio_assets::Module::<T>::free_balance(&locked_id, who);
        let amount = sp_std::cmp::min(amount, balance);
        let amount = sp_std::cmp::min(amount, exchange_op.max_payment());

        debug!(
            "[buy_fee_asset]|who:{:?}|require_fee:{:?}|real_fee:{:?}",
            who, require, amount
        );

        rio_assets::Module::<T>::make_transfer_between_assets(
            &locked_id,
            who,
            &to_id,
            who,
            WithdrawReason::TransactionPayment.into(),
            amount,
        )?;

        Ok(amount)
    }
}
