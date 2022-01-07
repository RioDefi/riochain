use super::*;
use std::{thread, time};
use support::{assert_noop, assert_ok};
// The testing primitives are very useful for avoiding having to work with signatures
// or public keys. `u64` is used as the `AccountId` and no `Signature`s are required.

#[allow(unused_imports)]
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup, OnFinalize, OnInitialize},
    Perbill,
};

use super::mock::{
    constants::*, new_test_ext, BitPoolTest, Call, ExtBuilder, Origin, System, TestRuntime,
};

fn next_block() {
    System::set_block_number(System::block_number() + 1);
    BitPoolTest::on_initialize(System::block_number());
}

#[test]
fn unittest_works() {
    dbg!("hello world");
}

fn set_admin() {
    assert_ok!(BitPoolTest::set_admin_account(
        frame_system::RawOrigin::Root.into(),
        ROOT,
        true
    ));
    assert_eq!(BitPoolTest::admins(ROOT), true);
}

#[test]
fn set_admin_account_works() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(BitPoolTest::set_admin_account(
            frame_system::RawOrigin::Root.into(),
            ROOT,
            true
        ));
        assert_eq!(BitPoolTest::admins(ROOT), true);
    });
}

#[test]
fn bind_recommend_account_works() {
    ExtBuilder::default().build().execute_with(|| {
        set_admin();

        let orig = Origin::signed(ROOT);
        assert_ok!(BitPoolTest::bind_recommend_account(orig.clone(), ALICE));
        assert_eq!(BitPoolTest::recommend_relative(ROOT), ALICE);
    });
}

#[test]
fn set_begin_btc_price_works() {
    ExtBuilder::default().build().execute_with(|| {
        set_admin();

        let orig = Origin::signed(ROOT);

        assert_ok!(BitPoolTest::add_game_way(orig.clone(), 1, 10800));
        let game_list = BitPoolTest::game_all_type();
        assert_eq!(game_list.len(), 1);

        assert_ok!(BitPoolTest::set_begin_btc_price(
            orig.clone(),
            1,
            9304_000000
        ));
        assert_eq!(BitPoolTest::beting_btc_price(1), 9304_000000);
    });
}

#[test]
fn set_bet_collection_account_works() {
    ExtBuilder::default().build().execute_with(|| {
        set_admin();

        let orig = Origin::signed(ROOT);
        assert_ok!(BitPoolTest::set_bet_collection_account(orig.clone(), ALICE));
        assert_eq!(BitPoolTest::bet_collection_account_id(), ALICE);
    });
}

#[test]
fn set_rake_collection_account_works() {
    ExtBuilder::default().build().execute_with(|| {
        set_admin();

        let orig = Origin::signed(ROOT);
        assert_ok!(BitPoolTest::set_rake_collection_account(
            orig.clone(),
            ALICE
        ));
        assert_eq!(BitPoolTest::rake_collection_account_id(), ALICE);
    });
}

#[test]
fn set_bet_collection_asset_id_works() {
    ExtBuilder::default().build().execute_with(|| {
        set_admin();

        let orig = Origin::signed(ROOT);

        assert_ok!(BitPoolTest::add_game_way(orig.clone(), 1, 10800));
        let game_list = BitPoolTest::game_all_type();
        assert_eq!(game_list.len(), 1);

        assert_ok!(BitPoolTest::set_bet_collection_asset_id(
            orig.clone(),
            RIO_ASSET_ID,
            1
        ));
        assert_eq!(BitPoolTest::bet_collection_asset_id(1), RIO_ASSET_ID);
    });
}

#[test]
fn set_xy_percent_works() {
    ExtBuilder::default().build().execute_with(|| {
        set_admin();

        let orig = Origin::signed(ROOT);
        assert_ok!(BitPoolTest::set_xy_percent(orig.clone(), 1, 1));
        assert_eq!(BitPoolTest::x_percent(), 1);
        assert_eq!(BitPoolTest::y_percent(), 1);
    });
}

#[test]
fn set_fee_percent_works() {
    ExtBuilder::default().build().execute_with(|| {
        set_admin();

        let orig = Origin::signed(ROOT);
        let fee = 29;

        assert_ok!(BitPoolTest::set_fee_percent(orig.clone(), fee));
        assert_eq!(BitPoolTest::fee_percent(), fee);
    });
}

#[test]
fn set_min_bet_price_works() {
    ExtBuilder::default().build().execute_with(|| {
        set_admin();

        let orig = Origin::signed(ROOT);
        let min = 0_0001;
        let max = 5_0000;

        assert_ok!(BitPoolTest::set_min_bet_price(
            orig.clone(),
            RIO_ASSET_ID,
            min,
            max
        ));

        let limit = BitPoolTest::max_min_bet_price(RIO_ASSET_ID);
        assert_eq!(limit.min_bet, min);
        assert_eq!(limit.max_bet, max);
    });
}

