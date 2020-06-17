//! The Substrate Node Template runtime. This can be compiled with `#[no_std]`, ready for Wasm.

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use aura_primitives::sr25519::AuthorityId as AuraId;
// use contracts_rpc_runtime_api::ContractExecResult;
use grandpa::fg_primitives;
use grandpa::AuthorityList as GrandpaAuthorityList;
use primitives::OpaqueMetadata;
use rstd::prelude::*;
use sp_api::impl_runtime_apis;
#[allow(unused_imports)]
use sp_runtime::traits::{
    BlakeTwo256, Block as BlockT, ConvertInto, IdentifyAccount, NumberFor, StaticLookup, Verify,
};
use sp_runtime::{
    create_runtime_str, generic, impl_opaque_keys, transaction_validity::TransactionValidity,
    ApplyExtrinsicResult,
};
use transaction_payment_rpc_runtime_api::RuntimeDispatchInfo;
#[cfg(feature = "std")]
use version::NativeVersion;
use version::RuntimeVersion;

// A few exports that help ease life for downstream crates.
use primitives::u32_trait::*;
pub use rio_assets::permissions;
pub use rio_assets::protocol::*;
pub use rio_bridge;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{Perbill, Permill};
pub use support::{
    construct_runtime, dispatch::Result as DispatchResult, parameter_types, traits::Randomness,
    weights::Weight, StorageValue,
};
pub use timestamp::Call as TimestampCall;

pub mod constants;
pub mod types;

use constants::{currency::*, time::*};
use types::*;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core datastructures.
pub mod opaque {
    use super::*;

    pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

    /// Opaque block header type.
    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// Opaque block type.
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    /// Opaque block identifier type.
    pub type BlockId = generic::BlockId<Block>;

    impl_opaque_keys! {
        pub struct SessionKeys {
            pub grandpa: Grandpa,
            pub aura: Aura,
        }
    }
}

/// This runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!(env!("CARGO_PKG_NAME")),
    impl_name: create_runtime_str!(env!("CARGO_PKG_NAME")),
    authoring_version: 1,
    spec_version: 3,
    impl_version: 3,
    apis: RUNTIME_API_VERSIONS,
};

/// The version infromation used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

parameter_types! {
    pub const BlockHashCount: BlockNumber = 250;
    pub const MaximumBlockWeight: Weight = 1_000_000_000;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    pub const MaximumBlockLength: u32 = 50 * 1024 * 1024;
    pub const Version: RuntimeVersion = VERSION;
}

impl system::Trait for Runtime {
    /// The ubiquitous origin type.
    type Origin = Origin;
    /// The aggregated dispatch type that is available for extrinsics.
    type Call = Call;
    /// The index type for storing how many extrinsics an account has signed.
    type Index = Index;
    /// The index type for blocks.
    type BlockNumber = BlockNumber;
    /// The type for hashing blocks and tries.
    type Hash = Hash;
    /// The hashing algorithm used.
    type Hashing = BlakeTwo256;
    /// The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    /// The lookup mechanism to get account ID from whatever is passed in dispatchers.
    type Lookup = Indices;
    /// The header type.
    type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// The ubiquitous event type.
    type Event = Event;
    /// Maximum number of block number to block hash mappings to keep (oldest pruned first).
    type BlockHashCount = BlockHashCount;
    /// Maximum weight of each block. With a default weight system of 1byte == 1weight, 4mb is ok.
    type MaximumBlockWeight = MaximumBlockWeight;
    /// Maximum size of all encoded transactions (in bytes) that are allowed in one block.
    type MaximumBlockLength = MaximumBlockLength;
    /// Portion of the block weight that is available to all normal transactions.
    type AvailableBlockRatio = AvailableBlockRatio;
    /// Version of the runtime.
    type Version = Version;
}

impl aura::Trait for Runtime {
    type AuthorityId = AuraId;
}

impl grandpa::Trait for Runtime {
    type Event = Event;
}

impl indices::Trait for Runtime {
    /// The type for recording indexing into the account enumeration. If this ever overflows, there
    /// will be problems!
    type AccountIndex = u32;
    /// Determine whether an account is dead.
    type IsDeadAccount = ();
    /// Use the standard means of resolving an index hint from an id.
    type ResolveHint = indices::SimpleResolveHint<Self::AccountId, Self::AccountIndex>;
    /// The ubiquitous event type.
    type Event = Event;
}

