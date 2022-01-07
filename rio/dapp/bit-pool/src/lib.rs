//! this module provides a simple account to aggregate the transaction fee
//! this account is under control of the RIO team
//! this is done by applying the "transaction_payment" strategy

#![cfg_attr(not(feature = "std"), no_std)]

#[allow(unused_imports)]
use codec::{Decode, Encode, Error as codecErr, HasCompact, Input, Output};

#[cfg(test)]
mod tests;
mod weight_info;

#[allow(unused_imports)]
use sp_runtime::traits::{
    Bounded, CheckedAdd, CheckedMul, CheckedSub, MaybeDisplay, MaybeSerializeDeserialize, Member,
    One, Saturating, StaticLookup, Zero,
};

use sp_std::{collections::btree_map, prelude::*};

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult,
    ensure,
    traits::{Currency, Get},
    weights::Weight,
    IterableStorageMap, StorageMap, StorageValue,
};
use frame_system::{ensure_root, ensure_signed};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

pub type RoundId = u64;
pub type GameWay = u32;

pub use rio_price::Price;
pub use weight_info::WeightInfo;

#[derive(Encode, Decode, PartialEq, Eq, Clone, Copy, Debug)]
pub enum BetType {
    Short,
    Long,
}
impl Default for BetType {
    fn default() -> Self {
        BetType::Short
    }
}

#[derive(Encode, Decode, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub enum BillingType {
    Win,        //
    Lose,       //
    Unchanging, //
}

impl Default for BillingType {
    fn default() -> Self {
        BillingType::Unchanging
    }
}

#[derive(Encode, Decode, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub enum GameChangeType {
    Add,
    Update,
    Unchanging,
}

impl Default for GameChangeType {
    fn default() -> Self {
        GameChangeType::Unchanging
    }
}

#[derive(Encode, Decode, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub enum GameStatusType {
    Begin,
    End,
    Paused,
    Restart,
    Unchanging,
}

impl Default for GameStatusType {
    fn default() -> Self {
        GameStatusType::Unchanging
    }
}

#[derive(Encode, Decode, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub enum RecommendStatusType {
    Success,
    Failure,
    Binded,
    Unchanging,
}

impl Default for RecommendStatusType {
    fn default() -> Self {
        RecommendStatusType::Unchanging
    }
}

#[derive(Encode, Decode, Default, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct GameControlItem<Moment> {
    pub paused: bool,
    pub time_stamp: Moment,
    pub wait_time: u32,
    pub duration: u32,
}

#[derive(Encode, Decode, Default, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct GameWayInfo<AssetId> {
    pub game_way: GameWay,
    pub asset_id: AssetId,
    pub duration: u32,
    pub wait_time: u32,
}

#[derive(Encode, Decode, Default, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct BetItem<AccountId, Balance, AssetId> {
    pub account_id: AccountId,
    pub amount: Balance,
    pub asset_id: AssetId,
    pub is_root: bool,
}

#[derive(Encode, Decode, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct BetPriceLimit<Balance> {
    pub min_bet: Balance,
    pub max_bet: Balance,
}

/// The module's configuration trait.
pub trait Trait:
    frame_system::Trait
    + rio_assets::Trait
    + pallet_sudo::Trait
    + pallet_timestamp::Trait
    + rio_price::Trait
{
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

    type WeightInfo: WeightInfo;
}

pub type BalanceOf<T> = <<T as rio_assets::Trait>::Currency as Currency<
    <T as frame_system::Trait>::AccountId,
>>::Balance;

