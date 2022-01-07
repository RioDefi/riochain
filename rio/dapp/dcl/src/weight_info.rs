use frame_support::weights::{
    constants::{RocksDbWeight as DbWeight, WEIGHT_PER_MICROS},
    Weight,
};

pub trait WeightInfo {
    fn set_merchant_return_rate() -> Weight;
    fn send_coin_by_admin() -> Weight;
    fn set_usdt_asset_id() -> Weight;
    fn set_coins_asset_id() -> Weight;
    fn add_admin() -> Weight;
    fn set_vip_price() -> Weight;
    fn set_system_account() -> Weight;
    fn free_acoin() -> Weight;
    fn free_bcoin() -> Weight;
    fn mint_and_lock() -> Weight;
    fn send_b_coin() -> Weight;
    fn join() -> Weight;
    fn dcl_purchase() -> Weight;
    fn pay_for_vip() -> Weight;
    fn merchant_return() -> Weight;
}

impl WeightInfo for () {
    fn set_merchant_return_rate() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(50)
            .saturating_add(DbWeight::get().reads_writes(2, 2))
    }
    fn send_coin_by_admin() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(60)
            .saturating_add(DbWeight::get().reads_writes(2, 2))
    }
    fn set_usdt_asset_id() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(50)
            .saturating_add(DbWeight::get().reads_writes(2, 2))
    }
    fn set_coins_asset_id() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(50)
            .saturating_add(DbWeight::get().reads_writes(2, 2))
    }
    fn add_admin() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(60)
            .saturating_add(DbWeight::get().reads_writes(2, 2))
    }
    fn set_vip_price() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(80)
            .saturating_add(DbWeight::get().reads_writes(2, 2))
    }
    fn set_system_account() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(80)
            .saturating_add(DbWeight::get().reads_writes(2, 2))
    }
    fn free_acoin() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(50)
            .saturating_add(DbWeight::get().reads_writes(2, 2))
    }
    fn free_bcoin() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(50)
            .saturating_add(DbWeight::get().reads_writes(2, 2))
    }
    fn mint_and_lock() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(80)
            .saturating_add(DbWeight::get().reads_writes(2, 2))
    }
    fn send_b_coin() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(50)
            .saturating_add(DbWeight::get().reads_writes(2, 2))
    }
    fn join() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(60)
            .saturating_add(DbWeight::get().reads_writes(2, 2))
    }
    fn dcl_purchase() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(200)
            .saturating_add(DbWeight::get().reads_writes(1, 1))
    }
    fn pay_for_vip() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(200)
            .saturating_add(DbWeight::get().reads_writes(1, 1))
    }
    fn merchant_return() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(200)
            .saturating_add(DbWeight::get().reads_writes(1, 1))
    }
}
