use serde::{Deserialize, Serialize};

use rio_gateway::DepositAddrInfo;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccountDepositAddr {
    pub deposit_addr_info: DepositAddrInfo<String>,
    pub index: Option<u64>,
}
