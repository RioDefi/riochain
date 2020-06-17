use codec::{Decode, Encode};
use rio_support::debug;
use rstd::result;
use sp_runtime::traits::Zero;
use sp_runtime::{
    traits::{Bounded, CheckedAdd, CheckedSub},
    RuntimeDebug,
};
use support::{
    dispatch,
    traits::{
        Currency, ExistenceRequirement, Imbalance, SignedImbalance, UpdateBalanceOutcome,
        WithdrawReasons,
    },
    StorageMap, StorageValue,
};

use crate::imbalances::{NegativeImbalance, PositiveImbalance};
use crate::protocol;
use crate::traits::{AssetIdProvider, RootKeyProvider};
use crate::{FromId, Module, Subtrait, ToId, TotalIssuance, Trait};

pub struct ElevatedTrait<T: Subtrait>(T);
impl<T: Subtrait> Clone for ElevatedTrait<T> {
    fn clone(&self) -> Self {
        unimplemented!()
    }
}
impl<T: Subtrait> PartialEq for ElevatedTrait<T> {
    fn eq(&self, _: &Self) -> bool {
        unimplemented!()
    }
}
impl<T: Subtrait> Eq for ElevatedTrait<T> {}
impl<T: Subtrait> system::Trait for ElevatedTrait<T> {
    type Origin = T::Origin;
    type Call = T::Call;
    type Index = T::Index;
    type BlockNumber = T::BlockNumber;
    type Hash = T::Hash;
    type Hashing = T::Hashing;
    type AccountId = T::AccountId;
    type Lookup = T::Lookup;
    type Header = T::Header;
    type Event = ();
    type MaximumBlockWeight = T::MaximumBlockWeight;
    type MaximumBlockLength = T::MaximumBlockLength;
    type AvailableBlockRatio = T::AvailableBlockRatio;
    type BlockHashCount = T::BlockHashCount;
    type Version = T::Version;
}
impl<T: Subtrait> RootKeyProvider for ElevatedTrait<T> {
    type AccountId = T::AccountId;

    fn root_key() -> Self::AccountId {
        Default::default()
    }
}
impl<T: Subtrait> Trait for ElevatedTrait<T> {
    type Event = ();
    type Balance = T::Balance;
    type AssetId = T::AssetId;
    type RootKey = Self;
    type BeforeAssetTransfer = ();
    type BeforeAssetCreate = ();
    type BeforeAssetMint = ();
    type BeforeAssetBurn = ();
    type OnAssetTransfer = ();
    type OnAssetCreate = ();
    type OnAssetMint = ();
    type OnAssetBurn = ();
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct AssetCurrency<T, U>(rstd::marker::PhantomData<T>, rstd::marker::PhantomData<U>);

impl<T, U> Currency<T::AccountId> for AssetCurrency<T, U>
where
    T: Trait,
    U: AssetIdProvider<AssetId = T::AssetId>,
{
    type Balance = T::Balance;
    type PositiveImbalance = PositiveImbalance<T, U>;
    type NegativeImbalance = NegativeImbalance<T, U>;

    fn total_balance(who: &T::AccountId) -> Self::Balance {
        Self::free_balance(&who) // + Self::reserved_balance(&who)
    }

    fn can_slash(who: &T::AccountId, value: Self::Balance) -> bool {
        <Module<T>>::free_balance(&U::asset_id(), &who) >= value
    }

    /// Returns the total staking asset issuance
    fn total_issuance() -> Self::Balance {
        <Module<T>>::total_issuance(U::asset_id())
    }

    fn minimum_balance() -> Self::Balance {
        Zero::zero()
    }

    fn burn(mut amount: Self::Balance) -> Self::PositiveImbalance {
        <TotalIssuance<T>>::mutate(&U::asset_id(), |issued| {
            issued.checked_sub(&amount).unwrap_or_else(|| {
                amount = *issued;
                Zero::zero()
            })
        });
        PositiveImbalance::new(amount)
    }

    fn issue(mut amount: Self::Balance) -> Self::NegativeImbalance {
        <TotalIssuance<T>>::mutate(&U::asset_id(), |issued| {
            *issued = issued.checked_add(&amount).unwrap_or_else(|| {
                amount = Self::Balance::max_value() - *issued;
                Self::Balance::max_value()
            })
        });
        NegativeImbalance::new(amount)
    }

    fn free_balance(who: &T::AccountId) -> Self::Balance {
        <Module<T>>::free_balance(&U::asset_id(), &who)
    }

    fn ensure_can_withdraw(
        who: &T::AccountId,
        amount: Self::Balance,
        reasons: WithdrawReasons,
        new_balance: Self::Balance,
    ) -> dispatch::Result {
        <Module<T>>::ensure_can_withdraw(&U::asset_id(), who, amount, reasons, new_balance)
    }

    fn transfer(
        transactor: &T::AccountId,
        dest: &T::AccountId,
        value: Self::Balance,
        _: ExistenceRequirement, // no existential deposit policy for generic asset
    ) -> dispatch::Result {
        <Module<T>>::make_transfer(&U::asset_id(), transactor, dest, value)
    }

    fn slash(
        _who: &T::AccountId,
        _value: Self::Balance,
    ) -> (Self::NegativeImbalance, Self::Balance) {
        unimplemented!("")
        // let remaining = Self::slash(&U::asset_id(), who, value);
        // if let Some(r) = remaining {
        //     (NegativeImbalance::new(value - r), r)
        // } else {
        //     (NegativeImbalance::new(value), Zero::zero())
        // }
    }

    fn deposit_into_existing(
        who: &T::AccountId,
        value: Self::Balance,
    ) -> result::Result<Self::PositiveImbalance, &'static str> {
        // No existential deposit rule and creation fee in GA. `deposit_into_existing` is same with `deposit_creating`.
        Ok(Self::deposit_creating(who, value))
    }

