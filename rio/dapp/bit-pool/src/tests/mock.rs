use super::*;
use primitives::{Blake2Hasher, H256};
use support::{
    construct_runtime, impl_outer_dispatch, impl_outer_event, impl_outer_origin, parameter_types,
    weights::Weight,
};
// The testing primitives are very useful for avoiding having to work with signatures
// or public keys. `u64` is used as the `AccountId` and no `Signature`s are required.
use crate::{Module, Trait};
use rio_assets;
use std::cell::RefCell;

#[allow(unused_imports)]
use sp_runtime::{
    testing::Header,
    traits::{
        BlakeTwo256, Block as BlockT, ConvertInto, IdentityLookup, OnFinalize, OnInitialize,
        StaticLookup,
    },
    MultiSignature, Perbill,
};

use primitives::u32_trait::*;

use crate as bit_pool;

pub mod constants {
    use super::TestRuntime;

    pub const DECIMALS: u128 = 100000000; // satoshi

    pub const ROOT: <TestRuntime as system::Trait>::AccountId = 999;
    pub const ALICE: <TestRuntime as system::Trait>::AccountId = 2;
    pub const BOB: <TestRuntime as system::Trait>::AccountId = 3;
    pub const CHRIS: <TestRuntime as system::Trait>::AccountId = 4;
    #[allow(dead_code)]
    pub const DAVE: <TestRuntime as system::Trait>::AccountId = 5;
    pub const TEAM: <TestRuntime as system::Trait>::AccountId = 6;
    pub const PROFIT_POOL: <TestRuntime as system::Trait>::AccountId = 7;
    pub const COLLECTION_ACCOUNT_ID: <TestRuntime as system::Trait>::AccountId = 999;

    pub const RBTC_ASSET_ID: <TestRuntime as rio_assets::Trait>::AssetId = 1;
    pub const RSC1_ASSET_ID: <TestRuntime as rio_assets::Trait>::AssetId = 2;
    pub const RSC2_ASSET_ID: <TestRuntime as rio_assets::Trait>::AssetId = 3;
    pub const RSC3_ASSET_ID: <TestRuntime as rio_assets::Trait>::AssetId = 4;
    pub const RSC4_ASSET_ID: <TestRuntime as rio_assets::Trait>::AssetId = 5;
    pub const RSC5_ASSET_ID: <TestRuntime as rio_assets::Trait>::AssetId = 6;
    pub const RBTC_ASSET_ID: <TestRuntime as rio_assets::Trait>::AssetId = 7;
    pub const RIO_ASSET_ID: <TestRuntime as rio_assets::Trait>::AssetId = 8;
}

use self::constants::*;

pub type Address = <Indices as StaticLookup>::Source;
pub type AccountId = u64;
pub type BlockNumber = u64;
pub type Balance = u128;
pub type Block = sp_runtime::generic::Block<Header, UncheckedExtrinsic>;
pub type UncheckedExtrinsic =
    sp_runtime::generic::UncheckedExtrinsic<Address, Call, MultiSignature, ()>;

construct_runtime!(
    pub enum TestRuntime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: system::{Module, Call, Event},
        RioOracleMembers: collective::<Instance1>::{Module, Call, Storage, Origin<T>, Event<T>},
        Sudo: sudo,
        RioAssets: rio_assets::{Module, Call, Storage, Config<T>, Event<T>},
        RioOracle: rio_oracle::{Module, Call, Storage, Event<T>},
        // RFUELaving: rio_saving::{Module, Call, Storage, Config<T>, Event<T>},
        BitPoolTest: bit_pool::{Module, Call, Storage, Event<T>},
        Indices: indices,
    }
);

impl indices::Trait for TestRuntime {
    type AccountIndex = u32;
    type IsDeadAccount = ();
    type ResolveHint = indices::SimpleResolveHint<Self::AccountId, Self::AccountIndex>;
    type Event = ();
}

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: u32 = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::one();
}
impl system::Trait for TestRuntime {
    type Origin = Origin;
    type Call = ();
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
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = ();
}