decl_error! {
    /// Error for the bit-pool module.
    pub enum Error for Module<T: Trait> {
        /// Relative already set
        RelativeAlreadySetted,
        /// No auth
        UnAuthorized,
        /// Bet mode not exists
        BetModeNotExisted,
        /// Bet mode already exists
        BitModeExisted,
        /// Bet pause or end of bet
        BetPaused,
        /// Bet already started
        BetStarted,
        /// Bet are billing
        BetBilling,
        /// Price can not be zero
        PriceZero,
        /// Please set bet duration
        DurationZero,
        /// Bet waiting stage
        WaitingStage,
        /// Bet cannot be less than or equal to zero
        AmountZero,
        /// Bet amount is too small
        AmountSmall,
        /// Bet amount is too large
        AmountLarge,
        /// not set limit for this asset
        NotSetLimit,
        /// Does not support for this asset
        NotSupportAsset,
    }
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as RioDappBitPool {
        /// the asset that user saves into our program
        BetCollectionAssetId get(fn bet_collection_asset_id):
            map hasher(twox_64_concat) GameWay => T::AssetId;

        /// the account where user bet
        BetCollectionAccountId get(fn bet_collection_account_id): T::AccountId;

        /// the account where user rake
        RakeCollectionAccountId get(fn rake_collection_account_id): T::AccountId;

        Admins get(fn admins):
            map hasher(blake2_128_concat) T::AccountId => bool;

        // bet price while game begin
        BetingBtcPrice get(fn beting_btc_price):
            map hasher(twox_64_concat) GameWay => Price;

        // bet price while billing
        BillingBtcPrice get(fn billing_btc_price):
            map hasher(twox_64_concat) GameWay => Price;

        // btc price now
        NowBtcPrice get(fn now_btc_price): Price;

        // queue of longs
        BetLongs get(fn bet_longs):
            map hasher(twox_64_concat) GameWay => Vec<BetItem<T::AccountId,BalanceOf<T>,T::AssetId>>;

        // queue of shors
        BetShorts get(fn bet_shorts):
            map hasher(twox_64_concat) GameWay => Vec<BetItem<T::AccountId,BalanceOf<T>,T::AssetId>>;

        // rake pool
        BetRakes get(fn bet_rakes):
            map hasher(twox_64_concat) GameWay => Vec<BetItem<T::AccountId,BalanceOf<T>,T::AssetId>>;

        // accommend map
        RecommendRelative get(fn recommend_relative):
            map hasher(blake2_128_concat) T::AccountId => Option<T::AccountId>;

        // Recommend subordinate
        RecommendSubordinate get(fn recommend_subordinate):
            map hasher(blake2_128_concat) T::AccountId => Vec<T::AccountId>;

        // round controller
        RoundController get(fn round_controller):
            map hasher(twox_64_concat) GameWay => RoundId;

        // game controller
        GameController get(fn game_controller):
            map hasher(twox_64_concat) GameWay => Option<GameControlItem<T::Moment>>;

        // store all gameway
        GameAllType get(fn game_all_type): Vec<GameWayInfo<T::AssetId>>;

        // lock resume controller or bolling controller
        LockResumeController get(fn lock_resume_controller):
            map hasher(twox_64_concat) GameWay => bool;

        // acommend reward controller for percent
        XPercent get(fn x_percent) : u32 = 0;
        YPercent get(fn y_percent) : u32 = 0;

        // fee
        FeePercent get(fn fee_percent) : u32 = 29;

        // min and max bet price
        MaxMinBetPrice get(fn max_min_bet_price):
            map hasher(twox_64_concat) T::AssetId => BetPriceLimit<BalanceOf<T>>;

        TestVar get(fn test_var): u32;
    }

    add_extra_genesis {
        // config(admins): Vec<(T::AccountId, bool)>;
        // config(bet_collection_account_id): T::AccountId;
        // config(rake_collection_account_id): T::AccountId;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        fn deposit_event() = default;

        fn on_initialize(height: T::BlockNumber) -> Weight {
            // interval set btc price
            <NowBtcPrice>::put(<rio_price::Module<T>>::current_price());

            let now = <pallet_timestamp::Module<T>>::get();
            for (k, mut v) in <GameController<T>>::iter() {
                // if v.paused {
                //     continue;
                // }
                if now > v.time_stamp {
                    v.paused = true;
                }

                let gametime = v.time_stamp + T::Moment::from(v.duration) + T::Moment::from(v.wait_time);
                if gametime > now {
                    continue;
                }

                Self::round_end_billing(k);
            }
            0
        }

        fn on_finalize(_height: T::BlockNumber) {

        }

        #[weight = T::WeightInfo::set_admin_account()]
        pub fn set_admin_account(origin, account_id: <T::Lookup as StaticLookup>::Source, auth: bool) -> DispatchResult {
            ensure_root(origin)?;
            let account_id = T::Lookup::lookup(account_id)?;

            <Admins<T>>::insert(account_id.clone(), auth);

            Ok(())
        }

        // set acommend relative
        #[weight = T::WeightInfo::bind_recommend_account()]
        pub fn bind_recommend_account(origin, account_id: <T::Lookup as StaticLookup>::Source) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let account_id = T::Lookup::lookup(account_id)?;

            if Self::recommend_relative(&who).is_some() {
                Self::deposit_event(RawEvent::RecommendStatus(RecommendStatusType::Binded));
                Err(Error::<T>::RelativeAlreadySetted)?
            }

            <RecommendRelative<T>>::insert(&who, account_id.clone());
            Self::set_subordinate(account_id.clone(), who);

            Self::deposit_event(RawEvent::RecommendStatus(RecommendStatusType::Success));

            Ok(())
        }

        #[weight = T::WeightInfo::sudo_bind_recommend_account()]
        pub fn sudo_bind_recommend_account(origin, account_src: <T::Lookup as StaticLookup>::Source, account_id: <T::Lookup as StaticLookup>::Source) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let account_src = T::Lookup::lookup(account_src)?;
            let account_id = T::Lookup::lookup(account_id)?;

            ensure!(Self::has_auth(&who), Error::<T>::UnAuthorized);

            <RecommendRelative<T>>::insert(account_src.clone(), account_id.clone());
            Self::set_subordinate(account_id.clone(), account_src.clone());

            Self::deposit_event(RawEvent::RecommendStatus(RecommendStatusType::Success));

            Ok(())
        }

        // pause or bet end
        #[weight = T::WeightInfo::pause()]
        pub fn pause(origin, game_way: GameWay) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Self::has_auth(&who), Error::<T>::UnAuthorized);

            let controller = Self::game_controller(game_way).ok_or(Error::<T>::BetModeNotExisted)?;
            ensure!(!controller.paused, Error::<T>::BetPaused);

            Self::set_puase_status(game_way, true);
            Self::deposit_event(RawEvent::GameStatus(game_way, Self::round_controller(game_way),GameStatusType::Paused));

            Ok(())
        }

        // new game
        #[weight = T::WeightInfo::resume()]
        pub fn resume(origin, game_way: GameWay) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Self::has_auth(&who), Error::<T>::UnAuthorized);

            let controller = Self::game_controller(game_way).ok_or(Error::<T>::BetModeNotExisted)?;
            ensure!(controller.paused, Error::<T>::BetStarted);
            ensure!(!<LockResumeController>::get(game_way), Error::<T>::BetBilling);

            Self::set_puase_status(game_way, false);
            Self::deposit_event(RawEvent::GameStatus(game_way, Self::round_controller(game_way),GameStatusType::Restart));

            Ok(())
        }

        #[weight = T::WeightInfo::set_begin_btc_price()]
        pub  fn set_begin_btc_price(origin, game_way: GameWay, #[compact] price: Price) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Self::has_auth(&who), Error::<T>::UnAuthorized);

            ensure!(!price.is_zero(), Error::<T>::PriceZero);

            Self::set_bet_btc_price(game_way, price);

            Ok(())
        }

        #[weight = T::WeightInfo::set_bet_collection_account()]
        pub fn set_bet_collection_account(origin, account_id: <T::Lookup as StaticLookup>::Source) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let account_id = T::Lookup::lookup(account_id)?;

            ensure!(Self::has_auth(&who), Error::<T>::UnAuthorized);

            <BetCollectionAccountId<T>>::put(account_id.clone());

            Ok(())
        }

        #[weight = T::WeightInfo::set_rake_collection_account()]
        pub fn set_rake_collection_account(origin, account_id: <T::Lookup as StaticLookup>::Source) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let account_id = T::Lookup::lookup(account_id)?;

            ensure!(Self::has_auth(&who), Error::<T>::UnAuthorized);

            <RakeCollectionAccountId<T>>::put(account_id.clone());

            Ok(())
        }

        #[weight = T::WeightInfo::set_bet_collection_asset_id()]
        pub fn set_bet_collection_asset_id(origin, #[compact] asset_id: T::AssetId, game_way: GameWay) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Self::has_auth(&who), Error::<T>::UnAuthorized);

            rio_assets::Module::<T>::should_exist(&asset_id)?;
            ensure!(<RoundController>::contains_key(&game_way), Error::<T>::BetModeNotExisted);

            <BetCollectionAssetId<T>>::insert(&game_way,asset_id);

            Ok(())
        }

        #[weight = T::WeightInfo::set_xy_percent()]
        pub fn set_xy_percent(origin, #[compact] x_percent: u32, #[compact] y_percent: u32) ->DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Self::has_auth(&who), Error::<T>::UnAuthorized);

            <XPercent>::put(x_percent);
            <YPercent>::put(y_percent);

            Ok(())
        }

        #[weight = T::WeightInfo::set_fee_percent()]
        pub fn set_fee_percent(origin, #[compact] percent: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Self::has_auth(&who), Error::<T>::UnAuthorized);

            <FeePercent>::put(percent);

            Ok(())
        }

        #[weight = T::WeightInfo::set_min_bet_price()]
        pub fn set_min_bet_price(origin, #[compact] asset_id: T::AssetId, #[compact] min_price: BalanceOf<T>, #[compact] max_price: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Self::has_auth(&who), Error::<T>::UnAuthorized);

            Self::reset_limit_bet_price(asset_id, min_price, max_price);

            Ok(())
        }

        #[weight = T::WeightInfo::bet()]
        pub fn bet(origin, #[compact] asset_id: T::AssetId, game_way: GameWay, #[compact] amount: BalanceOf<T>, bet_type: BetType) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(<RoundController>::contains_key(game_way), Error::<T>::BetModeNotExisted);
            let controller = Self::game_controller(game_way).ok_or(Error::<T>::BetModeNotExisted)?;

            let now = <pallet_timestamp::Module<T>>::get();
            let game_time = controller.time_stamp + T::Moment::from(controller.duration);
            ensure!(now < game_time, Error::<T>::WaitingStage);

            ensure!(!controller.paused, Error::<T>::BetPaused);
            ensure!(!amount.is_zero(), Error::<T>::AmountZero);
            ensure!(<MaxMinBetPrice<T>>::contains_key(asset_id), Error::<T>::NotSetLimit);
            ensure!(<MaxMinBetPrice<T>>::get(asset_id).min_bet <= amount, Error::<T>::AmountSmall);
            ensure!(<MaxMinBetPrice<T>>::get(asset_id).max_bet >= amount, Error::<T>::AmountLarge);
            ensure!(<BetCollectionAssetId<T>>::get(game_way) == asset_id, Error::<T>::NotSupportAsset);
            ensure!(<rio_assets::Module<T>>::free_balance(&asset_id, &who) >= amount, rio_assets::Error::<T>::InsufficientBalance);


            let rake_collection_account_id = Self::rake_collection_account_id();
            let rake = amount * BalanceOf::<T>::from(Self::fee_percent()) / BalanceOf::<T>::from(1000); // rake 2.9% amount
            <rio_assets::Module<T>>::make_transfer_with_event(&asset_id, &who, &rake_collection_account_id, rake)?;

            let bet = amount - rake; // real bet amount
            let bet_collection_account_id = Self::bet_collection_account_id();
            <rio_assets::Module<T>>::make_transfer_with_event(&asset_id, &who, &bet_collection_account_id, bet)?;

            let item = BetItem{account_id: who.clone(), amount: bet, asset_id: asset_id,is_root: false};
            match bet_type {
                BetType::Long => {
                    <BetLongs<T>>::mutate(&game_way, |v| {
                        v.push(item);
                    });
                },
                BetType::Short => {
                    <BetShorts<T>>::mutate(&game_way, |v| {
                        v.push(item);
                    });
                },
            }

            let rake_item = BetItem{account_id: who.clone(), amount: rake, asset_id: asset_id,is_root: false};
            <BetRakes<T>>::mutate(&game_way, |v| {
                v.push(rake_item);
            });

            Self::deposit_event(RawEvent::BetingCreated(who, asset_id,game_way, Self::round_controller(&game_way),amount,bet_type));

            Ok(())
        }

        #[weight = T::WeightInfo::sudo_bet()]
        pub fn sudo_bet(origin, #[compact] asset_id: T::AssetId, game_way: GameWay, #[compact] amount: BalanceOf<T>, delegatee: <T::Lookup as StaticLookup>::Source, bet_type: BetType) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let delegatee = T::Lookup::lookup(delegatee)?;

            ensure!(Self::has_auth(&who), Error::<T>::UnAuthorized);

            ensure!(<RoundController>::contains_key(&game_way), Error::<T>::BetModeNotExisted);
            let controller = Self::game_controller(game_way).ok_or(Error::<T>::BetModeNotExisted)?;

            let now = <pallet_timestamp::Module<T>>::get();
            let game_time = controller.time_stamp + T::Moment::from(controller.duration);
            ensure!(now < game_time, Error::<T>::WaitingStage);

            ensure!(!controller.paused, Error::<T>::BetPaused);
            ensure!(!amount.is_zero(), Error::<T>::AmountZero);
            ensure!(<MaxMinBetPrice<T>>::contains_key(asset_id), Error::<T>::NotSetLimit);
            ensure!(<MaxMinBetPrice<T>>::get(asset_id).min_bet <= amount, Error::<T>::AmountSmall);
            ensure!(<MaxMinBetPrice<T>>::get(asset_id).max_bet >= amount, Error::<T>::AmountLarge);
            ensure!(<BetCollectionAssetId<T>>::get(game_way) == asset_id, Error::<T>::NotSupportAsset);
            ensure!(<rio_assets::Module<T>>::free_balance(&asset_id, &delegatee) >= amount, rio_assets::Error::<T>::InsufficientBalance);

            let rake_collection_account_id = Self::rake_collection_account_id();
            let rake = amount * BalanceOf::<T>::from(Self::fee_percent()) / BalanceOf::<T>::from(1000); // rake 2.9% amount
            <rio_assets::Module<T>>::make_transfer_with_event(&asset_id, &delegatee, &rake_collection_account_id, rake)?;

            let bet = amount - rake; // real bet amount
            let bet_collection_account_id = Self::bet_collection_account_id();
            <rio_assets::Module<T>>::make_transfer_with_event(&asset_id, &delegatee, &bet_collection_account_id, bet)?;

            let item = BetItem{account_id: delegatee.clone(), amount: bet, asset_id: asset_id,is_root: true};
            match bet_type {
                BetType::Long => {
                    <BetLongs<T>>::mutate(&game_way, |v| {
                        v.push(item);
                    });
                },
                BetType::Short => {
                    <BetShorts<T>>::mutate(&game_way, |v| {
                        v.push(item);
                    });
                },
            }

            let rake_item = BetItem{account_id: delegatee.clone(), amount: rake, asset_id: asset_id,is_root: true};
            <BetRakes<T>>::mutate(&game_way, |v| {
                v.push(rake_item);
            });

            Self::deposit_event(RawEvent::BetingCreated(delegatee, asset_id,game_way, Self::round_controller(game_way),amount,bet_type));

            Ok(())
        }

        // duration: millisecond
        #[weight = T::WeightInfo::add_game_way()]
        pub fn add_game_way(origin, game_way: GameWay, #[compact] asset_id: T::AssetId, #[compact] wait_time: u32, #[compact] duration: u32) -> DispatchResult {
            // ensure_root(origin)?;
            let who = ensure_signed(origin)?;
            ensure!(Self::has_auth(&who), Error::<T>::UnAuthorized);
            ensure!(!<RoundController>::contains_key(game_way), Error::<T>::BitModeExisted);
            ensure!(duration > 0,  Error::<T>::DurationZero);

            <BillingBtcPrice>::insert(game_way,0);
            <RoundController>::insert(game_way, 1);
            <LockResumeController>::insert(game_way, false);
            <BetingBtcPrice>::insert(game_way,<rio_price::Module<T>>::current_price());
            <GameController<T>>::insert(game_way,GameControlItem {
                paused: true,
                duration: duration,
                wait_time: wait_time,
                time_stamp: <pallet_timestamp::Module<T>>::get()
            });

            <BetCollectionAssetId<T>>::insert(&game_way,asset_id);

            let info = GameWayInfo{game_way: game_way,asset_id: asset_id,duration: duration,wait_time: wait_time};

            <GameAllType<T>>::mutate(|v| v.push(info.clone()));

            Self::deposit_event(RawEvent::GameWayChange(info,GameChangeType::Add));

            Ok(())
        }

        #[weight = T::WeightInfo::force_bet_end()]
        pub fn force_bet_end(origin, game_way: GameWay) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Self::has_auth(&who), Error::<T>::UnAuthorized);

            ensure!(<RoundController>::contains_key(game_way), Error::<T>::BetModeNotExisted);

            Self::round_end_billing(game_way);

            Ok(())
        }
    }
}

