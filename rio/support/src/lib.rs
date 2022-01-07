#![cfg_attr(not(feature = "std"), no_std)]

pub mod rlog;
pub mod traits;
pub use rlog::RUNTIME_TARGET;
