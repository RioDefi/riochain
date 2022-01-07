#![cfg_attr(not(feature = "std"), no_std)]

use sp_runtime::DispatchResult;
use sp_std::prelude::*;

use frame_support::{decl_error, decl_event, decl_module, decl_storage};

use rio_assets::*;

/// The module's configuration trait.
pub trait Trait: frame_system::Trait + rio_assets::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_error! {
    /// Error for the generic-asset module.
    pub enum Error for Module<T: Trait> {

    }
}

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Trait>::AccountId,
    {
        Holder(AccountId),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as RioAssetsExt {
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// create a new asset with full permissions granted to whoever make the call
        /// *sudo or proposal approved only*
        #[weight = T::WeightInfo::create()]
        pub fn create(
            origin,
            currency_id: T::CurrencyId,
            asset_info: AssetInfo,
        ) -> DispatchResult {
            rio_assets::Module::<T>::create(origin, currency_id, asset_info)
        }

        #[weight = T::WeightInfo::update_asset_info()]
        pub fn update_asset_info(
            origin,
            currency_id: T::CurrencyId,
            asset_info: AssetInfo,
        ) -> DispatchResult {
            rio_assets::Module::<T>::update_asset_info(origin, currency_id, asset_info)
        }

        #[weight = T::WeightInfo::update_restriction()]
        pub fn update_restriction(origin, currency_id: T::CurrencyId, restrictions: Restrictions) -> DispatchResult {
            rio_assets::Module::<T>::update_restriction(origin, currency_id, restrictions)?;
            Ok(())
        }

        #[weight = T::WeightInfo::offline_asset()]
        pub fn offline_asset(origin, currency_id: T::CurrencyId) -> DispatchResult {
            rio_assets::Module::<T>::offline_asset(origin, currency_id)?;
            Ok(())
        }

        #[weight = T::WeightInfo::online_asset()]
        pub fn online_asset(origin, currency_id: T::CurrencyId) -> DispatchResult {
            rio_assets::Module::<T>::online_asset(origin, currency_id)
        }
    }
}

impl<T: Trait> Module<T> {}