#[rustfmt::skip]
decl_event!(
    pub enum Event<T>
    where
    AccountId = <T as frame_system::Trait>::AccountId,
    AssetId = <T as rio_assets::Trait>::AssetId,
    Balance = BalanceOf<T>,
    GameWayInfo = GameWayInfo<<T as rio_assets::Trait>::AssetId>,
    {
        // bet event
        // bet account_id
        // bet asset_id
        // bet game way
        // bet round_id
        // bet total
        // bet type
        BetingCreated(AccountId, AssetId,GameWay,RoundId,Balance,BetType),
        
        // billing event
        // account_id
        // win total
        // 
        // game type
        // round
        // billing type
        // bet price for bet
        // btc price for billing
        BetBilling(AccountId,Balance,Balance,GameWay,RoundId,BillingType,Price, Price),

        // recommend reward event
        RecommendReward(AccountId,Balance,GameWay,RoundId,AccountId),

        // game change event
        GameWayChange(GameWayInfo,GameChangeType),

        // game begin event
        GameStatus(GameWay,RoundId,GameStatusType),

        // bind recommend event
        RecommendStatus(RecommendStatusType),
    }
);

impl<T: Trait> Module<T> {
    fn has_auth(account_id: &T::AccountId) -> bool {
        Self::admins(account_id)
    }

