use super::*;
use crate::types::{Bip32, TxHash};
use frame_support::{assert_noop, assert_ok};

#[allow(unused_imports)]
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    DispatchError, Perbill,
};

use super::mock::{constants::*, ExtBuilder, Gateway, GatewayErr, Origin, RioAssets, System};

fn next_block() {
    System::set_block_number(System::block_number() + 1);
}

#[test]
fn unittest_works() {
    dbg!("hello gateway");
    assert_eq!(2 + 2, 4);
}

#[test]
fn set_xpub_of_asset_id_works() {
    ExtBuilder::default().build().execute_with(|| {
        let info = DepositAddrInfo::Bip32(Bip32 {
            x_pub: PUB_KEY.to_vec(),
            path: PATH_PREFIX.to_vec(),
        });
        assert_ok!(Gateway::set_deposit_addr_info_of_asset_id(
            frame_system::RawOrigin::Root.into(),
            CUR1,
            info.clone()
        ));
        assert_eq!(Gateway::deposit_addr_info_of_asset_id(CUR1), Some(info));
    });
}
#[test]
fn set_xpub_of_asset_id_non_root_fails() {
    ExtBuilder::default().build().execute_with(|| {
        let info = DepositAddrInfo::Bip32(Bip32 {
            x_pub: PUB_KEY.to_vec(),
            path: PATH_PREFIX.to_vec(),
        });
        assert_eq!(
            Gateway::set_deposit_addr_info_of_asset_id(Origin::signed(ALICE), CUR1, info),
            Err(DispatchError::BadOrigin)
        );
    });
}

#[test]
fn apply_deposit_index_works() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Gateway::apply_deposit_index(Origin::signed(ALICE)));
        // will give next index, so we subtract 1, but do it checked just as precaution
        let index = Gateway::next_deposit_index().checked_sub(1);
        assert_eq!(Gateway::deposit_index_of_account_id(ALICE), index);
    });
}

#[test]
fn apply_deposit_index_already_applied_fails() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Gateway::apply_deposit_index(Origin::signed(ALICE)));

        assert_noop!(
            Gateway::apply_deposit_index(Origin::signed(ALICE)),
            GatewayErr::AlreadyAppliedIndex,
        );
    });
}

