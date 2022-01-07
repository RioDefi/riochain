// Copyright 2018-2020 Parity Technologies (UK) Ltd. and Centrality Investments Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! # Transaction Payment Module
//!
//! Transaction Payment Customized Error Code Constants

pub mod error_code {
    // rio-assets
    pub const ID_UNAVAILABLE: u8 = 200;
    pub const INSUFFICIENT_BALANCE: u8 = 201;
    pub const OVERFLOW_BALANCE: u8 = 202;
    pub const RESTRICTED_ACTION: u8 = 203;

    pub const UNKNOWN_BUY_FEE_ASSET: u8 = 255;

    // Matches and converts module errors, such that
    // they are propagated in this module
    pub fn buy_fee_asset_error_msg_to_code(message: &'static str) -> u8 {
        match message {
            "IdUnavailable" => ID_UNAVAILABLE,
            "InsufficientBalance" => INSUFFICIENT_BALANCE,
            "OverflowBalance" => OVERFLOW_BALANCE,
            "RestrictedAction" => RESTRICTED_ACTION,
            _ => UNKNOWN_BUY_FEE_ASSET,
        }
    }
}