    fn clean_bets(game_way: GameWay) {
        <BetLongs<T>>::mutate(&game_way, |v| {
            v.truncate(0);
        });

        <BetShorts<T>>::mutate(&game_way, |v| {
            v.truncate(0);
        });

        <BetRakes<T>>::mutate(&game_way, |v| {
            v.truncate(0);
        });
    }

    fn set_subordinate(parent: T::AccountId, child: T::AccountId) {
        <RecommendSubordinate<T>>::mutate(&parent, |v| {
            if !v.contains(&child) {
                v.push(child);
            }
        });
    }

    fn next_round(game_way: GameWay) {
        <RoundController>::mutate(&game_way, |v| {
            *v += 1;
        });
    }

    fn restart_game(game_way: GameWay) {
        let mut bet = <BillingBtcPrice>::get(game_way);
        if bet.is_zero() {
            bet = <rio_price::Module<T>>::current_price();
        }

        Self::next_round(game_way);
        Self::set_resume_time(game_way);
        Self::set_bet_btc_price(game_way, bet);

        // Self::next_round(game_way);
        Self::set_puase_status(game_way, false);

        Self::deposit_event(RawEvent::GameStatus(
            game_way,
            Self::round_controller(&game_way),
            GameStatusType::Begin,
        ));
    }

