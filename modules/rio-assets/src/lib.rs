#![cfg_attr(not(feature = "std"), no_std)]

mod mock;
mod tests;

pub mod attributes;
pub mod currency;
pub mod imbalances;
pub mod permissions;
pub mod protocol;
pub mod traits;

use rstd::fmt::Debug;
use rstd::{collections::btree_map::BTreeMap, result, vec::Vec};
use sp_runtime::traits::{
    CheckedAdd, CheckedSub, MaybeSerializeDeserialize, Member, Saturating, SimpleArithmetic, Zero,
};
use support::{
    decl_event, decl_module, decl_storage, dispatch::Result, ensure, weights::SimpleDispatchInfo,
};
use support::{
    dispatch,
    traits::{Currency, ExistenceRequirement, Imbalance, WithdrawReason, WithdrawReasons},
    Parameter,
};
use system::{ensure_root, ensure_signed};

use rio_support::{debug, info};

pub use attributes::{AssetOptions, Owner};
use currency::{FromCurrency, FromToContext, ToCurrency};
pub use permissions::{
    PermissionLatest, PermissionType, PermissionVersions, PermissionsV1, Restriction,
};

use traits::*;

/// The module's configuration trait.
pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type Balance: Parameter
        + Member
        + SimpleArithmetic
        + Default
        + Copy
        + MaybeSerializeDeserialize
        + Debug;
    type AssetId: Parameter + Member + SimpleArithmetic + Default + Copy;

    type RootKey: RootKeyProvider<AccountId = Self::AccountId>;

    // defines all the event hooks for operations on Assets
    type BeforeAssetTransfer: BeforeAssetTransfer<Self::AssetId, Self::AccountId, Self::Balance>;
    type BeforeAssetCreate: BeforeAssetCreate<Self::AssetId>;
    type BeforeAssetMint: BeforeAssetMint<Self::AssetId, Self::AccountId, Self::Balance>;
    type BeforeAssetBurn: BeforeAssetBurn<Self::AssetId, Self::AccountId, Self::Balance>;

    type OnAssetTransfer: OnAssetTransfer<Self::AssetId, Self::AccountId, Self::Balance>;
    type OnAssetCreate: OnAssetCreate<Self::AssetId>;
    type OnAssetMint: OnAssetMint<Self::AssetId, Self::AccountId, Self::Balance>;
    type OnAssetBurn: OnAssetBurn<Self::AssetId, Self::AccountId, Self::Balance>;
}

pub trait Subtrait: system::Trait {
    type Balance: Parameter
        + Member
        + SimpleArithmetic
        + Default
        + Copy
        + MaybeSerializeDeserialize
        + Debug;
    type AssetId: Parameter + Member + SimpleArithmetic + Default + Copy;
}

impl<T: Trait> Subtrait for T {
    type Balance = T::Balance;
    type AssetId = T::AssetId;
}

