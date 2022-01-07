#![allow(dead_code)]

use super::*;
use frame_support::{assert_noop, assert_ok};

#[allow(unused_imports)]
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, Dispatchable, IdentityLookup, OnFinalize, OnInitialize},
    Perbill,
};

use super::mock::{new_test_ext, ExtBuilder, Origin, Test};

#[test]
fn test_join() {
    ExtBuilder::default().build().execute_with(|| {
        // assert_eq!(RFUELavingTest::current_phase_id(), PHASE1);
        // assert_eq!(
        //     RFUELavingTest::phase_info(PHASE2),
        //     PhaseInfo {
        //         id: PHASE2,
        //         quota: PHASE2_QUOTA,
        //         exchange: PHASE2_EXCHANGE,
        //         iou_asset_id: Some(RSC2_ASSET_ID),
        //     }
        // );
    });
}
