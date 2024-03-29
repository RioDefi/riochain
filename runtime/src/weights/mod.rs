// Copyright 2019-2020 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

/// A collection of weight modules used for pallets in the runtime.
pub mod frame_system;
pub mod pallet_balances;
pub mod pallet_collective;
pub mod pallet_democracy;
pub mod pallet_elections_phragmen;
pub mod pallet_identity;
pub mod pallet_im_online;
pub mod pallet_indices;
pub mod pallet_multisig;
pub mod pallet_scheduler;
pub mod pallet_session;
pub mod pallet_staking;
pub mod pallet_timestamp;
pub mod pallet_treasury;
pub mod pallet_utility;
