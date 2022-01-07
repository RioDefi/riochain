#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
mod weight_info;

use sp_runtime::{traits::StaticLookup, DispatchResult};
use sp_std::prelude::*;

use frame_support::{decl_error, decl_event, decl_module, decl_storage, traits::EnsureOrigin};

use orml_traits::{MultiCurrency, MultiReservableCurrency};

use rio_primitives::CurrencyId;
use rio_support::debug;

use frame_support::traits::Contains;
use weight_info::WeightInfo;

pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

    type Currency: MultiCurrency<Self::AccountId, CurrencyId = CurrencyId>
        + MultiReservableCurrency<Self::AccountId>;

    type RootOrigin: EnsureOrigin<Self::Origin>;

    type WeightInfo: WeightInfo;
}
pub type BalanceOf<T> =
    <<T as Trait>::Currency as MultiCurrency<<T as frame_system::Trait>::AccountId>>::Balance;

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Trait>::AccountId,
        Balance = BalanceOf<T>,
    {
        ModifyManager(AccountId, bool),
        LockedRFuelIssued(AccountId, Balance),
        /// An account was added to the blacklist. [who]
        Blacklisted(AccountId),
        /// An account was removed from the blacklist. [who]
        Unblacklisted(AccountId),
    }
);

decl_error! {
    /// Error for the rio-root module.
    pub enum Error for Module<T: Trait> {
        ///
        NotRegisted,
        ///
        NotValidator,
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        #[weight = T::WeightInfo::modify_manager()]
        pub fn modify_manager(origin, who: T::AccountId, add_or_remove: bool) -> DispatchResult {
            T::RootOrigin::ensure_origin(origin)?;
            Managers::<T>::mutate(|list| {
                if add_or_remove {
                    // add manager
                    if !list.contains(&who) {
                        list.push(who.clone());
                        list.sort();
                    }
                } else {
                    // remove manager
                    list.retain(|old| old != &who);
                }
            });
            Self::deposit_event(RawEvent::ModifyManager(who, add_or_remove));
            Ok(())
        }

        #[weight = T::WeightInfo::issue_locked_fee()]
        pub fn issue_locked_gas_token(origin, to: <T::Lookup as StaticLookup>::Source, #[compact] amount: BalanceOf<T>) -> DispatchResult {
            T::RootOrigin::ensure_origin(origin)?;

            let to = T::Lookup::lookup(to)?;
            let currency_id = rio_protocol::LOCKED_RFUEL.into();

            T::Currency::deposit(currency_id, &to, amount)?;
            T::Currency::reserve(currency_id, &to, amount).expect("must success after deposit locked rfuel; qed");

            debug!("[issue_locked_gas_token]|who:{:?}|amount:{:?}", to, amount);

            Self::deposit_event(RawEvent::LockedRFuelIssued(to, amount));
            Ok(())
        }

        #[weight = T::WeightInfo::toggle_blacklist()]
        pub fn toggle_blacklist(origin, who: <T::Lookup as StaticLookup>::Source, should_blacklist: bool) -> DispatchResult {
            T::RootOrigin::ensure_origin(origin)?;

            let who = T::Lookup::lookup(who)?;
            if should_blacklist {
                Blacklist::<T>::insert(who.clone(), true);
                Self::deposit_event(Event::<T>::Blacklisted(who))
            } else {
                Blacklist::<T>::remove(&who);
                Self::deposit_event(Event::<T>::Unblacklisted(who));
            }
            Ok(())
        }

    }
}

decl_storage! {
    trait Store for Module<T: Trait> as RioRoot {
        pub Managers get(fn managers) config(): Vec<T::AccountId>;

        /// The accounts that are blocked.
        pub Blacklist get(fn blacklist): map hasher(blake2_128_concat) T::AccountId => bool;
    }
}

impl<T: Trait> Contains<T::AccountId> for Module<T> {
    fn sorted_members() -> Vec<T::AccountId> {
        Self::managers()
    }
}

impl<T: Trait> Module<T> {
    /// Returns the blocked account id list.
    pub fn get_blacklist() -> Vec<T::AccountId> {
        Blacklist::<T>::iter()
            .filter_map(|(account, blocked)| if blocked { Some(account) } else { None })
            .collect()
    }
}