parameter_types! {
    pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl timestamp::Trait for Runtime {
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    type OnTimestampSet = Aura;
    type MinimumPeriod = MinimumPeriod;
}

impl sudo::Trait for Runtime {
    type Event = Event;
    type Proposal = Call;
}

pub struct KeyProvider;
impl rio_assets::traits::RootKeyProvider for KeyProvider {
    type AccountId = AccountId;

    fn root_key() -> Self::AccountId {
        Sudo::key()
    }
}

impl rio_assets::Trait for Runtime {
    type Event = Event;
    type Balance = Balance;
    type AssetId = AssetId;
    type RootKey = KeyProvider;
    // type BeforeAssetTransfer = RioSaving;
    type BeforeAssetTransfer = ();
    type BeforeAssetCreate = ();
    type BeforeAssetMint = ();
    type BeforeAssetBurn = ();
    // type OnAssetTransfer = OnAssetTransferHooks;
    type OnAssetTransfer = ();
    type OnAssetCreate = ();
    // type OnAssetMint = RioSaving;
    // type OnAssetBurn = RioSaving;
    type OnAssetMint = ();
    type OnAssetBurn = ();
}

impl rio_fee::Trait for Runtime {
    type Event = Event;
    type Currency = rio_assets::NativeAsset<Runtime>;
}

impl utility::Trait for Runtime {
    type Event = Event;
    type Call = Call;
}

parameter_types! {
    pub const TransactionBaseFee: Balance = 10 * CENTS;
    pub const TransactionByteFee: Balance = 0;
}
impl rio_transaction_payment::Trait for Runtime {
    type AssetId = AssetId;
    type Currency = rio_assets::NativeAsset<Runtime>;
    type OnTransactionPayment = RioFee;
    type TransactionBaseFee = TransactionBaseFee;
    type TransactionByteFee = TransactionByteFee;
    /// weight is weight, weight is not fee
    type WeightToFee = ();
    type FeeMultiplierUpdate = ();
    type BuyFeeAsset = RioFee;
}

// impl rio_saving::Trait for Runtime {
//     type Event = Event;
// }

// impl rio_loan::Trait for Runtime {
//     type Event = Event;
// }

type OracleCollective = collective::Instance2;

impl collective::Trait<OracleCollective> for Runtime {
    type Origin = Origin;
    type Proposal = Call;
    type Event = Event;
}

parameter_types! {
    pub const OracleFee: Balance = 1 * DOLLARS;
    pub const MissReportSlash: Balance = 1 * DOLLARS;
    pub const MinStaking: Balance = 1000 * DOLLARS;

    pub const Count: u16 = 3;

    pub const ReportInteval: BlockNumber = 10;
    pub const ElectionEra: BlockNumber = 10;
    pub const LockedDuration: BlockNumber = 1000;
}

impl rio_oracle::Trait for Runtime {
    type Event = Event;

    type Currency = rio_assets::NativeAsset<Runtime>;

    type OracleFee = OracleFee;
    type MissReportSlash = MissReportSlash;
    type MinStaking = MinStaking;

    type MaliciousSlashOrigin =
        collective::EnsureProportionMoreThan<_1, _2, AccountId, OracleCollective>;

    type Count = Count;

    type ReportInteval = ReportInteval;
    type ElectionEra = ElectionEra;
    type LockedDuration = LockedDuration;

    type ChangeMembers = RioOracleMembers;
}

parameter_types! {
    pub const PricePrecision: u32 = rio_price::PRICE_PRECISION;
}

impl rio_price::Trait for Runtime {
    type Event = Event;
    type OracleMixedIn = RioOracle;
    type ReportOrigin = collective::EnsureMember<AccountId, OracleCollective>;
    // type OnChange = RioLoan;
    type OnChange = ();
}

impl rio_bridge::Trait for Runtime {
    type Event = Event;
}

construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = opaque::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: system::{Module, Call, Storage, Config, Event},
        Timestamp: timestamp::{Module, Call, Storage, Inherent},
        Aura: aura::{Module, Config<T>, Inherent(Timestamp)},
        Grandpa: grandpa::{Module, Call, Storage, Config, Event},
        RandomnessCollectiveFlip: randomness_collective_flip::{Module, Call, Storage},
        // may remove in future?
        Indices: indices,
        Sudo: sudo,

        RioAssets: rio_assets::{Module, Storage, Call, Config<T>, Event<T>},
        RioFee: rio_fee::{Module, Call, Storage, Config<T>, Event<T>},
        RioTransactionPayment: rio_transaction_payment::{Module, Storage},

        RioOracle: rio_oracle::{Module, Call, Storage, Event<T>},
        RioOracleMembers: collective::<Instance2>::{Module, Call, Storage, Origin<T>, Event<T>},
        RioPrice: rio_price::{Module, Call, Storage, Event<T>},

        // RioSaving: rio_saving::{Module, Storage, Call, Config<T>, Event<T>},
        // RioLoan: rio_loan::{Module, Call, Storage, Config<T>, Event<T>, Log},

        RioBridge: rio_bridge::{Module, Call, Storage, Config<T>, Event<T>},

        Utility: utility::{Module, Call, Event},
    }
);

