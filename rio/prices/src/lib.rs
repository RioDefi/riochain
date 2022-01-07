//! # Prices Module
//!
//! ## Overview
//!
//! The data from Oracle cannot be used in business, prices module will do some
//! process and feed prices for Rio.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_event, decl_module, decl_storage, traits::EnsureOrigin, weights::DispatchClass,
};
use frame_system::{self as system};
use orml_traits::{DataFeeder, DataProvider};
use orml_utilities::with_transaction_result;

use sp_runtime::traits::CheckedDiv;

use rio_primitives::{CurrencyId, Price};
use rio_support::traits::PriceProvider;

mod mock;
mod tests;

pub trait Trait: system::Trait {
    type Event: From<Event> + Into<<Self as system::Trait>::Event>;

    /// The data source, such as Oracle.
    type Source: DataProvider<CurrencyId, Price> + DataFeeder<CurrencyId, Price, Self::AccountId>;

    /// The origin which may lock and unlock prices feed to system.
    type LockOrigin: EnsureOrigin<Self::Origin>;
}

decl_event!(
    pub enum Event {
        /// Lock price. [currency_id, locked_price]
        LockPrice(CurrencyId, Price),
        /// Unlock price. [currency_id]
        UnlockPrice(CurrencyId),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as RioPrices {
        /// Mapping from currency id to it's locked price
        LockedPrice get(fn locked_price): map hasher(twox_64_concat) CurrencyId => Option<Price>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Lock the price and feed it to system.
        ///
        /// The dispatch origin of this call must be `LockOrigin`.
        ///
        /// - `currency_id`: currency type.
        #[weight = (10_000, DispatchClass::Operational)]
        fn lock_price(origin, currency_id: CurrencyId) {
            with_transaction_result(|| {
                T::LockOrigin::ensure_origin(origin)?;
                <Module<T> as PriceProvider<CurrencyId>>::lock_price(currency_id);
                Ok(())
            })?;
        }

        /// Unlock the price and get the price from `PriceProvider` again
        ///
        /// The dispatch origin of this call must be `LockOrigin`.
        ///
        /// - `currency_id`: currency type.
        #[weight = (10_000, DispatchClass::Operational)]
        fn unlock_price(origin, currency_id: CurrencyId) {
            with_transaction_result(|| {
                T::LockOrigin::ensure_origin(origin)?;
                <Module<T> as PriceProvider<CurrencyId>>::unlock_price(currency_id);
                Ok(())
            })?;
        }
    }
}

impl<T: Trait> Module<T> {}

impl<T: Trait> PriceProvider<CurrencyId> for Module<T> {
    /// get related price between two currency types
    fn get_relative_price(
        base_currency_id: CurrencyId,
        quote_currency_id: CurrencyId,
    ) -> Option<Price> {
        if let (Some(base_price), Some(quote_price)) = (
            Self::get_price(base_currency_id),
            Self::get_price(quote_currency_id),
        ) {
            base_price.checked_div(&quote_price)
        } else {
            None
        }
    }

    /// get price in USD
    fn get_price(currency_id: CurrencyId) -> Option<Price> {
        // if locked price exists, return it, otherwise return latest price from oracle.
        Self::locked_price(currency_id).or_else(|| T::Source::get(&currency_id))
    }

    fn lock_price(currency_id: CurrencyId) {
        // lock price when get valid price from source
        if let Some(val) = T::Source::get(&currency_id) {
            LockedPrice::insert(currency_id, val);
            <Module<T>>::deposit_event(Event::LockPrice(currency_id, val));
        }
    }

    fn unlock_price(currency_id: CurrencyId) {
        LockedPrice::remove(currency_id);
        <Module<T>>::deposit_event(Event::UnlockPrice(currency_id));
    }
}
