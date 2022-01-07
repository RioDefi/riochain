use bitmask::bitmask;
use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use sp_runtime::RuntimeDebug;
use sp_std::{prelude::*, slice::Iter};

use frame_support::dispatch::DispatchResult;

use rio_primitives::Text;

use crate::{Error, Module, Trait};

macro_rules! define_enum {
    (
    $(#[$attr:meta])*
    $Name:ident { $($Variant:ident),* $(,)* }) =>
    {
        $(#[$attr])*
        pub enum $Name {
            $($Variant),*,
        }
        impl $Name {
            pub fn iterator() -> Iter<'static, $Name> {
                static ENUM_ITEMS: &[$Name] = &[$($Name::$Variant),*];
                ENUM_ITEMS.iter()
            }
        }
    }
}

define_enum!(
    #[derive(PartialEq, Eq, Ord, PartialOrd, Clone, Copy, Encode, Decode, RuntimeDebug)]
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    Chain {
        Rio,
        Bitcoin,
        Litecoin,
        Ethereum,
        EOS,
        Polkadot,
        Kusama,
        ChainX
    }
);

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct AssetInfo {
    pub chain: Chain,
    pub symbol: Text,
    pub name: Text,
    pub decimals: u8,
    pub desc: Text,
}

bitmask! {
    #[derive(Encode, Decode)]
    #[cfg_attr(not(feature = "std"), derive(RuntimeDebug))]
    #[cfg_attr(feature = "std", derive(Deserialize, Serialize))]
    pub mask Restrictions: u32 where

    #[derive(Encode, Decode)]
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    flags Restriction {
        Transferable        = 1 << 0,
        Depositable         = 1 << 1,
        Withdrawable        = 1 << 2,
        Slashable           = 1 << 3,
        Reservable          = 1 << 4,
        Unreservable        = 1 << 5,
    }
}

impl Default for Restrictions {
    fn default() -> Self {
        Self::none()
    }
}
impl<T: Trait> Module<T> {
    pub fn can_do(currency_id: &T::CurrencyId, restriction: Restriction) -> DispatchResult {
        if Self::asset_restrictions(currency_id).contains(restriction) {
            Err(Error::<T>::RestrictedAction)?
        } else {
            Ok(())
        }
    }
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct TotalAssetInfo<Balance> {
    pub info: AssetInfo,
    pub balance: Balance,
    pub is_online: bool,
    pub restrictions: Restrictions,
}