pub type NativeAsset<T> = currency::AssetCurrency<T, currency::RFUELProvider<T>>;

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        <T as Trait>::Balance,
        <T as Trait>::AssetId,
        AssetOptions = AssetOptions<<T as Trait>::Balance, <T as system::Trait>::AccountId>
    {
        /// Asset created (asset_id, creator, asset_options).
        Created(AssetId, AccountId, AssetOptions),
        /// Asset transfer succeeded (asset_id, from, to, amount).
        Transferred(AssetId, AccountId, AccountId, Balance),
        /// Asset permission updated (asset_id, new_permissions).
        PermissionUpdated(AssetId, PermissionLatest<AccountId>),
        /// New asset minted (asset_id, account, amount).
        Minted(AssetId, AccountId, Balance),
        /// Asset burned (asset_id, account, amount).
        Burned(AssetId, AccountId, Balance),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as RioAssets {
        /// Total issuance of a given asset.
        pub TotalIssuance get(fn total_issuance): map T::AssetId => T::Balance;

        /// The free balance of a given asset under an account.
        pub FreeBalance: double_map T::AssetId, twox_128(T::AccountId) => T::Balance;

        /// The reserved balance of a given asset under an account. would be dropped in future
        pub ReservedBalance: double_map T::AssetId, twox_128(T::AccountId) => T::Balance;

        /// Permission options for a given asset.
        pub Permissions get(fn get_permission): map T::AssetId => PermissionVersions<T::AccountId>;

        /// Restrictions means this asset can't do something
        pub Restrictions get(fn get_restrictions): map T::AssetId => BTreeMap<Restriction, ()>;

        // /// Any liquidity locks on some account balances.
        // pub Locks get(fn locks): map T::AccountId => Vec<BalanceLock<T::Balance, T::BlockNumber>>;

        // /// The identity of the asset which is the one that is designated for the chain's staking system.
        // pub StakingAssetId get(fn staking_asset_id) config(): T::AssetId;
        //
        // /// The identity of the asset which is the one that is designated for paying the chain's transaction fee.
        // pub SpendingAssetId get(fn spending_asset_id) config(): T::AssetId;

        /// "Symbols" can only keep Vec<u8>, and utf8 safty is totally on the client side
        pub Symbols get(symbols): linked_map hasher(blake2_256) T::AssetId => Option<Vec<u8>>;

        // hack for AssetCurrency
        /// FromId and ToId would not store in blockchain, it just use for FromToContext
        FromId get(fn get_from_id): Option<T::AssetId>;
        /// FromId and ToId would not store in blockchain, it just use for FromToContext
        ToId get(fn get_to_id): Option<T::AssetId>;
    }

    add_extra_genesis {
        config(symbols): Vec<(T::AssetId, Vec<u8>, Vec<Restriction>, Vec<(T::AccountId, T::Balance)>)>;
        config(root): T::AccountId;
        build(|config: &GenesisConfig<T>| {
            let options = AssetOptions {
                initial_issuance: T::Balance::from(0),
                permissions: PermissionLatest {
                    update: Owner::Address(config.root.clone()),
                    mint: Owner::Address(config.root.clone()),
                    burn: Owner::Address(config.root.clone()),
                },
            };

            // by default, create assets in pallet-generic-asset according to token symbol configs in this module
            for (id, symbol, restrictions, endowed) in config.symbols.iter() {
                let id = *id;
                <Module<T>>::create_asset(id, symbol.clone(), None, options.clone()).unwrap();

                restrictions.iter().for_each(|forbiden| {
                    let origin = system::RawOrigin::Root;
                    <Module<T>>::update_restriction(origin.into(), id, *forbiden, true).unwrap();
                });

                endowed.iter().for_each(|(account_id, balance)| {
                    <FreeBalance<T>>::insert(id, account_id, balance);
                });
                let sum: T::Balance = endowed.iter().fold(Zero::zero(), |sum, val| sum + val.1);
                <TotalIssuance<T>>::insert(id, sum);
            }
        });
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing events
        // this is needed only if you are using events in your module
        pub fn deposit_event() = default;

        /// create a new asset with full permissions granted to whoever make the call
        /// *sudo or proposal approved only*
        #[weight = SimpleDispatchInfo::FreeOperational]
        pub fn create(origin, #[compact] initial_balance: T::Balance, #[compact] asset_id: T::AssetId, symbol: Vec<u8>) -> Result {
            ensure_root(origin)?;

            let root_account_id = T::RootKey::root_key();

            // by default, only root can create assets, so root should be granted all permissions for the assets
            let options = AssetOptions {
                initial_issuance:initial_balance,
                permissions: PermissionLatest {
                    update: Owner::Address(root_account_id.clone()),
                    mint: Owner::Address(root_account_id.clone()),
                    burn: Owner::Address(root_account_id.clone()),
                },
            };

            Self::create_asset(asset_id, symbol, Some(root_account_id), options)?;

            Ok(())
        }

        /// generic_asset<T>::make_transfer_with_event delegation
        /// wrap 2 hooks around "make_transfer_with_event": T::BeforeAssetTransfer & T::OnAssetTransfer
        #[weight = SimpleDispatchInfo::FixedNormal(0)]
        pub fn transfer(origin, #[compact] asset_id: T::AssetId, to: T::AccountId, #[compact] amount: T::Balance) -> Result {
            let o = ensure_signed(origin)?;
            T::BeforeAssetTransfer::before_asset_transfer(&asset_id, &o, &to, &amount)?;

            Self::make_transfer_with_event(&asset_id, &o, &to, amount)?;
            // ignore the err
            T::OnAssetTransfer::on_asset_transfer(&asset_id, &o, &to, &amount).unwrap_or_default();
            Ok(())
        }

        // generic_asset<T>::update_permission delegation
        pub fn update_permission(origin, #[compact] asset_id: T::AssetId, new_permission: PermissionLatest<T::AccountId>) -> Result {
            ensure_root(origin)?;

            let permissions: PermissionVersions<T::AccountId> = new_permission.into();

            let root_account_id = T::RootKey::root_key();
            if Self::check_permission(&asset_id, &root_account_id.into(), &PermissionType::Update) {
                <Permissions<T>>::insert(asset_id, &permissions);

                Self::deposit_event(RawEvent::PermissionUpdated(asset_id, permissions.into()));

                Ok(())
            } else {
                Err("Origin does not have enough permission to update permissions.")
            }
        }

        pub fn update_restriction(origin, #[compact] asset_id: T::AssetId, restriction: Restriction, not_allow: bool) {
            ensure_root(origin)?;
            <Restrictions<T>>::mutate(asset_id, |map| {
                if not_allow == true {
                    map.insert(restriction, ());
                } else {
                    let _ = map.remove(&restriction);
                }
            });
        }

        /// generic_asset<T>::mint delegation
        #[weight = SimpleDispatchInfo::FreeOperational]
        pub fn mint(origin, #[compact] asset_id: T::AssetId, to: T::AccountId, #[compact] amount: T::Balance) -> Result {
            ensure_root(origin)?;
            let root_account_id = T::RootKey::root_key();
            T::BeforeAssetMint::before_asset_mint(&asset_id, &to, &amount)?;

            Self::mint_free(&asset_id, &root_account_id, &to, &amount)?;
            Self::deposit_event(RawEvent::Minted(asset_id, to.clone(), amount));

            // ignore the err
            T::OnAssetMint::on_asset_mint(&asset_id, &to, &amount).unwrap_or_default();
            Ok(())
        }

        /// generic_asset<T>::burn delegation
        #[weight = SimpleDispatchInfo::FreeOperational]
        pub fn burn(origin, #[compact] asset_id: T::AssetId, to: T::AccountId, #[compact] amount:T::Balance) -> Result {
            ensure_root(origin)?;
            let root_account_id = T::RootKey::root_key();
            T::BeforeAssetBurn::before_asset_burn(&asset_id, &to, &amount)?;

            Self::burn_free(&asset_id, &root_account_id, &to, &amount)?;
            Self::deposit_event(RawEvent::Burned(asset_id, to.clone(), amount));

            // ignore the err
            T::OnAssetBurn::on_asset_burn(&asset_id, &to, &amount).unwrap_or_default();
            Ok(())
        }

        // #[weight = SimpleDispatchInfo::FreeOperational]
        // pub fn create_reserved(origin, asset_id: T::AssetId, options: AssetOptions<T::Balance, T::AccountId>) -> Result {
        //     ensure_root(origin)?;
        //     let root_account_id = T::RootKey::root_key();
        //     Self::create_asset(Some(asset_id), None, options)
        // }

        /// generic_asset<T>::burn delegation
        #[weight = SimpleDispatchInfo::FreeOperational]
        pub fn set_reserve(origin, #[compact] asset_id: T::AssetId, who: T::AccountId, #[compact] amount:T::Balance) -> Result {
            ensure_root(origin)?;

            Self::reserve(&asset_id, &who, amount)?;

            Ok(())
        }

        #[weight = SimpleDispatchInfo::FreeOperational]
        pub fn set_unreserve(origin, #[compact] asset_id: T::AssetId, who: T::AccountId, #[compact] amount:T::Balance) -> Result {
            ensure_root(origin)?;
            Self::unreserve(&asset_id, &who, amount);
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    // PUBLIC IMMUTABLES
    // todo Deprecated, remove after refactor
    /// check if the asset id is already used
    pub fn asset_exists(asset_id: &T::AssetId) -> bool {
        <Symbols<T>>::exists(asset_id)
    }
    /// check the asset id is not existed
    #[inline]
    pub fn should_not_exist(asset_id: &T::AssetId) -> dispatch::Result {
        ensure!(Self::symbols(asset_id).is_none(), "AssetId is exist.");
        Ok(())
    }
    /// check the asset id is existed
    #[inline]
    pub fn should_exist(asset_id: &T::AssetId) -> dispatch::Result {
        ensure!(Self::symbols(asset_id).is_some(), "AssetId is not exist.");
        Ok(())
    }

    /// Get an account's total balance of an asset kind.
    pub fn total_balance(asset_id: &T::AssetId, who: &T::AccountId) -> T::Balance {
        Self::free_balance(asset_id, who) // + Self::reserved_balance(asset_id, who)
    }

    /// Get an account's free balance of an asset kind.
    pub fn free_balance(asset_id: &T::AssetId, who: &T::AccountId) -> T::Balance {
        <FreeBalance<T>>::get(asset_id, who)
    }

    /// Get an account's reserved balance of an asset kind.
    pub fn reserved_balance(asset_id: &T::AssetId, who: &T::AccountId) -> T::Balance {
        <ReservedBalance<T>>::get(asset_id, who)
    }

    /// Creates an asset.
    ///
    /// # Arguments
    /// * `asset_id`: An ID of a reserved asset.
    /// * `from_account`: The initiator account of this call
    /// * `asset_options`: Asset creation options.
    ///
    pub fn create_asset(
        asset_id: T::AssetId,
        symbol: Vec<u8>,
        from_account: Option<T::AccountId>,
        options: AssetOptions<T::Balance, T::AccountId>,
    ) -> dispatch::Result {
        // make sure the asset id is not exist
        Self::should_not_exist(&asset_id)?;

        let account_id = from_account.unwrap_or_default();
        let permissions: PermissionVersions<T::AccountId> = options.permissions.clone().into();

        info!(
            "[create_asset]|assetid:{:?}, symbol:{:?}, options:{:?}, permissions:{:?}",
            asset_id, symbol, options, permissions
        );

        <TotalIssuance<T>>::insert(&asset_id, &options.initial_issuance);
        <FreeBalance<T>>::insert(&asset_id, &account_id, &options.initial_issuance);
        <Permissions<T>>::insert(&asset_id, permissions);

        <Symbols<T>>::insert(asset_id, symbol.clone());

        Self::deposit_event(RawEvent::Created(asset_id, account_id, options));

        Ok(())
    }

    /// Transfer some liquid free balance from one account to another.
    /// This will not emit the `Transferred` event.
    pub fn make_transfer(
        asset_id: &T::AssetId,
        from: &T::AccountId,
        to: &T::AccountId,
        amount: T::Balance,
    ) -> dispatch::Result {
        Self::should_exist(asset_id)?;

        let new_balance = Self::free_balance(asset_id, from)
            .checked_sub(&amount)
            .ok_or_else(|| "balance too low to send amount")?;
        Self::ensure_can_withdraw(
            asset_id,
            from,
            amount,
            WithdrawReason::Transfer.into(),
            new_balance,
        )?;

        if from != to {
            <FreeBalance<T>>::mutate(asset_id, from, |balance| *balance -= amount);
            <FreeBalance<T>>::mutate(asset_id, to, |balance| *balance += amount);
        }
        debug!(
            "[make_transfer]|assetid:{:?}|from:{:?}|to:{:?}|amount:{:?}|result: from:{:?},to:{:?}",
            asset_id,
            from,
            to,
            amount,
            Self::free_balance(asset_id, from),
            Self::free_balance(asset_id, to)
        );
        Ok(())
    }

    /// Transfer some liquid free balance from one account to another.
    /// This will emit the `Transferred` event.
    pub fn make_transfer_with_event(
        asset_id: &T::AssetId,
        from: &T::AccountId,
        to: &T::AccountId,
        amount: T::Balance,
    ) -> dispatch::Result {
        Self::make_transfer(asset_id, from, to, amount)?;

        if from != to {
            Self::deposit_event(RawEvent::Transferred(
                *asset_id,
                from.clone(),
                to.clone(),
                amount,
            ));
        }

        Ok(())
    }

    pub fn make_transfer_between_assets(
        from_asset_id: &T::AssetId,
        from: &T::AccountId,
        to_asset_id: &T::AssetId,
        to: &T::AccountId,
        reasons: WithdrawReasons,
        amount: T::Balance,
    ) -> dispatch::Result {
        Self::should_exist(from_asset_id)?;
        Self::should_exist(to_asset_id)?;

        let new_balance = Self::free_balance(from_asset_id, from)
            .checked_sub(&amount)
            .ok_or_else(|| "balance too low to send amount")?;

        let _to_balance = Self::free_balance(to_asset_id, to)
            .checked_add(&amount)
            .ok_or_else(|| "balance too height to receive amount")?;

        Self::ensure_can_withdraw(from_asset_id, from, amount, reasons, new_balance)?;

        if from != to || from_asset_id != to_asset_id {
            let _c = FromToContext::<T>::new(*from_asset_id, *to_asset_id);
            // destroy for from asset
            let nag = FromCurrency::<T>::withdraw(
                from,
                amount,
                reasons,
                ExistenceRequirement::KeepAlive,
            )?;
            // issue for to asset
            let pos = ToCurrency::<T>::deposit_creating(to, amount);
            assert_eq!(nag.peek(), pos.peek());
            // after this block, nag and pos would do drop and modify TotalIssue
        }

        debug!("[make_transfer_between_assets]|from_id:{:?}|to_id:{:?}|from:{:?}|to:{:?}|amount:{:?}|result: from:{:?},to:{:?}",
            from_asset_id, to_asset_id, from, to, amount, Self::free_balance(from_asset_id, from), Self::free_balance(to_asset_id, to)
        );

        Ok(())
    }

    /// Mint to an account's free balance, without event
    pub fn mint_free(
        asset_id: &T::AssetId,
        who: &T::AccountId,
        to: &T::AccountId,
        amount: &T::Balance,
    ) -> Result {
        Self::should_exist(asset_id)?;

        if Self::check_permission(asset_id, who, &PermissionType::Mint) {
            let original_free_balance = Self::free_balance(&asset_id, &to);
            let current_total_issuance = <TotalIssuance<T>>::get(asset_id);
            let new_total_issuance = current_total_issuance
                .checked_add(&amount)
                .ok_or_else(|| "total_issuance got overflow after minting.")?;
            let value = original_free_balance
                .checked_add(&amount)
                .ok_or_else(|| "free balance got overflow after minting.")?;
            <TotalIssuance<T>>::insert(asset_id, new_total_issuance);
            Self::set_free_balance(&asset_id, &to, value);

            debug!(
                "[mint_free]|assetid:{:?}|who:{:?}|to:{:?}|amount:{:?}|result: now:{:?},total:{:?}",
                asset_id,
                who,
                to,
                amount,
                Self::free_balance(asset_id, who),
                Self::total_issuance(asset_id)
            );

            Ok(())
        } else {
            Err("The origin does not have permission to mint an asset.")
        }
    }

    /// Burn an account's free balance, without event
    pub fn burn_free(
        asset_id: &T::AssetId,
        who: &T::AccountId,
        to: &T::AccountId,
        amount: &T::Balance,
    ) -> Result {
        Self::should_exist(asset_id)?;

        if Self::check_permission(&asset_id, &who, &PermissionType::Burn) {
            let original_free_balance = Self::free_balance(&asset_id, &to);

            let current_total_issuance = <TotalIssuance<T>>::get(asset_id);
            let new_total_issuance = current_total_issuance
                .checked_sub(&amount)
                .ok_or_else(|| "total_issuance got underflow after burning")?;
            let value = original_free_balance
                .checked_sub(&amount)
                .ok_or_else(|| "free_balance got underflow after burning")?;

            <TotalIssuance<T>>::insert(asset_id, new_total_issuance);

            Self::set_free_balance(&asset_id, &to, value);

            debug!(
                "[burn_free]|assetid:{:?}|who:{:?}|to:{:?}|amount:{:?}|result: now:{:?},total:{:?}",
                asset_id,
                who,
                to,
                amount,
                Self::free_balance(asset_id, who),
                Self::total_issuance(asset_id)
            );

            Ok(())
        } else {
            Err("The origin does not have permission to burn an asset.")
        }
    }

    /// Move `amount` from free balance to reserved balance.
    ///
    /// If the free balance is lower than `amount`, then no funds will be moved and an `Err` will
    /// be returned. This is different behavior than `unreserve`.
    pub fn reserve(
        asset_id: &T::AssetId,
        who: &T::AccountId,
        amount: T::Balance,
    ) -> dispatch::Result {
        // Do we need to consider that this is an atomic transaction?
        let original_reserve_balance = Self::reserved_balance(asset_id, who);
        let original_free_balance = Self::free_balance(asset_id, who);
        if original_free_balance < amount {
            return Err("not enough free funds");
        }
        let new_reserve_balance = original_reserve_balance + amount;
        Self::set_reserved_balance(asset_id, who, new_reserve_balance);
        let new_free_balance = original_free_balance - amount;
        Self::set_free_balance(asset_id, who, new_free_balance);

        debug!(
            "[reserve]|assetid:{:?}|who:{:?}|amount:{:?}|result: free:{:?},reserved:{:?}",
            asset_id,
            who,
            amount,
            Self::free_balance(asset_id, who),
            Self::reserved_balance(asset_id, who),
        );

        Ok(())
    }

    /// Moves up to `amount` from reserved balance to free balance. This function cannot fail.
    ///
    /// As many assets up to `amount` will be moved as possible. If the reserve balance of `who`
    /// is less than `amount`, then the remaining amount will be returned.
    /// NOTE: This is different behavior than `reserve`.
    pub fn unreserve(asset_id: &T::AssetId, who: &T::AccountId, amount: T::Balance) -> T::Balance {
        let b = Self::reserved_balance(asset_id, who);
        let actual = rstd::cmp::min(b, amount);
        let original_free_balance = Self::free_balance(asset_id, who);
        let new_free_balance = original_free_balance + actual;
        Self::set_free_balance(asset_id, who, new_free_balance);
        Self::set_reserved_balance(asset_id, who, b - actual);

        debug!(
            "[unreserve]|assetid:{:?}|who:{:?}|amount:{:?}|result: free:{:?},reserved:{:?}",
            asset_id,
            who,
            amount,
            Self::free_balance(asset_id, who),
            Self::reserved_balance(asset_id, who),
        );

        amount - actual
    }

    // /// Deduct up to `amount` from the combined balance of `who`, preferring to deduct from the
    // /// free balance. This function cannot fail.
    // ///
    // /// As much funds up to `amount` will be deducted as possible. If this is less than `amount`
    // /// then `Some(remaining)` will be returned. Full completion is given by `None`.
    // /// NOTE: LOW-LEVEL: This will not attempt to maintain total issuance. It is expected that
    // /// the caller will do this.
    // fn slash(asset_id: &T::AssetId, who: &T::AccountId, amount: T::Balance) -> Option<T::Balance> {
    //     let free_balance = Self::free_balance(asset_id, who);
    //     let free_slash = rstd::cmp::min(free_balance, amount);
    //     let new_free_balance = free_balance - free_slash;
    //     Self::set_free_balance(asset_id, who, new_free_balance);
    //     if free_slash < amount {
    //         Self::slash_reserved(asset_id, who, amount - free_slash)
    //     } else {
    //         None
    //     }
    // }

    // /// Deducts up to `amount` from reserved balance of `who`. This function cannot fail.
    // ///
    // /// As much funds up to `amount` will be deducted as possible. If the reserve balance of `who`
    // /// is less than `amount`, then a non-zero second item will be returned.
    // /// NOTE: LOW-LEVEL: This will not attempt to maintain total issuance. It is expected that
    // /// the caller will do this.
    // fn slash_reserved(asset_id: &T::AssetId, who: &T::AccountId, amount: T::Balance) -> Option<T::Balance> {
    //     let original_reserve_balance = Self::reserved_balance(asset_id, who);
    //     let slash = rstd::cmp::min(original_reserve_balance, amount);
    //     let new_reserve_balance = original_reserve_balance - slash;
    //     Self::set_reserved_balance(asset_id, who, new_reserve_balance);
    //     if amount == slash {
    //         None
    //     } else {
    //         Some(amount - slash)
    //     }
    // }

    // /// Move up to `amount` from reserved balance of account `who` to free balance of account
    // /// `beneficiary`.
    // ///
    // /// As much funds up to `amount` will be moved as possible. If this is less than `amount`, then
    // /// the `remaining` would be returned, else `Zero::zero()`.
    // /// NOTE: LOW-LEVEL: This will not attempt to maintain total issuance. It is expected that
    // /// the caller will do this.
    // fn repatriate_reserved(
    //     asset_id: &T::AssetId,
    //     who: &T::AccountId,
    //     beneficiary: &T::AccountId,
    //     amount: T::Balance,
    // ) -> T::Balance {
    //     let b = Self::reserved_balance(asset_id, who);
    //     let slash = rstd::cmp::min(b, amount);
    //
    //     let original_free_balance = Self::free_balance(asset_id, beneficiary);
    //     let new_free_balance = original_free_balance + slash;
    //     Self::set_free_balance(asset_id, beneficiary, new_free_balance);
    //
    //     let new_reserve_balance = b - slash;
    //     Self::set_reserved_balance(asset_id, who, new_reserve_balance);
    //     amount - slash
    // }

    /// Check permission to perform burn, mint or update.
    ///
    /// # Arguments
    /// * `asset_id`:  A `T::AssetId` type that contains the `asset_id`, which has the permission embedded.
    /// * `who`: A `T::AccountId` type that contains the `account_id` for which to check permissions.
    /// * `what`: The permission to check.
    ///
    pub fn check_permission(
        asset_id: &T::AssetId,
        who: &T::AccountId,
        what: &PermissionType,
    ) -> bool {
        let permission_versions: PermissionVersions<T::AccountId> = Self::get_permission(asset_id);
        let permission = permission_versions.into();

        match (what, permission) {
            (
                PermissionType::Burn,
                PermissionLatest {
                    burn: Owner::Address(account),
                    ..
                },
            ) => account == *who,
            (
                PermissionType::Mint,
                PermissionLatest {
                    mint: Owner::Address(account),
                    ..
                },
            ) => account == *who,
            (
                PermissionType::Update,
                PermissionLatest {
                    update: Owner::Address(account),
                    ..
                },
            ) => account == *who,
            _ => false,
        }
    }

    /// Return `Ok` iff the account is able to make a withdrawal of the given amount
    /// for the given reason.
    ///
    /// `Err(...)` with the reason why not otherwise.
    pub fn ensure_can_withdraw(
        asset_id: &T::AssetId,
        _who: &T::AccountId,
        _amount: T::Balance,
        reasons: WithdrawReasons,
        _new_balance: T::Balance,
    ) -> dispatch::Result {
        if reasons.contains(WithdrawReason::Transfer) {
            // check transfer restriction
            permissions::can_do::<T>(asset_id, permissions::Restriction::Transferable)?;
        }

        Ok(())
    }

    // PRIVATE MUTABLES

    /// NOTE: LOW-LEVEL: This will not attempt to maintain total issuance. It is expected that
    /// the caller will do this.
    fn set_reserved_balance(asset_id: &T::AssetId, who: &T::AccountId, balance: T::Balance) {
        <ReservedBalance<T>>::insert(asset_id, who, &balance);
    }

    /// NOTE: LOW-LEVEL: This will not attempt to maintain total issuance. It is expected that
    /// the caller will do this.
    fn set_free_balance(asset_id: &T::AssetId, who: &T::AccountId, balance: T::Balance) {
        <FreeBalance<T>>::insert(asset_id, who, &balance);
    }

    // fn set_lock(
    //     id: LockIdentifier,
    //     who: &T::AccountId,
    //     amount: T::Balance,
    //     until: T::BlockNumber,
    //     reasons: WithdrawReasons,
    // ) {
    //     let now = <system::Module<T>>::block_number();
    //     let mut new_lock = Some(BalanceLock {
    //         id,
    //         amount,
    //         until,
    //         reasons,
    //     });
    //     let mut locks = Self::locks(who)
    //         .into_iter()
    //         .filter_map(|l| {
    //             if l.id == id {
    //                 new_lock.take()
    //             } else if l.until > now {
    //                 Some(l)
    //             } else {
    //                 None
    //             }
    //         })
    //         .collect::<Vec<_>>();
    //     if let Some(lock) = new_lock {
    //         locks.push(lock)
    //     }
    //     <Locks<T>>::insert(who, locks);
    // }
    //
    // fn extend_lock(
    //     id: LockIdentifier,
    //     who: &T::AccountId,
    //     amount: T::Balance,
    //     until: T::BlockNumber,
    //     reasons: WithdrawReasons,
    // ) {
    //     let now = <system::Module<T>>::block_number();
    //     let mut new_lock = Some(BalanceLock {
    //         id,
    //         amount,
    //         until,
    //         reasons,
    //     });
    //     let mut locks = Self::locks(who)
    //         .into_iter()
    //         .filter_map(|l| {
    //             if l.id == id {
    //                 new_lock.take().map(|nl| BalanceLock {
    //                     id: l.id,
    //                     amount: l.amount.max(nl.amount),
    //                     until: l.until.max(nl.until),
    //                     reasons: l.reasons | nl.reasons,
    //                 })
    //             } else if l.until > now {
    //                 Some(l)
    //             } else {
    //                 None
    //             }
    //         })
    //         .collect::<Vec<_>>();
    //     if let Some(lock) = new_lock {
    //         locks.push(lock)
    //     }
    //     <Locks<T>>::insert(who, locks);
    // }

    // fn remove_lock(id: LockIdentifier, who: &T::AccountId) {
    //     let now = <system::Module<T>>::block_number();
    //     let locks = Self::locks(who)
    //         .into_iter()
    //         .filter_map(|l| if l.until > now && l.id != id { Some(l) } else { None })
    //         .collect::<Vec<_>>();
    //     <Locks<T>>::insert(who, locks);
    // }
}
