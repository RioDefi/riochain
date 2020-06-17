use support::dispatch::DispatchError;

/// A trait which enables buying some fee asset using another asset.
pub trait BuyFeeAsset {
    /// The account identifier type
    type AccountId;
    /// The type to denote monetary values
    type Balance;
    /// A type with fee payment information
    type FeeExchange;

    /// Buy `amount` of fee asset for `who` using asset info from `fee_exchange.
    /// If the purchase has been successful, return Ok with sold amount
    /// deducting the actual fee in the users's specified asset id, otherwise return Err.
    /// Note: It does not charge the fee asset, that is left to a `ChargeFee` implementation
    fn buy_fee_asset(
        who: &Self::AccountId,
        amount: Self::Balance,
        fee_exchange: &Self::FeeExchange,
    ) -> Result<Self::Balance, DispatchError>;
}
