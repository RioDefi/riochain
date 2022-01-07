use super::*;
use frame_support::{
    impl_outer_dispatch, impl_outer_origin, parameter_types, weights::RuntimeDbWeight,
};
use rio_primitives::{Amount, Balance, CurrencyId};
use sp_core::H256;

#[allow(unused_imports)]
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, Block as BlockT, ConvertInto, IdentityLookup, Saturating, StaticLookup},
    MultiSignature, Perbill,
};

pub mod constants {
    use super::{Balance, CurrencyId, TestRuntime};

    pub const DECIMALS: u128 = 100000000; // satoshi

    pub const ROOT: <TestRuntime as frame_system::Trait>::AccountId = 999;
    pub const ALICE: <TestRuntime as frame_system::Trait>::AccountId = 2;
    pub const BOB: <TestRuntime as frame_system::Trait>::AccountId = 3;
    pub const CHRIS: <TestRuntime as frame_system::Trait>::AccountId = 4;

    pub const RIO: CurrencyId = 1;
}

use self::constants::*;
use crate as root;

pub type AccountId = u64;
pub type BlockNumber = u64;

impl_outer_origin! {
    pub enum Origin for TestRuntime where system = frame_system {}
}

impl_outer_dispatch! {
    pub enum OuterCall for TestRuntime where origin: Origin {
        self::RioRoot,
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct TestRuntime;
const AVERAGE_ON_INITIALIZE_WEIGHT: Perbill = Perbill::from_percent(10);
parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: u32 = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::one();
    pub const BlockExecutionWeight: u32 = 10;
    pub const ExtrinsicBaseWeight: u32 = 5;
    pub MaximumExtrinsicWeight: u32 =
        AvailableBlockRatio::get().saturating_sub(AVERAGE_ON_INITIALIZE_WEIGHT)
        * MaximumBlockWeight::get();
    pub const DbWeight: RuntimeDbWeight = RuntimeDbWeight {
        read: 10,
        write: 100,
    };
}

impl frame_system::Trait for TestRuntime {
    type BaseCallFilter = ();
    type Origin = Origin;
    type Call = OuterCall;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = ();
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type DbWeight = DbWeight;
    type BlockExecutionWeight = BlockExecutionWeight;
    type ExtrinsicBaseWeight = ExtrinsicBaseWeight;
    type MaximumExtrinsicWeight = MaximumExtrinsicWeight;

    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = ();
    type PalletInfo = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type AccountData = ();
    type SystemWeightInfo = ();
}

parameter_types! {
    pub const ExistentialDeposit: u64 = 0;
    pub const TransferFee: u64 = 0;
    pub const CreationFee: u64 = 0;
    pub const TransactionBaseFee: u64 = 0;
    pub const TransactionByteFee: u64 = 0;
}

impl rio_assets::Trait for TestRuntime {
    type Event = ();
    type Balance = Balance;
    type Amount = Amount;
    type CurrencyId = CurrencyId;
    type OnReceived = ();
    type WeightInfo = ();
}

impl Trait for TestRuntime {
    type Event = ();
    type Currency = RioAssets;
    type WeightInfo = ();
}

pub type System = frame_system::Module<TestRuntime>;
pub type RioAssets = rio_assets::Module<TestRuntime>;
pub type RioRoot = crate::Module<TestRuntime>;

pub struct ExtBuilder {}

impl Default for ExtBuilder {
    fn default() -> Self {
        Self {}
    }
}

impl ExtBuilder {
    pub fn build(self) -> sp_io::TestExternalities {
        new_test_ext()
    }
}

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<TestRuntime>()
        .unwrap();

    rio_assets::GenesisConfig::<TestRuntime> {
        init: vec![(
            CurrencyId::from(rio_protocol::LOCKED_RFUEL),
            rio_assets::AssetInfo {
                symbol: b"LOCKED_RFUEL".to_vec(),
                name: b"Locked RFuel Token".to_vec(),
                decimals: 12,
                desc: b"RioChain locked token".to_vec(),
                chain: rio_assets::Chain::Rio,
            },
            rio_assets::Restriction::Transferable.into(),
            vec![],
        )],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    t.into()
}