    fn lock_resume(game_way: GameWay) {
        <LockResumeController>::mutate(&game_way, |v| {
            let org = *v;
            *v = true;
            org
        });
    }

    fn unlock_resume(game_way: GameWay) {
        <LockResumeController>::mutate(&game_way, |v| {
            let org = *v;
            *v = false;
            org
        });
    }

    fn set_bet_btc_price(game_way: GameWay, price: Price) {
        <BetingBtcPrice>::mutate(&game_way, |v| {
            *v = price;
        });
    }

    fn set_billing_btc_price(game_way: GameWay, price: Price) {
        <BillingBtcPrice>::mutate(&game_way, |v| {
            *v = price;
        });
    }

    fn set_puase_status(game_way: GameWay, status: bool) {
        <GameController<T>>::mutate(game_way, |op| {
            if let Some(v) = op {
                v.paused = status;
            }
        });
    }

    fn set_resume_time(game_way: GameWay) {
        <GameController<T>>::mutate(game_way, |op| {
            if let Some(v) = op {
                v.time_stamp = <pallet_timestamp::Module<T>>::get();
            }
        });
    }

    fn reset_limit_bet_price(
        asset_id: T::AssetId,
        min_price: BalanceOf<T>,
        max_price: BalanceOf<T>,
    ) {
        <MaxMinBetPrice<T>>::insert(
            asset_id,
            BetPriceLimit {
                min_bet: min_price,
                max_bet: max_price,
            },
        );
    }

