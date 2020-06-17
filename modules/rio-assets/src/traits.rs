use super::Result;

pub trait AssetIdProvider {
    type AssetId;
    fn asset_id() -> Self::AssetId;
}

pub trait RootKeyProvider {
    type AccountId;
    fn root_key() -> Self::AccountId;
}

/// Event hook called before create
pub trait BeforeAssetCreate<AssetId> {
    fn before_asset_create(_asset_id: &AssetId) -> Result {
        Ok(())
    }
}

/// Event hook called after create
pub trait OnAssetCreate<AssetId> {
    fn on_asset_create(_asset_id: &AssetId) -> Result {
        Ok(())
    }
}

/// Event hook called before transfer
pub trait BeforeAssetTransfer<AssetId, AccountId, Balance> {
    fn before_asset_transfer(
        _asset_id: &AssetId,
        _from: &AccountId,
        _to: &AccountId,
        _balance: &Balance,
    ) -> Result {
        Ok(())
    }
}

/// Event hook called after transfer
pub trait OnAssetTransfer<AssetId, AccountId, Balance> {
    fn on_asset_transfer(
        _asset_id: &AssetId,
        _from: &AccountId,
        _to: &AccountId,
        _balance: &Balance,
    ) -> Result {
        Ok(())
    }
}

/// Event hook called before mint
pub trait BeforeAssetMint<AssetId, AccountId, Balance> {
    fn before_asset_mint(_asset_id: &AssetId, _to: &AccountId, _balance: &Balance) -> Result {
        Ok(())
    }
}

/// Event hook called after mint
pub trait OnAssetMint<AssetId, AccountId, Balance> {
    fn on_asset_mint(_asset_id: &AssetId, _to: &AccountId, _balance: &Balance) -> Result {
        Ok(())
    }
}

/// Event hook called before burn
pub trait BeforeAssetBurn<AssetId, AccountId, Balance> {
    fn before_asset_burn(_asset_id: &AssetId, _to: &AccountId, _balance: &Balance) -> Result {
        Ok(())
    }
}

/// Event hook called after burn
pub trait OnAssetBurn<AssetId, AccountId, Balance> {
    fn on_asset_burn(_asset_id: &AssetId, _to: &AccountId, _balance: &Balance) -> Result {
        Ok(())
    }
}

// empty implementations for ()
impl<A> BeforeAssetCreate<A> for () {}
impl<A> OnAssetCreate<A> for () {}
impl<A, B, C> BeforeAssetBurn<A, B, C> for () {}
impl<A, B, C> OnAssetBurn<A, B, C> for () {}
impl<A, B, C> BeforeAssetMint<A, B, C> for () {}
impl<A, B, C> OnAssetMint<A, B, C> for () {}
impl<A, B, C> BeforeAssetTransfer<A, B, C> for () {}
impl<A, B, C> OnAssetTransfer<A, B, C> for () {}