parameter_types! {
    pub const ExistentialDeposit: u64 = 0;
    pub const TransferFee: u64 = 0;
    pub const CreationFee: u64 = 0;
    pub const TransactionBaseFee: u64 = 0;
    pub const TransactionByteFee: u64 = 0;
}

impl sudo::Trait for TestRuntime {
    type Event = ();

    // this is a wild guess ^_^
    type Proposal = Call;
}

impl rio_assets::Trait for TestRuntime {
    type Event = ();
    type Balance = u128;
    type AssetId = u32;
    type OnAssetMint = ();
    type OnAssetCreate = ();
    type OnAssetTransfer = ();
    type OnAssetBurn = ();
    type BeforeAssetMint = ();
    type BeforeAssetCreate = ();
    type BeforeAssetTransfer = ();
    type BeforeAssetBurn = ();
    type RootKey = rio_test_utils::KeyProvider;
}

parameter_types! {
    pub const OracleFee: Balance = 1;
    pub const MissReportSlash: Balance = 1;
    pub const MinStaking: Balance = 1000;
    pub const Count: u16 = 3;
    pub const ReportInteval: BlockNumber = 10;
    pub const ElectionEra: BlockNumber = 10;
    pub const LockedDuration: BlockNumber = 1000;
}

impl rio_oracle::Trait for TestRuntime {
    type Event = ();
    type Currency = rio_assets::NativeAsset<TestRuntime>;
    type OracleFee = OracleFee;
    type MissReportSlash = MissReportSlash;
    type MinStaking = MinStaking;
    type MaliciousSlashOrigin =
        collective::EnsureProportionMoreThan<_1, _2, AccountId, OracleCollective>;
    type Count = Count;
    type ReportInteval = ReportInteval;
    type ElectionEra = ElectionEra;
    type LockedDuration = LockedDuration;
    type ChangeMembers = RioOracleMembers;
}

parameter_types! {
    pub const PricePrecision: u32 = rio_price::PRICE_PRECISION;
}

type OracleCollective = collective::Instance1;

impl collective::Trait<OracleCollective> for TestRuntime {
    type Origin = Origin;
    type Proposal = Call;
    type Event = ();
}

impl rio_price::Trait for TestRuntime {
    type Event = ();
    type OracleMixedIn = RioOracle;
    type ReportOrigin = collective::EnsureMember<AccountId, OracleCollective>;
    type OnChange = ();
}

parameter_types! {
    pub const MinimumPeriod: u64 = 1000;
}
impl timestamp::Trait for TestRuntime {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
}

impl Trait for TestRuntime {
    type Event = ();
}

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
    let mut t = system::GenesisConfig::default()
        .build_storage::<TestRuntime>()
        .unwrap();

    sudo::GenesisConfig::<TestRuntime> { key: ROOT }
        .assimilate_storage(&mut t)
        .unwrap();

    rio_assets::GenesisConfig::<TestRuntime> {
        symbols: vec![
            (RBTC_ASSET_ID, "RBTC".as_bytes().to_vec(), vec![], vec![]),
            (RBTC_ASSET_ID, "RBTC".as_bytes().to_vec(), vec![], vec![]),
            (RSC1_ASSET_ID, "RSC1".as_bytes().to_vec(), vec![], vec![]),
            (RSC2_ASSET_ID, "RSC2".as_bytes().to_vec(), vec![], vec![]),
            (RSC3_ASSET_ID, "RSC3".as_bytes().to_vec(), vec![], vec![]),
            (RSC4_ASSET_ID, "RSC4".as_bytes().to_vec(), vec![], vec![]),
            (RSC5_ASSET_ID, "RSC5".as_bytes().to_vec(), vec![], vec![]),
            (RIO_ASSET_ID, "RIO".as_bytes().to_vec(), vec![], vec![]),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    t.into()
}