#[test]
fn add_game_way_works() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(BitPoolTest::set_admin_account(
            frame_system::RawOrigin::Root.into(),
            ROOT,
            true
        ));
        assert_eq!(BitPoolTest::admins(ROOT), true);

        let orig = Origin::signed(ROOT);
        assert_ok!(BitPoolTest::add_game_way(orig.clone(), 1, 10800));
        let game_list = BitPoolTest::game_all_type();
        assert_eq!(game_list.len(), 1);

        let game = BitPoolTest::game_controller(1);
        assert_eq!(game.paused, true);
        assert_eq!(game.duration, 10800);
    });
}

#[test]
fn pause_works() {
    ExtBuilder::default().build().execute_with(|| {
        set_admin();

        let orig = Origin::signed(ROOT);
        assert_ok!(BitPoolTest::add_game_way(orig.clone(), 1, 10800));
        let game_list = BitPoolTest::game_all_type();
        assert_eq!(game_list.len(), 1);

        assert_ok!(BitPoolTest::set_begin_btc_price(
            orig.clone(),
            1,
            9304_000000
        ));
        assert_eq!(BitPoolTest::beting_btc_price(1), 9304_000000);

        assert_ok!(BitPoolTest::set_bet_collection_account(orig.clone(), ALICE));
        assert_eq!(BitPoolTest::bet_collection_account_id(), ALICE);

        assert_ok!(BitPoolTest::set_rake_collection_account(
            orig.clone(),
            ALICE
        ));
        assert_eq!(BitPoolTest::rake_collection_account_id(), ALICE);

        assert_ok!(BitPoolTest::set_bet_collection_asset_id(
            orig.clone(),
            RIO_ASSET_ID,
            1
        ));
        assert_eq!(BitPoolTest::bet_collection_asset_id(1), RIO_ASSET_ID);

        assert_ok!(BitPoolTest::set_xy_percent(orig.clone(), 1, 1));
        assert_eq!(BitPoolTest::x_percent(), 1);
        assert_eq!(BitPoolTest::y_percent(), 1);

        assert_ok!(BitPoolTest::set_fee_percent(orig.clone(), 29));
        assert_eq!(BitPoolTest::fee_percent(), 29);

        let min = 0_0001;
        let max = 5_0000;
        assert_ok!(BitPoolTest::set_min_bet_price(
            orig.clone(),
            RIO_ASSET_ID,
            min,
            max
        ));
        let limit = BitPoolTest::max_min_bet_price(RIO_ASSET_ID);
        assert_eq!(limit.min_bet, min);
        assert_eq!(limit.max_bet, max);

        assert_ok!(BitPoolTest::resume(orig.clone(), 1));
        assert_eq!(BitPoolTest::game_controller(1).paused, false);

        assert_ok!(BitPoolTest::pause(orig.clone(), 1));
        assert_eq!(BitPoolTest::game_controller(1).paused, true);
    });
}

#[test]
fn resume_works() {
    ExtBuilder::default().build().execute_with(|| {
        set_admin();

        let orig = Origin::signed(ROOT);
        assert_ok!(BitPoolTest::add_game_way(orig.clone(), 1, 10800));
        let game_list = BitPoolTest::game_all_type();
        assert_eq!(game_list.len(), 1);

        assert_ok!(BitPoolTest::set_begin_btc_price(
            orig.clone(),
            1,
            9304_000000
        ));
        assert_eq!(BitPoolTest::beting_btc_price(1), 9304_000000);

        assert_ok!(BitPoolTest::set_bet_collection_account(orig.clone(), ALICE));
        assert_eq!(BitPoolTest::bet_collection_account_id(), ALICE);

        assert_ok!(BitPoolTest::set_rake_collection_account(
            orig.clone(),
            ALICE
        ));
        assert_eq!(BitPoolTest::rake_collection_account_id(), ALICE);

        assert_ok!(BitPoolTest::set_bet_collection_asset_id(
            orig.clone(),
            RIO_ASSET_ID,
            1
        ));
        assert_eq!(BitPoolTest::bet_collection_asset_id(1), RIO_ASSET_ID);

        assert_ok!(BitPoolTest::set_xy_percent(orig.clone(), 1, 1));
        assert_eq!(BitPoolTest::x_percent(), 1);
        assert_eq!(BitPoolTest::y_percent(), 1);

        assert_ok!(BitPoolTest::set_fee_percent(orig.clone(), 29));
        assert_eq!(BitPoolTest::fee_percent(), 29);

        let min = 0_0001;
        let max = 5_0000;
        assert_ok!(BitPoolTest::set_min_bet_price(
            orig.clone(),
            RIO_ASSET_ID,
            min,
            max
        ));
        let limit = BitPoolTest::max_min_bet_price(RIO_ASSET_ID);
        assert_eq!(limit.min_bet, min);
        assert_eq!(limit.max_bet, max);

        assert_ok!(BitPoolTest::resume(orig.clone(), 1));
        assert_eq!(BitPoolTest::game_controller(1).paused, false);
    });
}

