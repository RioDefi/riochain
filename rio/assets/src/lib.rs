#![cfg_attr(not(feature = "std"), no_std)]

pub mod types;
mod weight_info;

use codec::{Decode, Encode};

use sp_runtime::{
    traits::{
        AtLeast32BitUnsigned, CheckedAdd, CheckedSub, MaybeSerializeDeserialize, Member,
        Saturating, StaticLookup, Zero,
    },
    DispatchError, DispatchResult, RuntimeDebug,
};
use sp_std::{
    collections::btree_map::BTreeMap,
    convert::{TryFrom, TryInto},
    prelude::*,
    result,
};

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure, IterableStorageMap, Parameter,
    StorageMap,
};
use frame_system::{ensure_root, ensure_signed};

use rio_support::info;

use orml_traits::{
    arithmetic::{self, Signed},
    BalanceStatus, LockIdentifier, MultiCurrency, MultiCurrencyExtended, MultiLockableCurrency,
    MultiReservableCurrency, OnReceived,
};

use crate::types::TotalAssetInfo;
pub use types::{AssetInfo, Chain, Restriction, Restrictions};
pub use weight_info::WeightInfo;

/// The module's configuration trait.
pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

    /// The balance type
    type Balance: Parameter
        + Member
        + AtLeast32BitUnsigned
        + Default
        + Copy
        + MaybeSerializeDeserialize;

    /// The amount type, should be signed version of `Balance`
    type Amount: Signed
        + TryInto<Self::Balance>
        + TryFrom<Self::Balance>
        + Parameter
        + Member
        + arithmetic::SimpleArithmetic
        + Default
        + Copy
        + MaybeSerializeDeserialize;

    /// The currency ID type
    type CurrencyId: Parameter + Member + Copy + MaybeSerializeDeserialize + Ord;

    /// Hook when some fund is deposited into an account
    type OnReceived: OnReceived<Self::AccountId, Self::CurrencyId, Self::Balance>;

    type WeightInfo: WeightInfo;
}

/// A single lock on a balance. There can be many of these on an account and
/// they "overlap", so the same balance is frozen by multiple locks.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct BalanceLock<Balance> {
    /// An identifier for this lock. Only one lock may be in existence for each
    /// identifier.
    pub id: LockIdentifier,
    /// The amount which the free balance may not drop below when this lock is
    /// in effect.
    pub amount: Balance,
}

/// balance information for an account.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug)]
pub struct AccountData<Balance> {
    /// Non-reserved part of the balance. There may still be restrictions on
    /// this, but it is the total pool what may in principle be transferred,
    /// reserved.
    ///
    /// This is the only balance that matters in terms of most operations on
    /// tokens.
    pub free: Balance,
    /// Balance which is reserved and may not be used at all.
    ///
    /// This can still get slashed, but gets slashed last of all.
    ///
    /// This balance is a 'reserve' balance that other subsystems use in order
    /// to set aside tokens that are still 'owned' by the account holder, but
    /// which are suspendable.
    pub reserved: Balance,
    /// The amount that `free` may not drop below when withdrawing.
    pub frozen: Balance,
}

impl<Balance: Saturating + Copy + Ord> AccountData<Balance> {
    /// The amount that this account's free balance may not be reduced beyond.
    fn frozen(&self) -> Balance {
        self.frozen
    }
    /// The total balance in this account including any that is reserved and
    /// ignoring any frozen.
    fn total(&self) -> Balance {
        self.free.saturating_add(self.reserved)
    }
}

