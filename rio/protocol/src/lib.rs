#![cfg_attr(not(feature = "std"), no_std)]

pub mod assets_def {
    pub const RFUEL: u32 = 0;
    pub const LOCKED_RFUEL: u32 = 1;
    pub const OM: u32 = 2;

    pub const RBTC: u32 = 100;
    pub const RLTC: u32 = 101;
    pub const RETH: u32 = 103;
    pub const RUSDT: u32 = 102;
}

pub use assets_def::*;

// assets
pub const ASSET_SYMBOL_LEN: usize = 24;
pub const ASSET_NAME_LEN: usize = 48;
pub const ASSET_DESC_LEN: usize = 128;

pub const MEMO_BYTES_LEN: usize = 80;
