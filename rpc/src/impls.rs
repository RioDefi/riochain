use std::sync::Arc;

use codec::Decode;
use jsonrpc_core::{Error, ErrorCode, Result};

use sc_client_api::{backend::Backend, CallExecutor, StorageProvider};
use sc_service::client::Client;
use sp_api::{BlockT, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_runtime::generic::BlockId;
use sp_state_machine::Backend as Backend2;

use frame_support::StorageMap;

use rio_primitives::{AccountId, Block, CurrencyId};
use riochain_runtime::Runtime;

use crate::apis::RioApi;
use crate::errors::RioRpcErr;
use crate::types::AccountDepositAddr;

pub struct RioRpc<BE, E, RA> {
    client: Arc<Client<BE, E, Block, RA>>,
}

impl<BE, E, RA> RioRpc<BE, E, RA>
where
    BE: Backend<Block>,
    BE::State: sp_state_machine::backend::Backend<sp_runtime::traits::BlakeTwo256>,
    E: CallExecutor<Block> + Clone + Send + Sync,
    RA: Send + Sync + 'static,
    Client<BE, E, Block, RA>: Send
        + Sync
        + 'static
        + ProvideRuntimeApi<Block>
        + HeaderBackend<Block>
        + StorageProvider<Block, BE>,
{
    /// Create new `RioRpc` with the given reference to the client.
    pub fn new(client: Arc<Client<BE, E, Block, RA>>) -> Self {
        RioRpc { client }
    }
    /// Returns given block hash or best block hash if None is passed.
    fn block_or_best(&self, hash: Option<<Block as BlockT>::Hash>) -> <Block as BlockT>::Hash {
        hash.unwrap_or_else(|| self.client.info().best_hash)
    }

    fn state(&self, hash: Option<<Block as BlockT>::Hash>) -> Result<BE::State> {
        let b = BlockId::Hash(self.block_or_best(hash));
        self.client.state_at(&b).map_err(|e| Error {
            code: ErrorCode::InternalError,
            message: format!("get state for block:{:?} error:{:?}", b, e),
            data: None,
        })
    }

    fn pickout<ReturnValue: Decode>(state: &BE::State, key: &[u8]) -> Result<Option<ReturnValue>> {
        let d = state.storage(&key).map_err(|e| Error {
            code: ErrorCode::InternalError,
            message: format!("get storage for key:0x{:} error:{:?}", hex::encode(key), e),
            data: None,
        })?;
        match d {
            None => Ok(None),
            Some(value) => Decode::decode(&mut value.as_slice())
                .map(Some)
                .map_err(|e| Error {
                    code: ErrorCode::InternalError,
                    message: format!(
                        "decode storage value:0x{:?} error:{:?}",
                        value.as_slice(),
                        e
                    ),
                    data: None,
                }),
        }
    }
}

impl<BE, E, RA> RioApi<<Block as BlockT>::Hash> for RioRpc<BE, E, RA>
where
    BE: Backend<Block>,
    BE::State: sp_state_machine::backend::Backend<sp_runtime::traits::BlakeTwo256>,
    E: CallExecutor<Block> + Clone + Send + Sync,
    RA: Send + Sync + 'static,
    Client<BE, E, Block, RA>: Send
        + Sync
        + 'static
        + ProvideRuntimeApi<Block>
        + HeaderBackend<Block>
        + StorageProvider<Block, BE>,
{
    fn deposit_address(
        &self,
        who: AccountId,
        currency_id: CurrencyId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<AccountDepositAddr> {
        use rio_gateway::{Bip32, Create2, DepositAddrInfo};

        let state = self.state(at)?;
        let k = rio_gateway::DepositAddrInfoOfAssetId::<Runtime>::hashed_key_for(currency_id);
        let info = Self::pickout::<DepositAddrInfo<Vec<u8>>>(&state, &k)?
            .ok_or(RioRpcErr::NotSupportDeposit(currency_id))?;

        let k = rio_gateway::DepoistIndexOfAccountId::<Runtime>::hashed_key_for(who);
        let index = Self::pickout::<u64>(&state, &k)?;

        let info = match info {
            DepositAddrInfo::Bip32(bip32) => DepositAddrInfo::Bip32(Bip32::<String> {
                x_pub: to_string!(&bip32.x_pub),
                path: to_string!(&bip32.path),
            }),
            DepositAddrInfo::Create2(create2) => DepositAddrInfo::Create2(Create2::<String> {
                creator_address: hex::encode(&create2.creator_address),
                implementation_address: hex::encode(&create2.implementation_address),
                vault_address: hex::encode(&create2.vault_address),
            }),
        };

        let account_addr = AccountDepositAddr {
            deposit_addr_info: info,
            index,
        };

        Ok(account_addr)
    }
}