    fn lose_win_billing(game_way: GameWay) {
        let round = Self::round_controller(&game_way);

        let bet_price = <BetingBtcPrice>::get(game_way);
        let billing_price = <BillingBtcPrice>::get(game_way);
        let bet_collection_account_id = Self::bet_collection_account_id();
        let bet_collection_asset_id = Self::bet_collection_asset_id(game_way);

        let mut lose_total = BalanceOf::<T>::zero();
        let mut win_total = BalanceOf::<T>::zero();
        let mut rake_total = BalanceOf::<T>::zero();

        let rake_list = <BetRakes<T>>::get(game_way);
        let mut all_bet_map: btree_map::BTreeMap<T::AccountId, BalanceOf<T>> =
            btree_map::BTreeMap::new();
        for item in &rake_list {
            rake_total = rake_total + item.amount;

            let value = all_bet_map
                .entry(item.account_id.clone())
                .or_insert(BalanceOf::<T>::zero());
            *value += item.amount;
        }

        for (account, value) in &all_bet_map {
            if let Some(level1) = Self::recommend_relative(account) {
                let level1_reward = *value
                    * (rake_total * BalanceOf::<T>::from(Self::x_percent())
                        / BalanceOf::<T>::from(100));

                if let Some(level2) = Self::recommend_relative(&level1) {
                    let level1_rake = all_bet_map.get(&level1).unwrap();

                    let level2_reward = level1_reward
                        + *level1_rake
                            * (rake_total * BalanceOf::<T>::from(Self::y_percent())
                                / BalanceOf::<T>::from(100));

                    <rio_assets::Module<T>>::make_transfer_with_event(
                        &bet_collection_asset_id,
                        &bet_collection_account_id,
                        &level2,
                        level2_reward,
                    )
                    .or_else(|err| -> DispatchResult { Err(err) });

                    Self::deposit_event(RawEvent::RecommendReward(
                        level2.clone(),
                        level2_reward,
                        game_way,
                        round,
                        level1.clone(),
                    ));
                }

                <rio_assets::Module<T>>::make_transfer_with_event(
                    &bet_collection_asset_id,
                    &bet_collection_account_id,
                    &level1,
                    level1_reward,
                )
                .or_else(|err| -> DispatchResult { Err(err) });

                Self::deposit_event(RawEvent::RecommendReward(
                    level1.clone(),
                    level1_reward,
                    game_way,
                    round,
                    account.clone(),
                ));
            }
        }

        let billing_btc_price = <BillingBtcPrice>::get(game_way);
        let bet_btc_price = <BetingBtcPrice>::get(game_way);
        let lose_list: Vec<BetItem<T::AccountId, BalanceOf<T>, T::AssetId>>;
        let win_list: Vec<BetItem<T::AccountId, BalanceOf<T>, T::AssetId>>;

        if billing_btc_price > bet_btc_price {
            // long win
            lose_list = <BetShorts<T>>::get(game_way);
            win_list = <BetLongs<T>>::get(game_way);
        } else {
            // shorts win
            win_list = <BetShorts<T>>::get(game_way);
            lose_list = <BetLongs<T>>::get(game_way);
        }

        for item in &lose_list {
            lose_total = lose_total + item.amount;

            Self::deposit_event(RawEvent::BetBilling(
                item.account_id.clone(),
                item.amount,
                BalanceOf::<T>::zero(),
                game_way,
                round,
                BillingType::Lose,
                bet_price,
                billing_price,
            ));
        }

        for item in &win_list {
            win_total = win_total + item.amount;
        }

        for item in &win_list {
            let obtain = (lose_total / win_total) * item.amount;
            let win_bet = obtain + item.amount;

            <rio_assets::Module<T>>::make_transfer_with_event(
                &item.asset_id,
                &bet_collection_account_id,
                &item.account_id,
                win_bet,
            )
            .or_else(|err| -> DispatchResult { Err(err) });

            Self::deposit_event(RawEvent::BetBilling(
                item.account_id.clone(),
                obtain,
                item.amount,
                game_way,
                round,
                BillingType::Win,
                bet_price,
                billing_price,
            ));
        }
    }

