//! this module provides a simple account to aggregate the transaction fee
//! this account is under control of the RIO team
//! this is done by applying the "transaction_payment" strategy

#![cfg_attr(not(feature = "std"), no_std)]

mod weight_info;

use frame_support::{
    decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    traits::{Currency, Imbalance, OnUnbalanced},
};
use frame_system::ensure_root;
use sp_runtime::traits::{Saturating, StaticLookup, Zero};

use rio_primitives::{traits::BuyFeeAsset, types::FeeExchange, CurrencyId};
use rio_support::debug;

use orml_traits::{MultiCurrency, MultiReservableCurrency};

use weight_info::WeightInfo;

type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;
type NegativeImbalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::NegativeImbalance;

/// The module's configuration trait.
pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    /// native currency
    type Currency: Currency<Self::AccountId>;
    /// Currency type for LockedRFuel
    type MultiCurrency: MultiCurrency<Self::AccountId, CurrencyId = CurrencyId, Balance = BalanceOf<Self>>
        + MultiReservableCurrency<Self::AccountId>;

    type WeightInfo: WeightInfo;
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as RioPaymentFee {
        AccountId get(fn account_id) config() : T::AccountId;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        #[weight = T::WeightInfo::set_account()]
        pub fn set_account(origin, who: <T::Lookup as StaticLookup>::Source) -> DispatchResult {
            ensure_root(origin)?;
            let who = T::Lookup::lookup(who)?;

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
        AccountId = <T as frame_system::Trait>::AccountId,
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
    type Balance = BalanceOf<T>;
    type FeeExchange = FeeExchange<BalanceOf<T>>;

    fn buy_fee_asset(
        who: &Self::AccountId,
        amount: Self::Balance,
        exchange_op: &Self::FeeExchange,
    ) -> Result<Self::Balance, DispatchError> {
        let locked_currency = rio_protocol::LOCKED_RFUEL.into();

        let require = amount; // just for log
        let balance = T::MultiCurrency::reserved_balance(locked_currency, who);
        let amount = sp_std::cmp::min(amount, balance);
        let amount = sp_std::cmp::min(amount, exchange_op.max_payment());

        if amount.is_zero() {
            return Ok(Zero::zero());
        }

        let remain = T::MultiCurrency::slash_reserved(locked_currency, who, amount);
        let real = amount.saturating_sub(remain);
        debug!(
            "[buy_fee_asset]|who:{:?}|require_fee:{:?}|real_fee:{:?}|real_pay:{:?}|remain:{:?}",
            who, require, amount, real, remain,
        );
        Ok(real)
    }

    fn refund_fee_asset(who: &Self::AccountId, amount: Self::Balance) -> Result<(), DispatchError> {
        let locked_currency = rio_protocol::LOCKED_RFUEL.into();
        T::MultiCurrency::deposit(locked_currency, who, amount)
            .expect("deposit remained balance must success");
        T::MultiCurrency::reserve(locked_currency, who, amount)
    }
}
