use frame_support::weights::{
    constants::{RocksDbWeight as DbWeight, WEIGHT_PER_MICROS},
    Weight,
};

pub trait WeightInfo {
    fn set_admin_account() -> Weight;
    fn bind_recommend_account() -> Weight;
    fn sudo_bind_recommend_account() -> Weight;
    fn pause() -> Weight;
    fn resume() -> Weight;
    fn set_begin_btc_price() -> Weight;
    fn set_bet_collection_account() -> Weight;
    fn set_rake_collection_account() -> Weight;
    fn set_bet_collection_asset_id() -> Weight;
    fn set_xy_percent() -> Weight;
    fn set_free_percent() -> Weight;
    fn set_min_bet_price() -> Weight;
    fn bet() -> Weight;
    fn sudo_bet() -> Weight;
    fn add_game_way() -> Weight;
    fn force_bet_end() -> Weight;
}

impl WeightInfo for () {
    fn set_admin_account() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(80)
            .saturating_add(DbWeight::get().reads_writes(2, 2))
    }

    fn bind_recommend_account() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(200)
            .saturating_add(DbWeight::get().reads_writes(1, 1))
    }
    fn sudo_bind_recommend_account() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(200)
            .saturating_add(DbWeight::get().reads_writes(1, 1))
    }
    fn pause() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(200)
            .saturating_add(DbWeight::get().reads_writes(1, 1))
    }
    fn resume() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(200)
            .saturating_add(DbWeight::get().reads_writes(1, 1))
    }
    fn set_begin_btc_price() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(200)
            .saturating_add(DbWeight::get().reads_writes(1, 1))
    }
    fn set_bet_collection_account() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(50)
            .saturating_add(DbWeight::get().reads_writes(2, 2))
    }
    fn set_rake_collection_account() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(50)
            .saturating_add(DbWeight::get().reads_writes(2, 2))
    }
    fn set_bet_collection_asset_id() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(50)
            .saturating_add(DbWeight::get().reads_writes(2, 2))
    }
    fn set_xy_percent() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(50)
            .saturating_add(DbWeight::get().reads_writes(2, 2))
    }
    fn set_free_percent() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(50)
            .saturating_add(DbWeight::get().reads_writes(2, 2))
    }
    fn set_min_bet_price() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(50)
            .saturating_add(DbWeight::get().reads_writes(2, 2))
    }
    fn bet() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(50)
            .saturating_add(DbWeight::get().reads_writes(2, 2))
    }
    fn sudo_bet() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(88)
            .saturating_add(DbWeight::get().reads_writes(2, 2))
    }
    fn add_game_way() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(50)
            .saturating_add(DbWeight::get().reads_writes(2, 2))
    }
    fn force_bet_end() -> Weight {
        WEIGHT_PER_MICROS
            .saturating_mul(80)
            .saturating_add(DbWeight::get().reads_writes(2, 2))
    }
}