    fn no_loser_win(game_way: GameWay) {
        let round = Self::round_controller(&game_way);

        let bet_price = <BetingBtcPrice>::get(game_way);
        let billing_price = <BillingBtcPrice>::get(game_way);

        let long_list = <BetLongs<T>>::get(game_way);
        let short_list = <BetShorts<T>>::get(game_way);

        let bet_collection_account_id = Self::bet_collection_account_id();

        for item in &long_list {
            <rio_assets::Module<T>>::make_transfer_with_event(
                &item.asset_id,
                &bet_collection_account_id,
                &item.account_id,
                item.amount,
            )
            .or_else(|err| -> DispatchResult { Err(err) });

            Self::deposit_event(RawEvent::BetBilling(
                item.account_id.clone(),
                BalanceOf::<T>::zero(),
                item.amount,
                game_way,
                round,
                BillingType::Unchanging,
                bet_price,
                billing_price,
            ));
        }

        for item in &short_list {
            <rio_assets::Module<T>>::make_transfer_with_event(
                &item.asset_id,
                &bet_collection_account_id,
                &item.account_id,
                item.amount,
            )
            .or_else(|err| -> DispatchResult { Err(err) });

            Self::deposit_event(RawEvent::BetBilling(
                item.account_id.clone(),
                BalanceOf::<T>::zero(),
                item.amount,
                game_way,
                round,
                BillingType::Unchanging,
                bet_price,
                billing_price,
            ));
        }
    }