#[test]
fn force_bet_end_works() {
    ExtBuilder::default().build().execute_with(|| {
        set_admin();

        let orig = Origin::signed(ROOT);
        assert_ok!(BitPoolTest::add_game_way(orig.clone(), 1, 10800));
        let game_list = BitPoolTest::game_all_type();
        assert_eq!(game_list.len(), 1);

        assert_ok!(BitPoolTest::set_begin_btc_price(
            orig.clone(),
            1,
            9304_000000
        ));
        assert_eq!(BitPoolTest::beting_btc_price(1), 9304_000000);

        assert_ok!(BitPoolTest::set_bet_collection_account(orig.clone(), ALICE));
        assert_eq!(BitPoolTest::bet_collection_account_id(), ALICE);

        assert_ok!(BitPoolTest::set_rake_collection_account(
            orig.clone(),
            ALICE
        ));
        assert_eq!(BitPoolTest::rake_collection_account_id(), ALICE);

        assert_ok!(BitPoolTest::set_bet_collection_asset_id(
            orig.clone(),
            RIO_ASSET_ID,
            1
        ));
        assert_eq!(BitPoolTest::bet_collection_asset_id(1), RIO_ASSET_ID);

        assert_ok!(BitPoolTest::set_xy_percent(orig.clone(), 1, 1));
        assert_eq!(BitPoolTest::x_percent(), 1);
        assert_eq!(BitPoolTest::y_percent(), 1);

        assert_ok!(BitPoolTest::set_fee_percent(orig.clone(), 29));
        assert_eq!(BitPoolTest::fee_percent(), 29);

        let min = 0_0001;
        let max = 5_0000;
        assert_ok!(BitPoolTest::set_min_bet_price(
            orig.clone(),
            RIO_ASSET_ID,
            min,
            max
        ));
        let limit = BitPoolTest::max_min_bet_price(RIO_ASSET_ID);
        assert_eq!(limit.min_bet, min);
        assert_eq!(limit.max_bet, max);

        assert_ok!(BitPoolTest::resume(orig.clone(), 1));
        assert_eq!(BitPoolTest::game_controller(1).paused, false);

        assert_ok!(BitPoolTest::force_bet_end(orig.clone(), 1));
        let round = BitPoolTest::round_controller(1);
        assert_eq!(round, 2);
    });
}

#[test]
fn on_initialize_works() {
    ExtBuilder::default().build().execute_with(|| {
        set_admin();

        let orig = Origin::signed(ROOT);
        assert_ok!(BitPoolTest::add_game_way(orig.clone(), 1, 1));
        let game_list = BitPoolTest::game_all_type();
        assert_eq!(game_list.len(), 1);
        assert_eq!(BitPoolTest::game_controller(1).time_stamp, 0);

        // wait 5 seconds
        // let five_millis = time::Duration::new(5,0);
        // thread::sleep(five_millis);

        assert_ok!(BitPoolTest::set_begin_btc_price(
            orig.clone(),
            1,
            9304_000000
        ));
        assert_eq!(BitPoolTest::beting_btc_price(1), 9304_000000);

        assert_ok!(BitPoolTest::set_bet_collection_account(orig.clone(), ALICE));
        assert_eq!(BitPoolTest::bet_collection_account_id(), ALICE);

        assert_ok!(BitPoolTest::set_rake_collection_account(
            orig.clone(),
            ALICE
        ));
        assert_eq!(BitPoolTest::rake_collection_account_id(), ALICE);

        assert_ok!(BitPoolTest::set_bet_collection_asset_id(
            orig.clone(),
            RIO_ASSET_ID,
            1
        ));
        assert_eq!(BitPoolTest::bet_collection_asset_id(1), RIO_ASSET_ID);

        assert_ok!(BitPoolTest::set_xy_percent(orig.clone(), 1, 1));
        assert_eq!(BitPoolTest::x_percent(), 1);
        assert_eq!(BitPoolTest::y_percent(), 1);

        assert_ok!(BitPoolTest::set_fee_percent(orig.clone(), 29));
        assert_eq!(BitPoolTest::fee_percent(), 29);

        let min = 0_0001;
        let max = 5_0000;
        assert_ok!(BitPoolTest::set_min_bet_price(
            orig.clone(),
            RIO_ASSET_ID,
            min,
            max
        ));
        let limit = BitPoolTest::max_min_bet_price(RIO_ASSET_ID);
        assert_eq!(limit.min_bet, min);
        assert_eq!(limit.max_bet, max);

        assert_ok!(BitPoolTest::resume(orig.clone(), 1));
        assert_eq!(BitPoolTest::game_controller(1).paused, false);

        next_block();

        // assert_ok!(BitPoolTest::force_bet_end(orig.clone(),1));
        let round = BitPoolTest::round_controller(1);
        assert_eq!(round, 1);
    });
}
