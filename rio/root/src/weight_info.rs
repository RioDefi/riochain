use frame_support::weights::{
    constants::{RocksDbWeight as DbWeight, WEIGHT_PER_MICROS},
    Weight,
};
pub trait WeightInfo {
    fn modify_manager() -> Weight;
    fn issue_locked_fee() -> Weight;
    fn toggle_blacklist() -> Weight;
}

impl WeightInfo for () {
    fn modify_manager() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(84)
            .saturating_add(DbWeight::get().reads_writes(4, 2))
    }
    fn issue_locked_fee() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(84)
            .saturating_add(DbWeight::get().reads_writes(4, 2))
    }
    fn toggle_blacklist() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(84)
            .saturating_add(DbWeight::get().reads_writes(4, 2))
    }
}
