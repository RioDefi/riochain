#![cfg_attr(not(feature = "std"), no_std)]
#[allow(unused_imports)]
#[cfg(test)]
mod tests;
mod weight_info;

use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use sp_runtime::{
    traits::{
        Bounded, CheckedAdd, CheckedMul, CheckedSub, MaybeDisplay, MaybeSerializeDeserialize,
        Member, One, Saturating, StaticLookup, Zero,
    },
    RuntimeDebug,
};
use sp_std::{prelude::*, vec::Vec};

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult,
    ensure,
    traits::{Currency, Get},
    IterableStorageMap,
};
#[allow(unused_imports)]
use frame_system::{ensure_root, ensure_signed};

pub use rio_assets::AssetOptions as DclAssetOptions;
pub use rio_assets::PermissionLatest as DclPermissionLatest;
pub use weight_info::WeightInfo;
use rio_support::debug;

#[derive(Encode, Decode, PartialEq, Eq, Clone, Copy, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum BlackOrWhite {
    Black,
    White,
}
impl Default for BlackOrWhite {
    fn default() -> Self {
        Self::Black
    }
}

#[derive(Encode, Decode, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum DclAuth {
    All,
    AddCoin,
    Task,
    None,
}
impl Default for DclAuth {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Encode, Decode, Clone, Default, PartialEq, Eq, RuntimeDebug)]
pub struct VipLevelPrice<Balance, AssetId> {
    pub amount: Balance,
    pub asset_id: AssetId,
}

#[derive(Encode, Decode, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum VipLevel {
    Normal = 0,
    Level1 = 1,
    Level2 = 2,
    Level3 = 3,
    Level4 = 4,
    Level5 = 5,
    Level6 = 6,
}
impl Default for VipLevel {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Encode, Decode, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum DclCoinSymbol {
    USDT,
    EMC,
    ELC,
    ETG,
}
impl Default for DclCoinSymbol {
    fn default() -> Self {
        Self::USDT
    }
}

// #[cfg(feature = "std")]
impl core::fmt::Debug for DclAuth {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "DclAuth ")
    }
}
impl core::fmt::Debug for VipLevel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "VipLevel ")
    }
}
impl core::fmt::Debug for DclCoinSymbol {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "DclCoinSymbol")
    }
}

pub trait Trait:
    frame_system::Trait + rio_assets::Trait + pallet_sudo::Trait + pallet_timestamp::Trait
{
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

    type WeightInfo: WeightInfo;
}

pub type BalanceOf<T> = <<T as rio_assets::Trait>::Currency as Currency<
    <T as frame_system::Trait>::AccountId,
>>::Balance;

