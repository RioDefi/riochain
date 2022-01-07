//! Mocks for the prices module.

#![cfg(test)]

use super::*;
use frame_support::{
    dispatch::DispatchResult, impl_outer_event, impl_outer_origin, ord_parameter_types,
    parameter_types,
};
use frame_system::{EnsureOneOf, EnsureRoot};
use orml_oracle::Module as Oracle;
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup, FixedPointNumber, Perbill};

pub type AccountId = u128;
pub type BlockNumber = u64;

pub const ACA: CurrencyId = 0;
pub const AUSD: CurrencyId = 1;
pub const BTC: CurrencyId = 2;
pub const DOT: CurrencyId = 3;
pub const LDOT: CurrencyId = 4;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Runtime;

mod prices {
    pub use super::super::*;
}

impl_outer_event! {
    pub enum TestEvent for Runtime {
        frame_system<T>,
        prices,
    }
}

impl_outer_origin! {
    pub enum Origin for Runtime {}
}

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: u32 = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::one();
}

impl frame_system::Trait for Runtime {
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Call = ();
    type Hash = H256;
    type Hashing = ::sp_runtime::traits::BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = TestEvent;
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = ();
    type PalletInfo = ();
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type DbWeight = ();
    type BlockExecutionWeight = ();
    type ExtrinsicBaseWeight = ();
    type MaximumExtrinsicWeight = ();
    type BaseCallFilter = ();
    type SystemWeightInfo = ();
}
pub type System = frame_system::Module<Runtime>;

pub struct MockDataProvider;
impl DataProvider<CurrencyId, Price> for MockDataProvider {
    fn get(currency_id: &CurrencyId) -> Option<Price> {
        match currency_id {
            &AUSD => Some(Price::saturating_from_rational(99, 100)),
            &BTC => Some(Price::saturating_from_integer(5000)),
            &DOT => Some(Price::saturating_from_integer(100)),
            &ACA => Some(Price::zero()),
            _ => None,
        }
    }
}
impl DataFeeder<CurrencyId, Price, AccountId> for MockDataProvider {
    fn feed_value(_who: AccountId, _key: CurrencyId, _value: Price) -> DispatchResult {
        // TODO: are we supposed to actually implement this for tests?
        // Self::do_feed_values(who, vec![(key, value)])?;
        Ok(())
    }
}

ord_parameter_types! {
    pub const One: AccountId = 1;
}

parameter_types! {
    pub const GetStableCurrencyId: CurrencyId = AUSD;
    pub const GetStakingCurrencyId: CurrencyId = DOT;
    pub const GetLiquidCurrencyId: CurrencyId = LDOT;
    pub StableCurrencyFixedPrice: Price = Price::one();
}

type EnsureRootOrHalfGeneralCouncil = EnsureOneOf<
    AccountId,
    EnsureRoot<AccountId>,
    EnsureRoot<AccountId>, // todo use council module to replace it.
>;

impl Trait for Runtime {
    type Event = Event;
    type Source = MockDataProvider;
    type LockOrigin = EnsureRootOrHalfGeneralCouncil;
}

pub type PricesModule = Module<Runtime>;

pub struct ExtBuilder;

impl Default for ExtBuilder {
    fn default() -> Self {
        ExtBuilder
    }
}

impl ExtBuilder {
    pub fn build(self) -> sp_io::TestExternalities {
        let t = frame_system::GenesisConfig::default()
            .build_storage::<Runtime>()
            .unwrap();

        t.into()
    }
}