    fn deposit_creating(who: &T::AccountId, value: Self::Balance) -> Self::PositiveImbalance {
        let (imbalance, _) = Self::make_free_balance_be(who, Self::free_balance(who) + value);
        if let SignedImbalance::Positive(p) = imbalance {
            p
        } else {
            // Impossible, but be defensive.
            Self::PositiveImbalance::zero()
        }
    }

    fn withdraw(
        who: &T::AccountId,
        value: Self::Balance,
        reasons: WithdrawReasons,
        _: ExistenceRequirement, // no existential deposit policy for generic asset
    ) -> result::Result<Self::NegativeImbalance, &'static str> {
        let new_balance = Self::free_balance(who)
            .checked_sub(&value)
            .ok_or_else(|| "account has too few funds")?;
        Self::ensure_can_withdraw(who, value, reasons, new_balance)?;
        <Module<T>>::set_free_balance(&U::asset_id(), who, new_balance);

        debug!(
            "[Currency|withdraw]|who:{:?}|value:{:?}|reasons:{:?}|current:{:?}",
            who,
            value,
            reasons.encode(),
            Module::<T>::free_balance(&U::asset_id(), who)
        );

        Ok(NegativeImbalance::new(value))
    }

    fn make_free_balance_be(
        who: &T::AccountId,
        balance: Self::Balance,
    ) -> (
        SignedImbalance<Self::Balance, Self::PositiveImbalance>,
        UpdateBalanceOutcome,
    ) {
        let original = <Module<T>>::free_balance(&U::asset_id(), who);
        let imbalance = if original <= balance {
            SignedImbalance::Positive(PositiveImbalance::new(balance - original))
        } else {
            SignedImbalance::Negative(NegativeImbalance::new(original - balance))
        };
        <Module<T>>::set_free_balance(&U::asset_id(), who, balance);

        debug!(
            "[Currency|make_free_balance_be]|who:{:?}|balance:{:?}|current:{:?}",
            who,
            balance,
            Module::<T>::free_balance(&U::asset_id(), who)
        );

        (imbalance, UpdateBalanceOutcome::Updated)
    }
}

