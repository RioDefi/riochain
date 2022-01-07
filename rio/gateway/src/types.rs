#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use bitmask::bitmask;
use codec::{Decode, Encode};

use sp_core::H256;
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

use rio_primitives::{ChainAddress, Memo};

/// tx hash alias
pub type TxHash = H256;

bitmask! {
    #[derive(Encode, Decode)]
    #[cfg_attr(not(feature = "std"), derive(RuntimeDebug))]
    #[cfg_attr(feature = "std", derive(Deserialize, Serialize))]
    pub mask Auths: u8 where

    #[derive(Encode, Decode)]
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    flags Auth {
        Register = 1 << 0,
        Deposit = 1 << 1,
        Withdraw = 1 << 2,
        Sudo = 1 << 3,
    }
}
impl Default for Auths {
    fn default() -> Self {
        Self::none()
    }
}

#[derive(Encode, Decode, Default, PartialEq, Eq, Clone, Copy, RuntimeDebug)]
pub struct Deposit<AccountId, Balance> {
    /// the account on RIO who will receive "amount" of RBTC
    pub account_id: AccountId,
    /// RBTC 1:1 BTC
    pub amount: Balance,
}

#[derive(Encode, Decode, PartialEq, Eq, Clone, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct WithdrawInfo<CurrencyId, AccountId, Balance> {
    pub currency_id: CurrencyId,
    pub who: AccountId,
    pub value: Balance,
    pub addr: ChainAddress,
    pub memo: Memo,
}

#[derive(Encode, Decode, PartialEq, Eq, Clone, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum WithdrawState {
    Pending,
    Cancelled,
    Rejected,
    Approved,
    Success(TxHash),
    ReBroadcasted(TxHash),
}

#[derive(Encode, Decode, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct WithdrawItem<CurrencyId, AccountId, Balance> {
    pub currency_id: CurrencyId,
    pub applicant: AccountId,
    pub value: Balance,
    pub addr: ChainAddress,
    pub memo: Memo,
    pub state: WithdrawState,
}

#[derive(RuntimeDebug)]
pub enum WithdrawPhase {
    First,
    Second,
}

#[derive(Encode, Decode, PartialEq, Eq, Clone, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Deserialize, Serialize))]
pub enum DepositAddrInfo<T: AsRef<[u8]>> {
    #[cfg_attr(feature = "std", serde(rename = "bip32"))]
    Bip32(Bip32<T>),
    #[cfg_attr(feature = "std", serde(rename = "create2"))]
    Create2(Create2<T>),
}

/// Bip32 parent pubkey and path prefix for an assets.
/// e.g. 64 bytes pubkey: 0x9eaa27...b1cac1 and bip32 path prefix: "m/",
/// and last path would use `DepoistIndexOfAccountId` to determine it.
/// Finally, one bip32 path for an account would be:
/// `Bip32.path` + `DepoistIndexOfAccountId(who)`, e.g."m/0"
/// plus pubkey `Bip32.x_pub` to derive all child pubkey.
#[derive(Encode, Decode, PartialEq, Eq, Clone, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Deserialize, Serialize))]
pub struct Bip32<T: AsRef<[u8]>> {
    #[cfg_attr(feature = "std", serde(rename = "xPub"))]
    pub x_pub: T,
    pub path: T,
}

/// Create2 type deposit address info.
/// would set creator_address/implementation_address/vault_address plus
/// salt(a number), would using `DepoistIndexOfAccountId(who)` determine it.
#[derive(Encode, Decode, PartialEq, Eq, Clone, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct Create2<T: AsRef<[u8]>> {
    pub creator_address: T,
    pub implementation_address: T,
    pub vault_address: T,
}