decl_error! {
    /// Error for the dcl module.
    pub enum Error for Module<T: Trait> {
        /// no AddCoin auth
        NoAddCoinAuth,
        /// no All auth
        NoAllAuth,
        /// no Task auth
        NoTaskAuth,
        /// no authorize
        UnAuthorized,
        /// is not vip
        NotVip,
        /// is not Task account
        NotTaskAccount,
        /// already max level
        AlreadyMaxLevel,
        /// set level error
        LevelError,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as RioDappDcl {
        pub SystemAccount get(fn system_account) config(): T::AccountId;
        pub Admins get(fn admins):
            map hasher(blake2_128_concat) T::AccountId => Option<DclAuth>;
        pub VipList get(fn vip_list):
            map hasher(blake2_128_concat) T::AccountId => VipLevel = VipLevel::Normal;
        pub MerchantReturnRate get(fn merchant_return_rate): BalanceOf<T>;
        pub VipLevelPriceList get(fn vip_level_price_list):
            map hasher(twox_64_concat) VipLevel => VipLevelPrice<BalanceOf<T>, T::AssetId>;

        pub ACoinAssetId get(fn a_coin_asset_id): T::AssetId;
        pub BCoinAssetId get(fn b_coin_asset_id): T::AssetId;
        pub CCoinAssetId get(fn c_coin_asset_id): T::AssetId;
        pub USDTAssetId get(fn usdt_asset_id): T::AssetId;

        // pub ACoinSymbol get(fn a_coin_symbol): Vec<u8>;
        // pub BCoinSymbol get(fn b_coin_symbol): Vec<u8>;
        // pub CCoinSymbol get(fn c_coin_symbol): Vec<u8>;
        // pub USDTSymbol get(fn usdt_symbol): Vec<u8>;
    }

    add_extra_genesis {
        config(admins): Vec<(T::AccountId, DclAuth)>;
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
        Balance = BalanceOf<T>,
    {
        // Just a dummy event.
        // Event `Something` is declared with a parameter of the type `u32` and `AccountId`
        // To emit this event, we call the deposit funtion, from our runtime funtions
        DclPurchaseEvent(AccountId, AccountId, Balance),
        PayForVipEvent(AccountId, AccountId, Balance, VipLevel),
        SendBCoinEvent(AccountId, AccountId, Balance, Vec<u8>),
        MerchantReturnEvent(AccountId, AccountId, Balance, Vec<u8>),
    }
);

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        fn deposit_event() = default;

        // 
        #[weight = T::WeightInfo::set_merchant_return_rate()]
        fn set_merchant_return_rate(origin, #[compact] rate: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Self::has_auth(who.clone(), DclAuth::All), Error::<T>::NoAllAuth);
            <MerchantReturnRate<T>>::mutate(|v| {
                *v = rate
            });
            Ok(())
        }

        #[weight = T::WeightInfo::send_coin_by_admin()]
        fn send_coin_by_admin(origin, symbol: DclCoinSymbol, to: <T::Lookup as StaticLookup>::Source, #[compact] amount: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let to = T::Lookup::lookup(to)?;
            ensure!(Self::has_auth(who.clone(), DclAuth::AddCoin), Error::<T>::NoAddCoinAuth);

            let asset_id = match symbol {
                DclCoinSymbol::USDT => Self::usdt_asset_id(),
                DclCoinSymbol::EMC => Self::a_coin_asset_id(),
                DclCoinSymbol::ELC => Self::b_coin_asset_id(),
                DclCoinSymbol::ETG => Self::c_coin_asset_id(),
            };
            Self::mint_inner(to, asset_id, amount)?;
            Ok(())
        }

        #[weight = T::WeightInfo::set_usdt_asset_id()]
        fn set_usdt_asset_id(origin, #[compact] asset_id: T::AssetId) -> DispatchResult {
            // 
            let who = ensure_signed(origin)?;
            ensure!(Self::has_auth(who, DclAuth::AddCoin), Error::<T>::NoAllAuth);
            <USDTAssetId<T>>::mutate(|v| {
                *v = asset_id
            });
            Ok(())
        }

        #[weight = T::WeightInfo::set_coins_asset_id()]
        fn set_coins_asset_id(origin, #[compact] acoin: T::AssetId, #[compact] bcoin: T::AssetId, #[compact] ccoin: T::AssetId, #[compact] usdt: T::AssetId) -> DispatchResult {
            // 
            let who = ensure_signed(origin)?;
            ensure!(Self::has_auth(who, DclAuth::AddCoin), Error::<T>::NoAddCoinAuth);
            <ACoinAssetId<T>>::mutate(|v| {
                *v = acoin
            });
            <BCoinAssetId<T>>::mutate(|v| {
                *v = bcoin
            });
            <CCoinAssetId<T>>::mutate(|v| {
                *v = ccoin
            });
            <USDTAssetId<T>>::mutate(|v| {
                *v = usdt
            });
            Ok(())
        }

        #[weight = T::WeightInfo::add_admin()]
        fn add_admin(origin, account_id: <T::Lookup as StaticLookup>::Source, auth: DclAuth) -> DispatchResult {
            let account_id = T::Lookup::lookup(account_id)?;
            if <Admins<T>>::iter().count() == 0 {
                ensure_root(origin)?;
                <Admins<T>>::insert(account_id, auth);
            } else {
                let who = ensure_signed(origin)?;
                ensure!(Self::has_auth(who, DclAuth::All), Error::<T>::NoAllAuth);
                <Admins<T>>::insert(account_id, auth);
            }
            Ok(())
        }

        #[weight = T::WeightInfo::set_vip_price()]
        fn set_vip_price(origin, level: VipLevel, #[compact] price: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Self::has_auth(who, DclAuth::All), Error::<T>::NoAllAuth);
            let option = VipLevelPrice {
                amount: price,
                asset_id: Self::usdt_asset_id(),
            };
            <VipLevelPriceList<T>>::insert(level, option);
            Ok(())
        }

        #[weight = T::WeightInfo::set_system_account()]
        pub fn set_system_account(origin, account_id: <T::Lookup as StaticLookup>::Source) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let account_id = T::Lookup::lookup(account_id)?;

            ensure!(Self::has_auth(who, DclAuth::All), Error::<T>::UnAuthorized);
            <SystemAccount<T>>::mutate(|v| {
                *v = account_id
            });
            Ok(())
        }

        // 
        #[weight = T::WeightInfo::free_acoin()]
        pub fn free_acoin(origin, account_id: <T::Lookup as StaticLookup>::Source) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let account_id = T::Lookup::lookup(account_id)?;

            ensure!(Self::has_auth(who, DclAuth::Task), Error::<T>::NoTaskAuth);
            ensure!(Self::is_vip(account_id.clone()), Error::<T>::NotVip);
            let asset_id = Self::a_coin_asset_id();
            let can_free_balance = <rio_assets::Module<T>>::generic_reserved_balance(&asset_id, &account_id);
            let amount = can_free_balance * BalanceOf::<T>::from(5) / BalanceOf::<T>::from(10000);

            debug!("[free_acoin]free a coin {:?}", amount);

            Self::force_free_coins(asset_id, account_id, amount)?;
            // TODO 
            // TODO free 0.0005
            Ok(())
        }

        // 
        #[weight = T::WeightInfo::free_bcoin()]
        pub fn free_bcoin(origin, account_id: <T::Lookup as StaticLookup>::Source) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let account_id = T::Lookup::lookup(account_id)?;

            ensure!(Self::has_auth(who, DclAuth::Task), Error::<T>::NoTaskAuth);
            ensure!(Self::is_vip(account_id.clone()), Error::<T>::NotVip);
            let asset_id = Self::b_coin_asset_id();
            let can_free_balance = <rio_assets::Module<T>>::generic_reserved_balance(&asset_id, &account_id);
            let amount = can_free_balance * BalanceOf::<T>::from(5) / BalanceOf::<T>::from(10000);

            debug!("[free_bcoin]free b coin {:?}", amount);

            Self::force_free_coins(asset_id, account_id, amount)?;
            // TODO check if coin remain more than 1000, free 0.0005
            Ok(())
        }

        #[weight = T::WeightInfo::mint_and_lock()]
        pub fn mint_and_lock(origin, #[compact] asset_id: T::AssetId, #[compact] amount: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::mint_and_lock_inner(who, asset_id, amount)?;
            Ok(())
        }

        // TODO 
        #[weight = T::WeightInfo::send_b_coin()]
        pub fn send_b_coin(origin, account_id: <T::Lookup as StaticLookup>::Source, #[compact] amount: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let account_id = T::Lookup::lookup(account_id)?;

            ensure!(Self::has_auth(who.clone(), DclAuth::Task), Error::<T>::NotTaskAccount);
            ensure!(Self::is_vip(account_id.clone()), Error::<T>::NotVip);
            let asset_id = Self::b_coin_asset_id();
            Self::mint_and_lock_inner(account_id.clone(), asset_id, amount.clone())?;
            Self::deposit_event(RawEvent::SendBCoinEvent(who, account_id, amount, b"".to_vec()));
            Ok(())
        }

        //

        #[weight = T::WeightInfo::join()]
        pub fn join(origin, #[compact] amount: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let root1 = frame_system::RawOrigin::Root.into();
            let root2 = frame_system::RawOrigin::Root.into();
            let root3 = frame_system::RawOrigin::Root.into();

            let aid = Self::a_coin_asset_id();
            let bid = Self::b_coin_asset_id();
            let cid = Self::c_coin_asset_id();

            let free_a_balance = <rio_assets::Module<T>>::free_balance(&aid, &who);
            let free_b_balance = <rio_assets::Module<T>>::free_balance(&bid, &who);
            ensure!(free_a_balance >= amount, rio_assets::Error::<T>::InsufficientBalance);
            ensure!(free_b_balance >= amount, rio_assets::Error::<T>::InsufficientBalance);

            let addr = T::Lookup::unlookup(who);
            <rio_assets::Module<T>>::burn(root1, aid, addr.clone(), amount.clone())?;
            <rio_assets::Module<T>>::burn(root2, bid, addr.clone(), amount.clone())?;
            <rio_assets::Module<T>>::mint(root3, cid, addr.clone(), amount * BalanceOf::<T>::from(2))?;
            Ok(())
        }

        #[weight = T::WeightInfo::dcl_purchase()]
        pub fn dcl_purchase(origin, to: <T::Lookup as StaticLookup>::Source, #[compact] amount: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let to = T::Lookup::lookup(to)?;

            let asset_id = Self::usdt_asset_id();
            let balance = <rio_assets::Module<T>>::free_balance(&asset_id.clone(), &who.clone());
            ensure!(balance >= amount.clone(), rio_assets::Error::<T>::InsufficientBalance);
            <rio_assets::Module<T>>::make_transfer(&asset_id, &who.clone(), &to, amount.clone())?;
            // Self::mint_and_lock_inner(who.clone(), Self::a_coin_asset_id(), amount.clone())?;
            Self::deposit_event(RawEvent::DclPurchaseEvent(who, to, amount));
            Ok(())
        }

        #[weight = T::WeightInfo::pay_for_vip()]
        fn pay_for_vip(origin, level: VipLevel) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let sys_acc = Self::system_account();
            let asset_id = Self::usdt_asset_id();
            let level_now = Self::vip_list(who.clone());

            debug!("[pay_for_vip]pay_for_vip: asset: {:?} {:?}", asset_id, level_now);

            match level_now {
                VipLevel::Level6 => {
                    Err(Error::<T>::AlreadyMaxLevel)?
                },
                _ => {
                    if (level as u32) <= (level_now as u32) {
                        Err(Error::<T>::LevelError)?
                    }
                    let vip_level_content = Self::vip_level_price_list(level);
                    let amount = vip_level_content.amount;
                    <rio_assets::Module<T>>::make_transfer(&asset_id, &who.clone(), &sys_acc, amount.clone())?;
                    Self::mint_and_lock_inner(who.clone(), Self::a_coin_asset_id(), amount.clone())?;
                    <VipList<T>>::insert(who.clone(), level);
                    Self::deposit_event(RawEvent::PayForVipEvent(who, sys_acc, amount, level));
                    Ok(())
                }
            }
        }

        #[weight = T::WeightInfo::merchant_return()]
        pub fn merchant_return(origin, customer: <T::Lookup as StaticLookup>::Source, #[compact] amount: BalanceOf<T>) -> DispatchResult {
            let merchant = ensure_signed(origin)?;
            let customer = T::Lookup::lookup(customer)?;

            let our = Self::system_account();
            let asset_id = Self::a_coin_asset_id();
            let usdt_asset_id = Self::usdt_asset_id();
            let return_amount = amount.clone() * Self::merchant_return_rate() / BalanceOf::<T>::from(100);

            <rio_assets::Module<T>>::make_transfer(&usdt_asset_id, &merchant.clone(), &our, return_amount.clone())?;
            Self::mint_and_lock_inner(customer.clone(), asset_id, amount.clone())?;
            Self::mint_and_lock_inner(merchant.clone(), asset_id, return_amount.clone())?;
            Self::deposit_event(RawEvent::MerchantReturnEvent(merchant, customer, return_amount, "".as_bytes().to_vec()));
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    fn force_free_coins(
        asset_id: T::AssetId,
        account_id: T::AccountId,
        amount: BalanceOf<T>,
    ) -> DispatchResult {
        <rio_assets::Module<T>>::generic_unreserve(&asset_id, &account_id, amount);
        Ok(())
    }

    fn mint_and_lock_inner(
        account_id: T::AccountId,
        asset_id: T::AssetId,
        amount: BalanceOf<T>,
    ) -> DispatchResult {
        // TODO remove origin
        let root1 = frame_system::RawOrigin::Root.into();
        let addr = T::Lookup::unlookup(account_id.clone());
        <rio_assets::Module<T>>::mint(root1, asset_id, addr, amount.clone())?;
        <rio_assets::Module<T>>::generic_reserve(&asset_id, &account_id, amount)?;
        Ok(())
    }

    fn mint_inner(
        account_id: T::AccountId,
        asset_id: T::AssetId,
        amount: BalanceOf<T>,
    ) -> DispatchResult {
        // todo remove origin
        let root1 = frame_system::RawOrigin::Root.into();
        let addr = T::Lookup::unlookup(account_id);
        <rio_assets::Module<T>>::mint(root1, asset_id, addr, amount.clone())?;
        Ok(())
    }

    fn has_auth(account_id: T::AccountId, auth: DclAuth) -> bool {
        Self::admins(&account_id)
            .map(|account_auth| account_auth == DclAuth::All || account_auth == auth)
            .unwrap_or(false)
    }

    fn is_vip(account_id: T::AccountId) -> bool {
        return if !<VipList<T>>::contains_key(&account_id) {
            false
        } else {
            Self::vip_list(account_id) != VipLevel::Normal
        };
    }
}
