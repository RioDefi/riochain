use frame_support::weights::{
    constants::{RocksDbWeight as DbWeight, WEIGHT_PER_MICROS, WEIGHT_PER_MILLIS},
    Weight,
};

pub trait WeightInfo {
    fn transfer() -> Weight;
    fn transfer_all() -> Weight;
    fn create() -> Weight;
    fn update_asset_info() -> Weight;
    fn update_restriction() -> Weight;
    fn offline_asset() -> Weight;
    fn online_asset() -> Weight;
}

impl WeightInfo for () {
    fn transfer() -> Weight {
        WEIGHT_PER_MILLIS.saturating_mul(100)
    }

    fn transfer_all() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(88)
            .saturating_add(DbWeight::get().reads_writes(4, 2))
    }
    fn create() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(88)
            .saturating_add(DbWeight::get().reads_writes(4, 2))
    }
    fn update_asset_info() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(88)
            .saturating_add(DbWeight::get().reads_writes(4, 2))
    }
    fn update_restriction() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(88)
            .saturating_add(DbWeight::get().reads_writes(4, 2))
    }
    fn offline_asset() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(88)
            .saturating_add(DbWeight::get().reads_writes(4, 2))
    }
    fn online_asset() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(88)
            .saturating_add(DbWeight::get().reads_writes(4, 2))
    }
}
