use jsonrpc_core::{Error, ErrorCode};

use rio_primitives::{AccountId, CurrencyId};

pub enum RioRpcErr {
    AssetNotExist(CurrencyId),
    NotSupportDeposit(CurrencyId),
    NotApplyDeposit(AccountId),
}

const BASE_ERROR: i64 = 5000;
const ASSET: i64 = BASE_ERROR + 0;
const ASSET_INFO_NOT_EXIST: i64 = ASSET + 1;
const GATEWAY: i64 = BASE_ERROR + 100;
const NOT_SUPPORT_DEPOSIT: i64 = GATEWAY + 1;
const NOT_APPLY_DEPOSIT: i64 = GATEWAY + 2;

impl From<RioRpcErr> for Error {
    fn from(e: RioRpcErr) -> Self {
        match e {
            RioRpcErr::AssetNotExist(currency_id) => Error {
                code: ErrorCode::ServerError(ASSET_INFO_NOT_EXIST),
                message: format!("currency_id not exist: currency_id: {}", currency_id),
                data: None,
            },
            RioRpcErr::NotSupportDeposit(currency_id) => Error {
                code: ErrorCode::ServerError(NOT_SUPPORT_DEPOSIT),
                message: format!("not support deposit for this currency_id: {}", currency_id),
                data: None,
            },
            RioRpcErr::NotApplyDeposit(who) => Error {
                code: ErrorCode::ServerError(NOT_APPLY_DEPOSIT),
                message: format!("this account not apply deposit address yet: {}", who),
                data: None,
            },
        }
    }
}
