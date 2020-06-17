use codec::{Decode, Encode};

/// The outer `FeeExchange` type. It is versioned to provide flexibility for future iterations
/// while maintaining backward compatibility.
#[derive(PartialEq, Eq, Clone, Encode, Decode, Debug)]
pub enum FeeExchange<Balance> {
    /// A V1 FeeExchange
    #[codec(compact)]
    V1(FeeExchangeV1<Balance>),
}

/// A v1 FeeExchange
/// Signals a fee payment requiring the different assets exchange. It is intended to
/// embed within extrinsic payload.
/// It specifies input asset ID and the max. limit of input asset to pay
#[derive(PartialEq, Eq, Clone, Encode, Decode, Debug)]
pub struct FeeExchangeV1<Balance> {
    /// The maximum `asset_id` to pay, given the exchange rate
    #[codec(compact)]
    pub max_payment: Balance,
}

impl<Balance> FeeExchangeV1<Balance> {
    /// Create a new FeeExchangeV1
    pub fn new(max_payment: Balance) -> Self {
        Self { max_payment }
    }
}

impl<Balance: Copy> FeeExchange<Balance> {
    /// Create a `FeeExchangeV1`
    pub fn new_v1(balance: Balance) -> Self {
        FeeExchange::V1(FeeExchangeV1 {
            max_payment: balance,
        })
    }

    /// Return the max. payment limit
    pub fn max_payment(&self) -> Balance {
        match self {
            FeeExchange::V1(x) => x.max_payment,
        }
    }
}
