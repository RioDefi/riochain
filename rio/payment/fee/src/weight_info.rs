use frame_support::weights::{
    constants::{RocksDbWeight as DbWeight, WEIGHT_PER_MICROS},
    Weight,
};
pub trait WeightInfo {
    fn set_account() -> Weight;
}

impl WeightInfo for () {
    fn set_account() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(84)
            .saturating_add(DbWeight::get().reads_writes(4, 2))
    }
}