decl_error! {
    /// Error for the generic-asset module.
    pub enum Error for Module<T: Trait> {
        /// The balance is too low
        BalanceTooLow,
        /// This operation will cause total issuance to overflow
        TotalIssuanceOverflow,
        /// Cannot convert Amount into Balance type
        AmountIntoBalanceFailed,
        /// Failed because liquidity restrictions due to locking
        LiquidityRestrictions,
        /// asset already exist
        ExistedAsset,
        /// asset not exist
        NotExistedAsset,
        /// invalid asset
        InvalidAsset,
        /// invalid asset info when create asset
        InvalidAssetInfo,
        /// The asset is restricted by this action.
        RestrictedAction,
        /// Symbol, Name or Desc too long
        TextTooLong,
    }
}

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Trait>::AccountId,
        <T as Trait>::CurrencyId,
        <T as Trait>::Balance
    {
        /// Token transfer success. [currency_id, from, to, amount]
        Transferred(CurrencyId, AccountId, AccountId, Balance),
        /// Asset created (currency_id, creator, asset_options).
        Created(CurrencyId),
        /// update asset restrictions
        UpdateAssetRestriction(CurrencyId, Restrictions),
        ///
        Revoke(CurrencyId),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as RioAssets {
        /// Restrictions means this asset can't do something
        pub AssetRestrictions get(fn asset_restrictions):
            map hasher(twox_64_concat) T::CurrencyId => Restrictions;

        /// "Symbols" can only keep Vec<u8>, and utf8 safety is totally on the client side
        pub AssetInfos get(fn asset_info_of):
            map hasher(twox_64_concat) T::CurrencyId => Option<AssetInfo>;

        pub Online get(fn online): map hasher(twox_64_concat) T::CurrencyId => bool;

        /// The total issuance of a token type.
        pub TotalIssuance get(fn total_issuance): map hasher(twox_64_concat) T::CurrencyId => T::Balance;

        /// Any liquidity locks of a token type under an account.
        /// NOTE: Should only be accessed when setting, changing and freeing a lock.
        pub Locks get(fn locks): double_map hasher(blake2_128_concat) T::AccountId, hasher(twox_64_concat) T::CurrencyId => Vec<BalanceLock<T::Balance>>;

        /// The balance of a token type under an account.
        ///
        /// NOTE: If the total is ever zero, decrease account ref account.
        ///
        /// NOTE: This is only used in the case that this module is used to store balances.
        pub Accounts get(fn accounts): double_map hasher(blake2_128_concat) T::AccountId, hasher(twox_64_concat) T::CurrencyId => AccountData<T::Balance>;
    }
    add_extra_genesis {
        config(init): Vec<(T::CurrencyId, AssetInfo, Restrictions, Vec<(T::AccountId, T::Balance)>)>;
        build(|config: &GenesisConfig<T>| {
            for (currency_id, info, restrictions, endowed) in config.init.iter() {
                let currency_id = *currency_id;
                // create asset
                <Module<T>>::create_asset(
                    currency_id,
                    info.clone(),
                ).expect("create_asset must success in genesis");
                // restriction
                AssetRestrictions::<T>::insert(currency_id, restrictions);
                // endowed
                endowed.iter().for_each(|(account_id, initial_balance)| {
                    <Accounts<T>>::mutate(account_id, currency_id, |account_data| account_data.free = *initial_balance)
                });
                let total = endowed.iter().map(|(_, b)| b).fold(Zero::zero(), |acc: T::Balance, initial_balance| {
                    acc + *initial_balance
                });
                TotalIssuance::<T>::insert(currency_id, total);
            }
        });
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Transfer some balance to another account.
        ///
        /// The dispatch origin for this call must be `Signed` by the transactor.
        ///
        /// # <weight>
        /// - Complexity: `O(1)`
        /// - Db reads: 2 * `Accounts`
        /// - Db writes: 2 * `Accounts`
        /// -------------------
        /// Base Weight: 26.65 µs
        /// # </weight>
        #[weight = T::WeightInfo::transfer()]
        pub fn transfer(
            origin,
            dest: <T::Lookup as StaticLookup>::Source,
            currency_id: T::CurrencyId,
            #[compact] amount: T::Balance,
        ) {
            let from = ensure_signed(origin)?;
            let to = T::Lookup::lookup(dest)?;
            <Self as MultiCurrency<_>>::transfer(currency_id, &from, &to, amount)?;

            Self::deposit_event(RawEvent::Transferred(currency_id, from, to, amount));
        }

        /// Transfer all remaining balance to the given account.
        ///
        /// The dispatch origin for this call must be `Signed` by the transactor.
        ///
        /// # <weight>
        /// - Complexity: `O(1)`
        /// - Db reads: 2 * `Accounts`
        /// - Db writes: 2 * `Accounts`
        /// -------------------
        /// Base Weight: 26.99 µs
        /// # </weight>
        #[weight = T::WeightInfo::transfer_all()]
        pub fn transfer_all(
            origin,
            dest: <T::Lookup as StaticLookup>::Source,
            currency_id: T::CurrencyId,
        ) {
            let from = ensure_signed(origin)?;
            let to = T::Lookup::lookup(dest)?;
            let balance = <Self as MultiCurrency<T::AccountId>>::free_balance(currency_id, &from);
            <Self as MultiCurrency<T::AccountId>>::transfer(currency_id, &from, &to, balance)?;

            Self::deposit_event(RawEvent::Transferred(currency_id, from, to, balance));
        }

        /// create a new asset with full permissions granted to whoever make the call
        /// *sudo or proposal approved only*
        #[weight = T::WeightInfo::create()]
        pub fn create(
            origin,
            currency_id: T::CurrencyId,
            asset_info: AssetInfo,
        ) -> DispatchResult {
            ensure_root(origin)?;

            Self::create_asset(currency_id, asset_info)?;

            Ok(())
        }

        #[weight = T::WeightInfo::update_asset_info()]
        pub fn update_asset_info(
            origin,
            currency_id: T::CurrencyId,
            asset_info: AssetInfo,
        ) -> DispatchResult {
            ensure_root(origin)?;
            ensure!(Self::asset_info_of(currency_id).is_some(), Error::<T>::NotExistedAsset);
            AssetInfos::<T>::insert(currency_id, asset_info);
            Ok(())
        }

        #[weight = T::WeightInfo::update_restriction()]
        pub fn update_restriction(origin, currency_id: T::CurrencyId, restrictions: Restrictions) {
            ensure_root(origin)?;
            <AssetRestrictions<T>>::insert(currency_id, restrictions);
            Self::deposit_event(RawEvent::UpdateAssetRestriction(currency_id, restrictions));
        }

        #[weight = T::WeightInfo::offline_asset()]
        pub fn offline_asset(origin, currency_id: T::CurrencyId) {
            ensure_root(origin)?;
            Online::<T>::remove(currency_id);
            Self::deposit_event(RawEvent::Revoke(currency_id));
        }

        #[weight = T::WeightInfo::online_asset()]
        pub fn online_asset(origin, currency_id: T::CurrencyId) -> DispatchResult {
            ensure_root(origin)?;
            ensure!(Self::asset_info_of(currency_id).is_some(), Error::<T>::NotExistedAsset);
            Online::<T>::insert(currency_id, true);
            Self::deposit_event(RawEvent::Created(currency_id));
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    // PUBLIC IMMUTABLES

    /// check the currency_id is existed
    #[inline]
    pub fn get_asset(currency_id: &T::CurrencyId) -> result::Result<AssetInfo, DispatchError> {
        if let Some(asset) = Self::asset_info_of(currency_id) {
            if Self::online(currency_id) {
                Ok(asset)
            } else {
                Err(Error::<T>::InvalidAsset)?
            }
        } else {
            Err(Error::<T>::NotExistedAsset)?
        }
    }

    /// Creates an asset.
    ///
    /// # Arguments
    /// * `currency_id`: An ID of a reserved asset.
    ///
    pub fn create_asset(currency_id: T::CurrencyId, asset_info: AssetInfo) -> DispatchResult {
        // make sure the asset id is not exist
        ensure!(
            Self::asset_info_of(&currency_id).is_none(),
            Error::<T>::ExistedAsset
        );

        ensure!(
            asset_info.symbol.len() <= rio_protocol::ASSET_SYMBOL_LEN,
            Error::<T>::InvalidAssetInfo
        );
        ensure!(
            asset_info.name.len() <= rio_protocol::ASSET_NAME_LEN,
            Error::<T>::InvalidAssetInfo
        );
        ensure!(
            asset_info.desc.len() <= rio_protocol::ASSET_DESC_LEN,
            Error::<T>::InvalidAssetInfo
        );
        ensure!(asset_info.decimals != 0, Error::<T>::InvalidAssetInfo);

        info!(
            "[create_asset]|currency_id:{:?}, symbol:{:?}",
            currency_id, asset_info.symbol
        );

        <AssetInfos<T>>::insert(currency_id, asset_info);
        Online::<T>::insert(currency_id, true);

        Self::deposit_event(RawEvent::Created(currency_id));

        Ok(())
    }

    pub fn total_asset_infos() -> BTreeMap<T::CurrencyId, TotalAssetInfo<T::Balance>> {
        AssetInfos::<T>::iter()
            .map(|(id, info)| {
                (
                    id,
                    TotalAssetInfo {
                        info,
                        balance: Self::total_issuance(id),
                        is_online: Self::online(id),
                        restrictions: Self::asset_restrictions(id),
                    },
                )
            })
            .collect()
    }
}

impl<T: Trait> Module<T> {
    /// Set free balance of `who` to a new value.
    ///
    /// Note this will not maintain total issuance.
    fn set_free_balance(currency_id: T::CurrencyId, who: &T::AccountId, balance: T::Balance) {
        <Accounts<T>>::mutate(who, currency_id, |account_data| account_data.free = balance);
    }

    /// Set reserved balance of `who` to a new value, meanwhile enforce
    /// existential rule.
    ///
    /// Note this will not maintain total issuance, and the caller is expected
    /// to do it.
    fn set_reserved_balance(currency_id: T::CurrencyId, who: &T::AccountId, balance: T::Balance) {
        <Accounts<T>>::mutate(who, currency_id, |account_data| {
            account_data.reserved = balance
        });
    }

    /// Update the account entry for `who` under `currency_id`, given the locks.
    fn update_locks(
        currency_id: T::CurrencyId,
        who: &T::AccountId,
        locks: &[BalanceLock<T::Balance>],
    ) {
        // update account data
        <Accounts<T>>::mutate(who, currency_id, |account_data| {
            account_data.frozen = Zero::zero();
            for lock in locks.iter() {
                account_data.frozen = account_data.frozen.max(lock.amount);
            }
        });

        // update locks
        let existed = <Locks<T>>::contains_key(who, currency_id);
        if locks.is_empty() {
            <Locks<T>>::remove(who, currency_id);
            if existed {
                // decrease account ref count when destruct lock
                frame_system::Module::<T>::dec_ref(who);
            }
        } else {
            <Locks<T>>::insert(who, currency_id, locks);
            if !existed {
                // increase account ref count when initialize lock
                frame_system::Module::<T>::inc_ref(who);
            }
        }
    }
}

impl<T: Trait> MultiCurrency<T::AccountId> for Module<T> {
    type CurrencyId = T::CurrencyId;
    type Balance = T::Balance;

    fn total_issuance(currency_id: Self::CurrencyId) -> Self::Balance {
        <TotalIssuance<T>>::get(currency_id)
    }

    fn total_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
        Self::accounts(who, currency_id).total()
    }

    fn free_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
        Self::accounts(who, currency_id).free
    }

    // Ensure that an account can withdraw from their free balance given any
    // existing withdrawal restrictions like locks and vesting balance.
    // Is a no-op if amount to be withdrawn is zero.
    fn ensure_can_withdraw(
        currency_id: Self::CurrencyId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> DispatchResult {
        let _ = Self::get_asset(&currency_id)?;
        Self::can_do(&currency_id, Restriction::Withdrawable)?;

        if amount.is_zero() {
            return Ok(());
        }

        let new_balance = Self::free_balance(currency_id, who)
            .checked_sub(&amount)
            .ok_or(Error::<T>::BalanceTooLow)?;
        ensure!(
            new_balance >= Self::accounts(who, currency_id).frozen(),
            Error::<T>::LiquidityRestrictions
        );
        Ok(())
    }

    /// Transfer some free balance from `from` to `to`.
    /// Is a no-op if value to be transferred is zero or the `from` is the same
    /// as `to`.
    fn transfer(
        currency_id: Self::CurrencyId,
        from: &T::AccountId,
        to: &T::AccountId,
        amount: Self::Balance,
    ) -> DispatchResult {
        let _ = Self::get_asset(&currency_id)?;
        Self::can_do(&currency_id, Restriction::Transferable)?;

        if amount.is_zero() || from == to {
            return Ok(());
        }

        Self::ensure_can_withdraw(currency_id, from, amount)?;

        let from_balance = Self::free_balance(currency_id, from);
        let to_balance = Self::free_balance(currency_id, to);
        Self::set_free_balance(currency_id, from, from_balance - amount);
        Self::set_free_balance(currency_id, to, to_balance + amount);
        T::OnReceived::on_received(to, currency_id, amount);

        Ok(())
    }

    /// Deposit some `amount` into the free balance of account `who`.
    ///
    /// Is a no-op if the `amount` to be deposited is zero.
    fn deposit(
        currency_id: Self::CurrencyId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> DispatchResult {
        let _ = Self::get_asset(&currency_id)?;
        Self::can_do(&currency_id, Restriction::Depositable)?;

        if amount.is_zero() {
            return Ok(());
        }

        let new_total = Self::total_issuance(currency_id)
            .checked_add(&amount)
            .ok_or(Error::<T>::TotalIssuanceOverflow)?;
        <TotalIssuance<T>>::insert(currency_id, new_total);
        Self::set_free_balance(
            currency_id,
            who,
            Self::free_balance(currency_id, who) + amount,
        );
        T::OnReceived::on_received(who, currency_id, amount);

        Ok(())
    }

    fn withdraw(
        currency_id: Self::CurrencyId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> DispatchResult {
        let _ = Self::get_asset(&currency_id)?;
        Self::can_do(&currency_id, Restriction::Withdrawable)?;

        if amount.is_zero() {
            return Ok(());
        }
        Self::ensure_can_withdraw(currency_id, who, amount)?;

        <TotalIssuance<T>>::mutate(currency_id, |v| *v -= amount);
        Self::set_free_balance(
            currency_id,
            who,
            Self::free_balance(currency_id, who) - amount,
        );

        Ok(())
    }

    // Check if `value` amount of free balance can be slashed from `who`.
    fn can_slash(currency_id: Self::CurrencyId, who: &T::AccountId, value: Self::Balance) -> bool {
        if Self::get_asset(&currency_id).is_err()
            || Self::can_do(&currency_id, Restriction::Slashable).is_err()
        {
            return false;
        }

        if value.is_zero() {
            return true;
        }
        Self::free_balance(currency_id, who) >= value
    }

    /// Is a no-op if `value` to be slashed is zero.
    ///
    /// NOTE: `slash()` prefers free balance, but assumes that reserve balance
    /// can be drawn from in extreme circumstances. `can_slash()` should be used
    /// prior to `slash()` to avoid having to draw from reserved funds, however
    /// we err on the side of punishment if things are inconsistent
    /// or `can_slash` wasn't used appropriately.
    fn slash(
        currency_id: Self::CurrencyId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> Self::Balance {
        if amount.is_zero() {
            return amount;
        }

        let account = Self::accounts(who, currency_id);
        let free_slashed_amount = account.free.min(amount);
        let mut remaining_slash = amount - free_slashed_amount;

        // slash free balance
        if !free_slashed_amount.is_zero() {
            Self::set_free_balance(currency_id, who, account.free - free_slashed_amount);
        }

        // slash reserved balance
        if !remaining_slash.is_zero() {
            let reserved_slashed_amount = account.reserved.min(remaining_slash);
            remaining_slash -= reserved_slashed_amount;
            Self::set_reserved_balance(
                currency_id,
                who,
                account.reserved - reserved_slashed_amount,
            );
        }

        <TotalIssuance<T>>::mutate(currency_id, |v| *v -= amount - remaining_slash);
        remaining_slash
    }
}

impl<T: Trait> MultiCurrencyExtended<T::AccountId> for Module<T> {
    type Amount = T::Amount;

    fn update_balance(
        currency_id: Self::CurrencyId,
        who: &T::AccountId,
        by_amount: Self::Amount,
    ) -> DispatchResult {
        if by_amount.is_zero() {
            return Ok(());
        }

        let by_balance = TryInto::<Self::Balance>::try_into(by_amount.abs())
            .map_err(|_| Error::<T>::AmountIntoBalanceFailed)?;
        if by_amount.is_positive() {
            Self::deposit(currency_id, who, by_balance)
        } else {
            Self::withdraw(currency_id, who, by_balance)
        }
    }
}

impl<T: Trait> MultiLockableCurrency<T::AccountId> for Module<T> {
    type Moment = T::BlockNumber;

    // Set a lock on the balance of `who` under `currency_id`.
    // Is a no-op if lock amount is zero.
    fn set_lock(
        lock_id: LockIdentifier,
        currency_id: Self::CurrencyId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) {
        if amount.is_zero() {
            return;
        }
        let mut new_lock = Some(BalanceLock {
            id: lock_id,
            amount,
        });
        let mut locks = Self::locks(who, currency_id)
            .into_iter()
            .filter_map(|lock| {
                if lock.id == lock_id {
                    new_lock.take()
                } else {
                    Some(lock)
                }
            })
            .collect::<Vec<_>>();
        if let Some(lock) = new_lock {
            locks.push(lock)
        }
        Self::update_locks(currency_id, who, &locks[..]);
    }

    // Extend a lock on the balance of `who` under `currency_id`.
    // Is a no-op if lock amount is zero
    fn extend_lock(
        lock_id: LockIdentifier,
        currency_id: Self::CurrencyId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) {
        if amount.is_zero() {
            return;
        }
        let mut new_lock = Some(BalanceLock {
            id: lock_id,
            amount,
        });
        let mut locks = Self::locks(who, currency_id)
            .into_iter()
            .filter_map(|lock| {
                if lock.id == lock_id {
                    new_lock.take().map(|nl| BalanceLock {
                        id: lock.id,
                        amount: lock.amount.max(nl.amount),
                    })
                } else {
                    Some(lock)
                }
            })
            .collect::<Vec<_>>();
        if let Some(lock) = new_lock {
            locks.push(lock)
        }
        Self::update_locks(currency_id, who, &locks[..]);
    }

    fn remove_lock(lock_id: LockIdentifier, currency_id: Self::CurrencyId, who: &T::AccountId) {
        let mut locks = Self::locks(who, currency_id);
        locks.retain(|lock| lock.id != lock_id);
        Self::update_locks(currency_id, who, &locks[..]);
    }
}

impl<T: Trait> MultiReservableCurrency<T::AccountId> for Module<T> {
    /// Check if `who` can reserve `value` from their free balance.
    ///
    /// Always `true` if value to be reserved is zero.
    fn can_reserve(
        currency_id: Self::CurrencyId,
        who: &T::AccountId,
        value: Self::Balance,
    ) -> bool {
        if Self::get_asset(&currency_id).is_err()
            || Self::can_do(&currency_id, Restriction::Reservable).is_err()
        {
            return false;
        }

        if value.is_zero() {
            return true;
        }
        Self::ensure_can_withdraw(currency_id, who, value).is_ok()
    }

    /// Slash from reserved balance, returning any amount that was unable to be
    /// slashed.
    ///
    /// Is a no-op if the value to be slashed is zero.
    fn slash_reserved(
        currency_id: Self::CurrencyId,
        who: &T::AccountId,
        value: Self::Balance,
    ) -> Self::Balance {
        if value.is_zero() {
            return Zero::zero();
        }

        let reserved_balance = Self::reserved_balance(currency_id, who);
        let actual = reserved_balance.min(value);
        Self::set_reserved_balance(currency_id, who, reserved_balance - actual);
        <TotalIssuance<T>>::mutate(currency_id, |v| *v -= actual);
        value - actual
    }

    fn reserved_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
        Self::accounts(who, currency_id).reserved
    }

    /// Move `value` from the free balance from `who` to their reserved balance.
    ///
    /// Is a no-op if value to be reserved is zero.
    fn reserve(
        currency_id: Self::CurrencyId,
        who: &T::AccountId,
        value: Self::Balance,
    ) -> DispatchResult {
        let _ = Self::get_asset(&currency_id)?;
        Self::can_do(&currency_id, Restriction::Reservable)?;

        if value.is_zero() {
            return Ok(());
        }
        Self::ensure_can_withdraw(currency_id, who, value)?;

        let account = Self::accounts(who, currency_id);
        Self::set_free_balance(currency_id, who, account.free - value);
        Self::set_reserved_balance(currency_id, who, account.reserved + value);
        Ok(())
    }

    /// Unreserve some funds, returning any amount that was unable to be
    /// unreserved.
    ///
    /// Is a no-op if the value to be unreserved is zero.
    fn unreserve(
        currency_id: Self::CurrencyId,
        who: &T::AccountId,
        value: Self::Balance,
    ) -> Self::Balance {
        if Self::get_asset(&currency_id).is_err()
            || Self::can_do(&currency_id, Restriction::Unreservable).is_err()
        {
            return Zero::zero();
        }

        if value.is_zero() {
            return Zero::zero();
        }

        let account = Self::accounts(who, currency_id);
        let actual = account.reserved.min(value);
        Self::set_reserved_balance(currency_id, who, account.reserved - actual);
        Self::set_free_balance(currency_id, who, account.free + actual);
        T::OnReceived::on_received(who, currency_id, actual);
        value - actual
    }

    /// Move the reserved balance of one account into the balance of another,
    /// according to `status`.
    ///
    /// Is a no-op if:
    /// - the value to be moved is zero; or
    /// - the `slashed` id equal to `beneficiary` and the `status` is
    ///   `Reserved`.
    fn repatriate_reserved(
        currency_id: Self::CurrencyId,
        slashed: &T::AccountId,
        beneficiary: &T::AccountId,
        value: Self::Balance,
        status: BalanceStatus,
    ) -> result::Result<Self::Balance, DispatchError> {
        if value.is_zero() {
            return Ok(Zero::zero());
        }

        if slashed == beneficiary {
            return match status {
                BalanceStatus::Free => Ok(Self::unreserve(currency_id, slashed, value)),
                BalanceStatus::Reserved => {
                    Ok(value.saturating_sub(Self::reserved_balance(currency_id, slashed)))
                }
            };
        }

        let from_account = Self::accounts(slashed, currency_id);
        let to_account = Self::accounts(beneficiary, currency_id);
        let actual = from_account.reserved.min(value);
        match status {
            BalanceStatus::Free => {
                Self::set_free_balance(currency_id, beneficiary, to_account.free + actual);
                T::OnReceived::on_received(beneficiary, currency_id, actual);
            }
            BalanceStatus::Reserved => {
                Self::set_reserved_balance(currency_id, beneficiary, to_account.reserved + actual);
            }
        }
        Self::set_reserved_balance(currency_id, slashed, from_account.reserved - actual);
        Ok(value - actual)
    }
}
