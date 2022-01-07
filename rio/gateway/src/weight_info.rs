use frame_support::weights::{
    constants::{RocksDbWeight as DbWeight, WEIGHT_PER_MICROS},
    Weight,
};

pub trait WeightInfo {
    fn set_auth() -> Weight;
    fn register_asset() -> Weight;
    fn remove_asset() -> Weight;
    fn set_bip32_info() -> Weight;
    fn set_withdrawal_fee() -> Weight;
    fn apply_deposit_address() -> Weight;
    fn set_max_deposit_index() -> Weight;
    fn deposit() -> Weight;
    fn withdraw() -> Weight;
    fn revoke_withdraw() -> Weight;
    fn reject_withdraw() -> Weight;
    fn approve_withdraw() -> Weight;
    fn withdraw_finish() -> Weight;
    fn rebroadcast() -> Weight;
    fn modify_withdraw_state() -> Weight;
}

impl WeightInfo for () {
    fn set_auth() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(80)
            .saturating_add(DbWeight::get().reads_writes(2, 2))
    }

    fn register_asset() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(60)
            .saturating_add(DbWeight::get().reads_writes(2, 2))
    }
    fn remove_asset() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(80)
            .saturating_add(DbWeight::get().reads_writes(2, 2))
    }
    fn set_bip32_info() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(80)
            .saturating_add(DbWeight::get().reads_writes(2, 2))
    }
    fn set_withdrawal_fee() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(60)
            .saturating_add(DbWeight::get().reads_writes(2, 2))
    }
    fn apply_deposit_address() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(80)
            .saturating_add(DbWeight::get().reads_writes(2, 2))
    }
    fn set_max_deposit_index() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(80)
            .saturating_add(DbWeight::get().reads_writes(2, 2))
    }
    fn deposit() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(80)
            .saturating_add(DbWeight::get().reads_writes(2, 2))
    }
    fn withdraw() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(80)
            .saturating_add(DbWeight::get().reads_writes(2, 2))
    }
    fn revoke_withdraw() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(81)
            .saturating_add(DbWeight::get().reads_writes(2, 2))
    }
    fn reject_withdraw() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(81)
            .saturating_add(DbWeight::get().reads_writes(2, 2))
    }
    fn approve_withdraw() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(81)
            .saturating_add(DbWeight::get().reads_writes(2, 2))
    }
    fn withdraw_finish() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(50)
            .saturating_add(DbWeight::get().reads_writes(2, 2))
    }
    fn rebroadcast() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(80)
            .saturating_add(DbWeight::get().reads_writes(2, 2))
    }
    fn modify_withdraw_state() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(81)
            .saturating_add(DbWeight::get().reads_writes(2, 2))
    }
}
