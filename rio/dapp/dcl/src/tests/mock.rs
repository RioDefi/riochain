use super::*;
use frame_support::{construct_runtime, impl_outer_origin, parameter_types};
use sp_core::{Blake2Hasher, H256};
// The testing primitives are very useful for avoiding having to work with signatures
// or public keys. `u64` is used as the `AccountId` and no `Signature`s are required.

#[allow(unused_imports)]
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup, OnFinalize, OnInitialize},
    Perbill,
};

pub use rio_test_utils::Test;

impl_outer_origin! {
    pub enum Origin for Test {}
}

impl Trait for Test {
    type Event = ();
}

pub type RioDcl = Module<Test>;

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
    let mut t = frame_frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    sudo::GenesisConfig::<Test> { key: 1 }
        .assimilate_storage(&mut t)
        .unwrap();

    // We use default for brevity, but you can configure as desired if needed.
    // balances::GenesisConfig::<Test>::default().assimilate_storage(&mut t).unwrap();
    // GenesisConfig::<Test> {
    //     phase_infos: vec![
    //         (PHASE1_QUOTA, PHASE1_EXCHANGE),
    //         (PHASE2_QUOTA, PHASE2_EXCHANGE),
    //         (PHASE3_QUOTA, PHASE3_EXCHANGE),
    //         (PHASE4_QUOTA, PHASE4_EXCHANGE),
    //         (PHASE5_QUOTA, PHASE5_EXCHANGE),
    //     ],
    //     collection_account_id: COLLECTION_ACCOUNT_ID,
    // }
    // .assimilate_storage(&mut t)
    // .unwrap();

    t.into()
}
