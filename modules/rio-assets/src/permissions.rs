use rstd::prelude::*;

use codec::{Decode, Encode, Error, Input, Output};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::RuntimeDebug;
use support::dispatch::Result;

use crate::{attributes::Owner, Module, Trait};

/// Asset permissions
#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug)]
pub struct PermissionsV1<AccountId> {
    /// Who have permission to update asset permission
    pub update: Owner<AccountId>,
    /// Who have permission to mint new asset
    pub mint: Owner<AccountId>,
    /// Who have permission to burn asset
    pub burn: Owner<AccountId>,
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug)]
#[repr(u8)]
enum PermissionVersionNumber {
    V1 = 0,
}

/// Versioned asset permission
#[derive(Clone, PartialEq, Eq, RuntimeDebug)]
pub enum PermissionVersions<AccountId> {
    V1(PermissionsV1<AccountId>),
}

/// Asset permission types
pub enum PermissionType {
    /// Permission to burn asset permission
    Burn,
    /// Permission to mint new asset
    Mint,
    /// Permission to update asset
    Update,
}

/// Alias to latest asset permissions
pub type PermissionLatest<AccountId> = PermissionsV1<AccountId>;

impl<AccountId> Default for PermissionVersions<AccountId> {
    fn default() -> Self {
        PermissionVersions::V1(Default::default())
    }
}

impl<AccountId: Encode> Encode for PermissionVersions<AccountId> {
    fn encode_to<T: Output>(&self, dest: &mut T) {
        match self {
            PermissionVersions::V1(payload) => {
                dest.push(&PermissionVersionNumber::V1);
                dest.push(payload);
            }
        }
    }
}

impl<AccountId: Encode> codec::EncodeLike for PermissionVersions<AccountId> {}

impl<AccountId: Decode> Decode for PermissionVersions<AccountId> {
    fn decode<I: Input>(input: &mut I) -> core::result::Result<Self, Error> {
        let version = PermissionVersionNumber::decode(input)?;
        Ok(match version {
            PermissionVersionNumber::V1 => PermissionVersions::V1(Decode::decode(input)?),
        })
    }
}

impl<AccountId> Default for PermissionsV1<AccountId> {
    fn default() -> Self {
        PermissionsV1 {
            update: Owner::None,
            mint: Owner::None,
            burn: Owner::None,
        }
    }
}

impl<AccountId> Into<PermissionLatest<AccountId>> for PermissionVersions<AccountId> {
    fn into(self) -> PermissionLatest<AccountId> {
        match self {
            PermissionVersions::V1(v1) => v1,
        }
    }
}

/// Converts the latest permission to other version.
impl<AccountId> Into<PermissionVersions<AccountId>> for PermissionLatest<AccountId> {
    fn into(self) -> PermissionVersions<AccountId> {
        PermissionVersions::V1(self)
    }
}

#[derive(PartialEq, PartialOrd, Ord, Eq, Clone, Copy, Encode, Decode, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Restriction {
    Transferable,
}

pub fn can_do<T: Trait>(asset_id: &T::AssetId, restriction: Restriction) -> Result {
    if <Module<T>>::get_restrictions(asset_id).contains_key(&restriction) {
        Err("The asset is restricted for this action.")
    } else {
        Ok(())
    }
}