/// The address format for describing accounts.
pub type Address = <Indices as StaticLookup>::Source;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
    system::CheckVersion<Runtime>,
    system::CheckGenesis<Runtime>,
    system::CheckEra<Runtime>,
    system::CheckNonce<Runtime>,
    system::CheckWeight<Runtime>,
    rio_transaction_payment::ChargeTransactionPayment<Runtime>,
    // contracts::CheckBlockGasLimit<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive =
    executive::Executive<Runtime, Block, system::ChainContext<Runtime>, Runtime, AllModules>;

impl_runtime_apis! {
    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block)
        }

        fn initialize_block(header: &<Block as BlockT>::Header) {
            Executive::initialize_block(header)
        }
    }

    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            Runtime::metadata().into()
        }
    }

    impl block_builder_api::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(
            block: Block,
            data: inherents::InherentData,
        ) -> inherents::CheckInherentsResult {
            data.check_extrinsics(&block)
        }

        fn random_seed() -> <Block as BlockT>::Hash {
            RandomnessCollectiveFlip::random_seed()
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(tx: <Block as BlockT>::Extrinsic) -> TransactionValidity {
            Executive::validate_transaction(tx)
        }
    }

    impl offchain_primitives::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(number: NumberFor<Block>) {
            Executive::offchain_worker(number)
        }
    }

    impl aura_primitives::AuraApi<Block, AuraId> for Runtime {
        fn slot_duration() -> u64 {
            Aura::slot_duration()
        }

        fn authorities() -> Vec<AuraId> {
            Aura::authorities()
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            opaque::SessionKeys::generate(seed)
        }
    }

    impl fg_primitives::GrandpaApi<Block> for Runtime {
        fn grandpa_authorities() -> GrandpaAuthorityList {
            Grandpa::grandpa_authorities()
        }
    }

    // impl contracts_rpc_runtime_api::ContractsApi<Block, AccountId, Balance> for Runtime {
    //     fn call(
    //         origin: AccountId,
    //         dest: AccountId,
    //         value: Balance,
    //         gas_limit: u64,
    //         input_data: Vec<u8>,
    //     ) -> ContractExecResult {
    //         let exec_result = Contracts::bare_call(
    //             origin,
    //             dest.into(),
    //             value,
    //             gas_limit,
    //             input_data,
    //         );
    //         match exec_result {
    //             Ok(v) => ContractExecResult::Success {
    //                 status: v.status,
    //                 data: v.data,
    //             },
    //             Err(_) => ContractExecResult::Error,
    //         }
    //     }

    //     fn get_storage(
    //         address: AccountId,
    //         key: [u8; 32],
    //     ) -> contracts_rpc_runtime_api::GetStorageResult {
    //         Contracts::get_storage(address, key).map_err(|rpc_err| {
    //             use contracts::GetStorageError;
    //             use contracts_rpc_runtime_api::{GetStorageError as RpcGetStorageError};
    //             /// Map the contract error into the RPC layer error.
    //             match rpc_err {
    //                 GetStorageError::ContractDoesntExist => RpcGetStorageError::ContractDoesntExist,
    //                 GetStorageError::IsTombstone => RpcGetStorageError::IsTombstone,
    //             }
    //         })
    //     }
    // }

    impl transaction_payment_rpc_runtime_api::TransactionPaymentApi<
        Block,
        Balance,
        UncheckedExtrinsic,
      > for Runtime {
            fn query_info(uxt: UncheckedExtrinsic, len: u32) -> RuntimeDispatchInfo<Balance> {
                  RioTransactionPayment::query_info(uxt, len)
            }
      }
}
