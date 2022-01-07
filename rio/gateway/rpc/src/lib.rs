use std::collections::BTreeMap;
use std::sync::Arc;

use codec::Codec;
use serde::{Deserialize, Serialize};

use jsonrpc_core::{Error, ErrorCode, Result};
use jsonrpc_derive::rpc;

use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};

use rio_gateway_rpc_runtime_api::{
    GatewayApi as GatewayRuntimeApi, WithdrawItem as RuntimeWithdrawItem, WithdrawState,
};

pub struct Gateway<C, B> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<B>,
}

impl<C, B> Gateway<C, B> {
    /// Create new `Contracts` with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        Gateway {
            client,
            _marker: Default::default(),
        }
    }
}

#[rpc]
pub trait GatewayApi<BlockHash, CurrencyId, AccountId, Balance> {
    #[rpc(name = "riogateway_withdrawList")]
    fn withdraw_list(
        &self,
        at: Option<BlockHash>,
    ) -> Result<BTreeMap<u64, WithdrawItem<CurrencyId, AccountId, Balance>>>;

    #[rpc(name = "riogateway_pendingWithdrawList")]
    fn pending_withdraw_list(
        &self,
        at: Option<BlockHash>,
    ) -> Result<BTreeMap<u64, WithdrawItem<CurrencyId, AccountId, Balance>>>;
}

impl<C, Block, CurrencyId, AccountId, Balance>
    GatewayApi<<Block as BlockT>::Hash, CurrencyId, AccountId, Balance> for Gateway<C, Block>
where
    C: HeaderBackend<Block>,
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: GatewayRuntimeApi<Block, CurrencyId, AccountId, Balance>,
    Block: BlockT,
    CurrencyId: Clone + std::fmt::Display + Codec,
    AccountId: Clone + std::fmt::Display + Codec,
    Balance: Clone + std::fmt::Display + Codec + ToString,
{
    fn withdraw_list(
        &self,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<BTreeMap<u64, WithdrawItem<CurrencyId, AccountId, Balance>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        api.withdraw_list(&at)
            .map(|list| {
                list.into_iter()
                    .map(|(i, (item, fee))| (i, WithdrawItem::from_runtime_type(item, fee)))
                    .collect()
            })
            .map_err(runtime_error_into_rpc_err)
    }

    fn pending_withdraw_list(
        &self,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<BTreeMap<u64, WithdrawItem<CurrencyId, AccountId, Balance>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        api.pending_withdraw_list(&at)
            .map(|list| {
                list.into_iter()
                    .map(|(i, (item, fee))| (i, WithdrawItem::from_runtime_type(item, fee)))
                    .collect()
            })
            .map_err(runtime_error_into_rpc_err)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawItem<CurrencyId, AccountId, Balance> {
    pub currency_id: CurrencyId,
    pub applicant: AccountId,
    pub value: Balance,
    pub addr: String,
    pub memo: String,
    pub state: WithdrawState,
    pub fee: Balance,
}

pub fn try_hex_or_str(src: &[u8]) -> String {
    let should_as_string = src.iter().try_for_each(|c| {
        if b'!' <= *c && *c <= b'~' {
            Ok(())
        } else {
            Err(())
        }
    });
    if should_as_string.is_ok() {
        to_string(src)
    } else {
        to_hex(src)
    }
}

#[inline]
fn to_hex(s: &[u8]) -> String {
    format!("0x{}", hex::encode(s))
}

#[inline]
fn to_string(s: &[u8]) -> String {
    String::from_utf8_lossy(s).into_owned()
}

impl<CurrencyId, AccountId, Balance> WithdrawItem<CurrencyId, AccountId, Balance> {
    fn from_runtime_type(
        item: RuntimeWithdrawItem<CurrencyId, AccountId, Balance>,
        fee: Balance,
    ) -> Self {
        WithdrawItem {
            currency_id: item.currency_id,
            applicant: item.applicant,
            value: item.value,
            addr: try_hex_or_str(&item.addr),
            memo: to_string(&item.memo),
            state: item.state,
            fee,
        }
    }
}

const RUNTIME_ERROR: i64 = 1;
fn runtime_error_into_rpc_err(err: impl std::fmt::Debug) -> Error {
    Error {
        code: ErrorCode::ServerError(RUNTIME_ERROR),
        message: "Runtime trapped".into(),
        data: Some(format!("{:?}", err).into()),
    }
}
