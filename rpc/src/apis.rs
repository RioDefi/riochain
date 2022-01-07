use jsonrpc_core::Result;
use jsonrpc_derive::rpc;

use rio_primitives::{AccountId, CurrencyId};

use crate::types::AccountDepositAddr;

#[rpc]
pub trait RioApi<BlockHash> {
    #[rpc(name = "riogateway_depositAddress")]
    fn deposit_address(
        &self,
        who: AccountId,
        currency_id: CurrencyId,
        at: Option<BlockHash>,
    ) -> Result<AccountDepositAddr>;
}
