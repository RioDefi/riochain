#![cfg_attr(not(feature = "std"), no_std)]

use rstd::prelude::*;
use sp_runtime::traits::EnsureOrigin;
use support::traits::{ChangeMembers, Currency, Get};
use support::{decl_event, decl_module, decl_storage, StorageMap};

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    /// Currency type
    type Currency: Currency<Self::AccountId>;

    /// The amount of fee that should be paid to each oracle during each reporting cycle.
    type OracleFee: Get<BalanceOf<Self>>;

    /// The amount that'll be slashed if one oracle missed its reporting window.
    type MissReportSlash: Get<BalanceOf<Self>>;

    /// The minimum amount to stake for an oracle candidate.
    type MinStaking: Get<BalanceOf<Self>>;

    /// The origin that's responsible for slashing malicious oracles.
    type MaliciousSlashOrigin: EnsureOrigin<Self::Origin>;

    /// The maxium count of working oracles.
    type Count: Get<u16>;

    /// The duration in which oracles should report and be paid.
    type ReportInteval: Get<Self::BlockNumber>;

    /// The duration between oracle elections.
    type ElectionEra: Get<Self::BlockNumber>;

    /// The locked time of staked amount.
    type LockedDuration: Get<Self::BlockNumber>;

    /// The actual oracle membership management type. (Usually the `srml_collective::Trait`)
    type ChangeMembers: ChangeMembers<Self::AccountId>;
}

/// Business module should use this trait to
/// communicate with oracle module in order to decouple them.
pub trait OracleMixedIn<T: system::Trait> {
    /// Tell oracle module that an event is reported by a speicifc oracle.
    fn on_witnessed(who: &T::AccountId);
    /// Predicate if one oracle is valid.
    fn is_valid(who: &T::AccountId) -> bool;
}

decl_storage! {
    trait Store for Module<T: Trait> as RioOracleStorage {
        /// Acting oracles.
        Oracles get(oracles): Vec<T::AccountId>;

        /// Blockstamp of each oracle's last event report.
        WitnessReport get(witness_report): map T::AccountId => T::BlockNumber;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        const OracleFee: BalanceOf<T> = T::OracleFee::get();
        const MissReportSlash: BalanceOf<T> = T::MissReportSlash::get();
        const MinStaking: BalanceOf<T> = T::MinStaking::get();
        const Count: u16 = T::Count::get();
        const ElectionEra: T::BlockNumber = T::ElectionEra::get();
        const ReportInteval: T::BlockNumber = T::ReportInteval::get();
        const LockedDuration: T::BlockNumber = T::LockedDuration::get();
    }
}

impl<T: Trait> OracleMixedIn<T> for Module<T> {
    fn on_witnessed(who: &T::AccountId) {
        let current_height = <system::Module<T>>::block_number();
        <WitnessReport<T>>::insert(who, current_height);
    }

    fn is_valid(who: &T::AccountId) -> bool {
        let report_height = Self::witness_report(who);
        report_height + T::ReportInteval::get() >= <system::Module<T>>::block_number()
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        Balance = BalanceOf<T>,
    {
        /// Amount unlocked for one oracle.
        OracleStakeReleased(AccountId, Balance),
    }
);