    fn round_end_billing(game_way: GameWay) {
        // let current_price: BalanceOf<T> = 0;
        Self::lock_resume(game_way);
        Self::set_puase_status(game_way, true);
        Self::set_billing_btc_price(game_way, <rio_price::Module<T>>::current_price());

        Self::deposit_event(RawEvent::GameStatus(
            game_way,
            Self::round_controller(game_way),
            GameStatusType::End,
        ));

        // To Do
        let billing_btc = <BillingBtcPrice>::get(game_way);
        let bet_btc_price = <BetingBtcPrice>::get(game_way);

        if billing_btc == bet_btc_price
            || (0 == <BetLongs<T>>::get(game_way).len())
            || (0 == <BetLongs<T>>::get(game_way).len())
        {
            Self::no_loser_win(game_way);
        }

        Self::lose_win_billing(game_way);
        Self::clean_bets(game_way);
        Self::unlock_resume(game_way);

        Self::restart_game(game_way);
    }
}

impl<T: Trait> rio_assets::traits::OnAssetTransfer<T::AssetId, T::AccountId, BalanceOf<T>>
    for Module<T>
{
    fn on_asset_transfer(
        _asset_id: &T::AssetId,
        _from: &T::AccountId,
        _to: &T::AccountId,
        _balance: &BalanceOf<T>,
    ) -> DispatchResult {
        Ok(())
    }
}