#[test]
fn deposit_works() {
    ExtBuilder::default().build().execute_with(|| {
        let tx = TxHash::from(b"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
        assert_ok!(Gateway::deposit(
            Origin::signed(ALICE),
            ALICE,
            CUR1,
            tx,
            CASUAL_TRANSFER
        ));
        assert_eq!(RioAssets::accounts(ALICE, CUR1).free, CASUAL_TRANSFER);
    });
}

#[test]
fn deposit_no_auth_fails() {
    ExtBuilder::default().build().execute_with(|| {
        let tx = TxHash::from(b"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
        assert_noop!(
            Gateway::deposit(Origin::signed(BOB), BOB, CUR1, tx, CASUAL_TRANSFER),
            GatewayErr::UnAuthorized,
        );
    });
}

#[test]
fn deposit_not_supported_fails() {
    ExtBuilder::default().build().execute_with(|| {
        let tx = TxHash::from(b"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
        assert_noop!(
            Gateway::deposit(Origin::signed(ALICE), ALICE, CUR2, tx, CASUAL_TRANSFER),
            GatewayErr::AssetNotSupported,
        );
    });
}

#[test]
fn deposit_repeated_tx_fails() {
    ExtBuilder::default().build().execute_with(|| {
        let tx = TxHash::from(b"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
        assert_ok!(Gateway::deposit(
            Origin::signed(ALICE),
            ALICE,
            CUR1,
            tx,
            CASUAL_TRANSFER
        ));
        assert_noop!(
            Gateway::deposit(Origin::signed(ALICE), ALICE, CUR1, tx, CASUAL_TRANSFER),
            GatewayErr::TransactionRepeated,
        );
    });
}

#[test]
fn create_withdraw_request_works() {
    ExtBuilder::default().build().execute_with(|| {
        let deposit = TxHash::from(b"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
        assert_ok!(Gateway::deposit(
            Origin::signed(ALICE),
            ALICE,
            CUR1,
            deposit,
            LARGE_TRANSFER
        ));
        assert_ok!(Gateway::request_withdraw(
            Origin::signed(ALICE),
            CUR1,
            CASUAL_TRANSFER,
            ADDRESS.to_vec(),
            MEMO.to_vec()
        ));
        let curr_id = Gateway::next_withdrawal_id() - 1;
        let result = crate::types::WithdrawInfo {
            currency_id: CUR1,
            who: ALICE,
            value: CASUAL_TRANSFER,
            addr: ADDRESS.to_vec(),
            memo: MEMO.to_vec(),
        };
        assert_eq!(Gateway::pending_withdraws(0), Some(result));
    });
}

#[test]
fn create_withdraw_request_len_fails() {
    ExtBuilder::default().build().execute_with(|| {
        let deposit = TxHash::from(b"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
        assert_ok!(Gateway::deposit(
            Origin::signed(ALICE),
            ALICE,
            CUR1,
            deposit,
            LARGE_TRANSFER
        ));
        assert_noop!(
            Gateway::request_withdraw(
                Origin::signed(ALICE),
                CUR1,
                CASUAL_TRANSFER,
                BIG_STRING.to_vec(),
                MEMO.to_vec()
            ),
            GatewayErr::InvalidWithdraw,
        );
        assert_noop!(
            Gateway::request_withdraw(
                Origin::signed(ALICE),
                CUR1,
                CASUAL_TRANSFER,
                ADDRESS.to_vec(),
                BIG_STRING.to_vec()
            ),
            GatewayErr::InvalidWithdraw,
        );
    });
}

#[test]
fn approve_withdraw_request_works() {
    ExtBuilder::default().build().execute_with(|| {
        let deposit = TxHash::from(b"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
        assert_ok!(Gateway::deposit(
            Origin::signed(ALICE),
            ALICE,
            CUR1,
            deposit,
            LARGE_TRANSFER
        ));
        assert_ok!(Gateway::request_withdraw(
            Origin::signed(ALICE),
            CUR1,
            CASUAL_TRANSFER,
            ADDRESS.to_vec(),
            MEMO.to_vec()
        ));
        let curr_id = Gateway::next_withdrawal_id() - 1;
        assert_ok!(Gateway::approve_withdraw(Origin::signed(ALICE), curr_id));
        assert_eq!(
            Gateway::active_withdrawal_states(curr_id),
            Some(types::WithdrawState::Approved)
        );
    });
}

#[test]
fn reject_withdraw_request_works() {
    ExtBuilder::default().build().execute_with(|| {
        let deposit = TxHash::from(b"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
        assert_ok!(Gateway::deposit(
            Origin::signed(ALICE),
            ALICE,
            CUR1,
            deposit,
            LARGE_TRANSFER
        ));
        assert_ok!(Gateway::request_withdraw(
            Origin::signed(ALICE),
            CUR1,
            CASUAL_TRANSFER,
            ADDRESS.to_vec(),
            MEMO.to_vec()
        ));
        let curr_id = Gateway::next_withdrawal_id() - 1;
        assert_ok!(Gateway::reject_withdraw(Origin::signed(ALICE), curr_id));
    });
}

#[test]
fn finish_withdraw_request_works() {
    ExtBuilder::default().build().execute_with(|| {
        let deposit = TxHash::from(b"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
        assert_ok!(Gateway::deposit(
            Origin::signed(ALICE),
            ALICE,
            CUR1,
            deposit,
            LARGE_TRANSFER
        ));
        assert_ok!(Gateway::request_withdraw(
            Origin::signed(ALICE),
            CUR1,
            CASUAL_TRANSFER,
            ADDRESS.to_vec(),
            MEMO.to_vec()
        ));
        let curr_id = Gateway::next_withdrawal_id() - 1;
        assert_ok!(Gateway::approve_withdraw(Origin::signed(ALICE), curr_id));
        let tx_hash = TxHash::from(b"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
        assert_ok!(Gateway::finish_withdraw(
            Origin::signed(ALICE),
            curr_id,
            tx_hash
        ));
        assert_eq!(
            RioAssets::accounts(ALICE, CUR1).free,
            LARGE_TRANSFER - CASUAL_TRANSFER
        );
    });
}

#[test]
fn cancel_withdraw_request_works() {
    ExtBuilder::default().build().execute_with(|| {
        let deposit = TxHash::from(b"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
        assert_ok!(Gateway::deposit(
            Origin::signed(ALICE),
            ALICE,
            CUR1,
            deposit,
            LARGE_TRANSFER
        ));
        assert_ok!(Gateway::request_withdraw(
            Origin::signed(ALICE),
            CUR1,
            CASUAL_TRANSFER,
            ADDRESS.to_vec(),
            MEMO.to_vec()
        ));
        let curr_id = Gateway::next_withdrawal_id() - 1;
        assert_ok!(Gateway::cancel_withdraw(Origin::signed(ALICE), curr_id,));
        assert_eq!(Gateway::pending_withdraws(curr_id), None);
    });
}

#[test]
fn cancel_withdraw_request_not_applicant_failed() {
    ExtBuilder::default().build().execute_with(|| {
        let deposit = TxHash::from(b"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
        assert_ok!(Gateway::deposit(
            Origin::signed(ALICE),
            ALICE,
            CUR1,
            deposit,
            LARGE_TRANSFER
        ));
        assert_ok!(Gateway::request_withdraw(
            Origin::signed(ALICE),
            CUR1,
            CASUAL_TRANSFER,
            ADDRESS.to_vec(),
            MEMO.to_vec()
        ));
        let curr_id = Gateway::next_withdrawal_id() - 1;
        assert_noop!(
            Gateway::cancel_withdraw(Origin::signed(BOB), curr_id),
            GatewayErr::CanNotCancelOtherWithdrawals,
        );
    });
}

#[test]
fn cancel_withdraw_request_invalid_withdrawal_state_failed() {
    ExtBuilder::default().build().execute_with(|| {
        let deposit = TxHash::from(b"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
        assert_ok!(Gateway::deposit(
            Origin::signed(ALICE),
            ALICE,
            CUR1,
            deposit,
            LARGE_TRANSFER
        ));
        assert_ok!(Gateway::request_withdraw(
            Origin::signed(ALICE),
            CUR1,
            CASUAL_TRANSFER,
            ADDRESS.to_vec(),
            MEMO.to_vec()
        ));
        let curr_id = Gateway::next_withdrawal_id() - 1;
        assert_ok!(Gateway::approve_withdraw(Origin::signed(ALICE), curr_id));
        // Cannot cancel approved withdraw
        assert_noop!(
            Gateway::cancel_withdraw(Origin::signed(ALICE), curr_id),
            GatewayErr::InvalidWithdrawalState,
        );
    });
}
