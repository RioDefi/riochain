use codec::{Decode, Encode, HasCompact};
use sp_runtime::RuntimeDebug;

use crate::permissions::PermissionLatest;

/// Asset creation options.
#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug)]
pub struct AssetOptions<Balance: HasCompact, AccountId> {
    /// Initial issuance of this asset. All deposit to the creater of the asset.
    #[codec(compact)]
    pub initial_issuance: Balance,
    /// Which accounts are allowed to possess this asset.
    pub permissions: PermissionLatest<AccountId>,
}

/// Owner of an asset.
#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug)]
pub enum Owner<AccountId> {
    /// No owner.
    None,
    /// Owned by an AccountId
    Address(AccountId),
}

impl<AccountId> Default for Owner<AccountId> {
    fn default() -> Self {
        Owner::None
    }
}