pub struct RFUELProvider<T>(rstd::marker::PhantomData<T>);

impl<T: Trait> AssetIdProvider for RFUELProvider<T> {
    type AssetId = T::AssetId;

    fn asset_id() -> Self::AssetId {
        protocol::RFUEL.into()
    }
}

pub struct LockedRFUELProvider<T>(rstd::marker::PhantomData<T>);

impl<T: Trait> AssetIdProvider for LockedRFUELProvider<T> {
    type AssetId = T::AssetId;

    fn asset_id() -> Self::AssetId {
        protocol::LOCKED_RFUEL.into()
    }
}

pub struct FromAssetIdProvider<T>(rstd::marker::PhantomData<T>);
impl<T: Trait> AssetIdProvider for FromAssetIdProvider<T> {
    type AssetId = T::AssetId;
    fn asset_id() -> Self::AssetId {
        <Module<T>>::get_from_id().expect("must in `FromToContext` scope")
    }
}

pub struct ToAssetIdProvider<T>(rstd::marker::PhantomData<T>);
impl<T: Trait> AssetIdProvider for ToAssetIdProvider<T> {
    type AssetId = T::AssetId;
    fn asset_id() -> Self::AssetId {
        <Module<T>>::get_to_id().expect("must in `FromToContext` scope")
    }
}

pub(crate) type FromCurrency<T> = AssetCurrency<T, FromAssetIdProvider<T>>;
pub(crate) type ToCurrency<T> = AssetCurrency<T, ToAssetIdProvider<T>>;

pub(crate) struct FromToContext<T: Trait>(rstd::marker::PhantomData<T>);

impl<T: Trait> FromToContext<T> {
    pub fn new(from: T::AssetId, to: T::AssetId) {
        <FromId<T>>::put(from);
        <ToId<T>>::put(to);
    }
}
impl<T: Trait> Drop for FromToContext<T> {
    fn drop(&mut self) {
        <FromId<T>>::kill();
        <ToId<T>>::kill();
    }
}

// pub struct StakingAssetIdProvider<T>(rstd::marker::PhantomData<T>);
//
// impl<T: Trait> AssetIdProvider for StakingAssetIdProvider<T> {
//     type AssetId = T::AssetId;
//     fn asset_id() -> Self::AssetId {
//         Self::staking_asset_id()
//     }
// }

// pub struct SpendingAssetIdProvider<T>(rstd::marker::PhantomData<T>);
//
// impl<T: Trait> AssetIdProvider for SpendingAssetIdProvider<T> {
//     type AssetId = T::AssetId;
//     fn asset_id() -> Self::AssetId {
//         Self::spending_asset_id()
//     }
// }

// impl<T> LockableCurrency<T::AccountId> for AssetCurrency<T, StakingAssetIdProvider<T>>
//     where
//         T: Trait,
//         T::Balance: MaybeSerializeDeserialize + Debug,
// {
//     type Moment = T::BlockNumber;
//
//     fn set_lock(
//         id: LockIdentifier,
//         who: &T::AccountId,
//         amount: T::Balance,
//         until: T::BlockNumber,
//         reasons: WithdrawReasons,
//     ) {
//         Self::set_lock(id, who, amount, until, reasons)
//     }
//
//     fn extend_lock(
//         id: LockIdentifier,
//         who: &T::AccountId,
//         amount: T::Balance,
//         until: T::BlockNumber,
//         reasons: WithdrawReasons,
//     ) {
//         Self::extend_lock(id, who, amount, until, reasons)
//     }
//
//     fn remove_lock(id: LockIdentifier, who: &T::AccountId) {
//         Self::remove_lock(id, who)
//     }
// }

// pub type StakingAssetCurrency<T> = AssetCurrency<T, StakingAssetIdProvider<T>>;
// pub type SpendingAssetCurrency<T> = AssetCurrency<T, SpendingAssetIdProvider<T>>;
