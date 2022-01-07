#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use codec::Encode;

/// Implementations of some helper traits passed into runtime modules as associated types.
pub mod impls;
use impls::{CurrencyToVoteHandler, SlowAdjustingFeeUpdate};
use static_assertions::const_assert;

use sp_api::impl_runtime_apis;
use sp_core::{
    crypto::KeyTypeId,
    u32_trait::{_1, _2, _3, _4},
    OpaqueMetadata,
};
#[cfg(feature = "runtime-benchmarks")]
use sp_runtime::RuntimeString;
use sp_runtime::{
    create_runtime_str,
    curve::PiecewiseLinear,
    generic, impl_opaque_keys,
    traits::{
        self, AccountIdConversion, BlakeTwo256, Block as BlockT, DispatchInfoOf, NumberFor,
        OpaqueKeys, SaturatedConversion, Saturating, SignedExtension, StaticLookup,
    },
    transaction_validity::{
        InvalidTransaction, TransactionPriority, TransactionSource, TransactionValidity,
        TransactionValidityError, ValidTransaction,
    },
    ApplyExtrinsicResult, FixedPointNumber, ModuleId, Percent, Perquintill,
};
use sp_std::{collections::btree_map::BTreeMap, prelude::*};

use pallet_grandpa::fg_primitives;
use pallet_grandpa::{AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList};
pub use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use pallet_session::historical as pallet_session_historical;

pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;

use sp_staking::SessionIndex;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

use pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo;

// A few exports that help ease life for downstream crates.
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{Perbill, Permill};

pub use frame_support::{
    construct_runtime, debug, parameter_types,
    traits::{EnsureOrigin, Filter, KeyOwnerProofSystem, LockIdentifier, Randomness},
    weights::{
        constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
        IdentityFee, Weight,
    },
    StorageValue,
};
use frame_system::{EnsureOneOf, EnsureRoot, EnsureSignedBy, RawOrigin};
pub use pallet_balances::Call as BalancesCall;
#[cfg(feature = "std")]
pub use pallet_staking::StakerStatus;
pub use pallet_staking::{Call as StakingCall, Forcing};
pub use pallet_timestamp::Call as TimestampCall;

use orml_currencies::BasicCurrencyAdapter;

pub use rio_assets;
pub use rio_gateway::{self, Bip32, Create2, DepositAddrInfo, WithdrawItem};
pub use rio_payment::{Multiplier, TargetedFeeAdjustment};
pub use rio_primitives::{
    AccountId, AccountIndex, Amount, Balance, BlockNumber, CurrencyId, DataProviderId, Hash, Index,
    Moment, Price, Signature,
};
pub use rio_protocol::*;

pub mod constants;
use crate::constants::fee::WeightToFee;
pub use constants::{currency::*, time::*};

mod weights;

impl_opaque_keys! {
    pub struct SessionKeys {
        pub aura: Aura,
        pub grandpa: Grandpa,
        pub im_online: ImOnline,
        // pub authority_discovery: AuthorityDiscovery,
    }
}

pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!(env!("CARGO_PKG_NAME")),
    impl_name: create_runtime_str!(env!("CARGO_PKG_NAME")),
    authoring_version: 1,
    spec_version: 5,
    impl_version: 2,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

#[derive(Debug, Clone, Eq, PartialEq, codec::Encode, codec::Decode)]
pub struct BaseFilter;
impl Filter<Call> for BaseFilter {
    fn filter(call: &Call) -> bool {
        // `false` means that one is not allowed to be called by transactions:
        match call {
            Call::Staking(staking_call) => match staking_call {
                StakingCall::validate(_) => false,
                _ => true,
            },
            _ => true,
        }
    }
}
pub const FORBIDDEN_CALL: u8 = 255;
pub const FORBIDDEN_ACCOUNT: u8 = 254;
impl SignedExtension for BaseFilter {
    const IDENTIFIER: &'static str = "BaseFilter";
    type AccountId = AccountId;
    type Call = Call;
    type AdditionalSigned = ();
    type Pre = ();
    fn additional_signed(&self) -> sp_std::result::Result<(), TransactionValidityError> {
        Ok(())
    }

    fn validate(
        &self,
        who: &Self::AccountId,
        call: &Self::Call,
        _info: &DispatchInfoOf<Self::Call>,
        _len: usize,
    ) -> TransactionValidity {
        if !Self::filter(&call) {
            return Err(InvalidTransaction::Custom(FORBIDDEN_CALL).into());
        }
        if RioRoot::blacklist(who) {
            return Err(InvalidTransaction::Custom(FORBIDDEN_ACCOUNT).into());
        }
        Ok(ValidTransaction::default())
    }
}

// Configure FRAME pallets to include in runtime.
const AVERAGE_ON_INITIALIZE_WEIGHT: Perbill = Perbill::from_percent(10);
parameter_types! {
    pub const BlockHashCount: BlockNumber = 2400;
    /// We allow for 2 seconds of compute with a 6 second average block time.
    pub const MaximumBlockWeight: Weight = 2 * WEIGHT_PER_SECOND;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    /// Assume 10% of weight for average on_initialize calls.
    pub MaximumExtrinsicWeight: Weight =
        AvailableBlockRatio::get().saturating_sub(AVERAGE_ON_INITIALIZE_WEIGHT)
        * MaximumBlockWeight::get();
    pub const MaximumBlockLength: u32 = 5 * 1024 * 1024;
    pub const Version: RuntimeVersion = VERSION;
}

const_assert!(
    AvailableBlockRatio::get().deconstruct() >= AVERAGE_ON_INITIALIZE_WEIGHT.deconstruct()
);

impl frame_system::Trait for Runtime {
    /// The basic call filter to use in dispatchable.
    type BaseCallFilter = BaseFilter;
    /// The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    /// The aggregated dispatch type that is available for extrinsics.
    type Call = Call;
    /// The lookup mechanism to get account ID from whatever is passed in dispatchers.
    type Lookup = Indices;
    /// The index type for storing how many extrinsics an account has signed.
    type Index = Index;
    /// The index type for blocks.
    type BlockNumber = BlockNumber;
    /// The type for hashing blocks and tries.
    type Hash = Hash;
    /// The hashing algorithm used.
    type Hashing = BlakeTwo256;
    /// The header type.
    type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// The ubiquitous event type.
    type Event = Event;
    /// The ubiquitous origin type.
    type Origin = Origin;
    /// Maximum number of block number to block hash mappings to keep (oldest pruned first).
    type BlockHashCount = BlockHashCount;
    /// Maximum weight of each block.
    type MaximumBlockWeight = MaximumBlockWeight;
    /// The weight of database operations that the runtime can invoke.
    type DbWeight = RocksDbWeight;
    /// The weight of the overhead invoked on the block import process, independent of the
    /// extrinsics included in that block.
    type BlockExecutionWeight = BlockExecutionWeight;
    /// The base weight of any extrinsic processed by the runtime, independent of the
    /// logic of that extrinsic. (Signature verification, nonce increment, fee, etc...)
    type ExtrinsicBaseWeight = ExtrinsicBaseWeight;
    /// The maximum weight that a single extrinsic of `Normal` dispatch class can have,
    /// idependent of the logic of that extrinsics. (Roughly max block weight - average on
    /// initialize cost).
    type MaximumExtrinsicWeight = MaximumExtrinsicWeight;
    /// Maximum size of all encoded transactions (in bytes) that are allowed in one block.
    type MaximumBlockLength = MaximumBlockLength;
    /// Portion of the block weight that is available to all normal transactions.
    type AvailableBlockRatio = AvailableBlockRatio;
    /// Version of the runtime.
    type Version = Version;
    /// Converts a module to the index of the module in `construct_runtime!`.
    ///
    /// This type is being generated by `construct_runtime!`.
    type PalletInfo = PalletInfo;
    /// What to do if a new account is created.
    type OnNewAccount = ();
    /// What to do if an account is fully reaped from the system.
    type OnKilledAccount = ();
    /// The data to be stored in an account.
    type AccountData = pallet_balances::AccountData<Balance>;
    /// Weight information for the extrinsics of this pallet.
    type SystemWeightInfo = weights::frame_system::WeightInfo;
}

parameter_types! {
    pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) * MaximumBlockWeight::get();
    pub const MaxScheduledPerBlock: u32 = 50;
}

impl pallet_scheduler::Trait for Runtime {
    type Event = Event;
    type Origin = Origin;
    type PalletsOrigin = OriginCaller;
    type Call = Call;
    type MaximumWeight = MaximumSchedulerWeight;
    type ScheduleOrigin = EnsureRoot<AccountId>;
    type MaxScheduledPerBlock = MaxScheduledPerBlock;
    type WeightInfo = weights::pallet_scheduler::WeightInfo;
}

impl pallet_aura::Trait for Runtime {
    type AuthorityId = AuraId;
}

parameter_types! {
    pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Trait for Runtime {
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    type OnTimestampSet = Aura;
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = weights::pallet_timestamp::WeightInfo;
}

parameter_types! {
    pub const IndexDeposit: Balance = 10 * DOLLARS;
}

impl pallet_indices::Trait for Runtime {
    type AccountIndex = AccountIndex;
    type Currency = Balances;
    type Deposit = IndexDeposit;
    type Event = Event;
    type WeightInfo = weights::pallet_indices::WeightInfo;
}

parameter_types! {
    pub const ExistentialDeposit: u128 = 0;
    // For weight estimation, we assume that the most locks on an individual account will be 50.
    // This number may need to be adjusted in the future if this assumption no longer holds true.
    pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Trait for Runtime {
    type MaxLocks = MaxLocks;
    /// The type for recording an account's balance.
    type Balance = Balance;
    type DustRemoval = ();
    /// The ubiquitous event type.
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = weights::pallet_balances::WeightInfo;
}

parameter_types! {
    pub const UncleGenerations: u32 = 0;
}

impl pallet_authorship::Trait for Runtime {
    type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
    type UncleGenerations = UncleGenerations;
    type FilterUncle = ();
    type EventHandler = (Staking, ImOnline);
}

// TODO #6469: This shouldn't be static, but a lazily cached value, not built unless needed, and
// re-built in case input parameters have changed. The `ideal_stake` should be determined by the
// amount of parachain slots being bid on: this should be around `(75 - 25.min(slots / 4))%`.
pallet_staking_reward_curve::build! {
    const REWARD_CURVE: PiecewiseLinear<'static> = curve!(
        min_inflation: 0_001_000,
        max_inflation: 0_010_000,
        // 3:2:1 staked : parachains : float.
        // while there's no parachains, then this is 75% staked : 25% float.
        ideal_stake: 0_750_000,
        falloff: 0_050_000,
        max_piece_count: 40,
        test_precision: 0_005_000,
    );
}

parameter_types! {
    // Six sessions in an era (24 hours).
    pub const SessionsPerEra: SessionIndex = 6;
    // 7 eras for unbonding (7 days).
    pub const BondingDuration: pallet_staking::EraIndex = 8;
    pub const SlashDeferDuration: pallet_staking::EraIndex = 27;
    pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
    pub const MaxNominatorRewardedPerValidator: u32 = 64;
    // last 15 minutes of the last session will be for election.
    pub const ElectionLookahead: BlockNumber = EPOCH_DURATION_IN_BLOCKS / 16;
    pub const MaxIterations: u32 = 10;
    pub MinSolutionScoreBump: Perbill = Perbill::from_rational_approximation(5u32, 10_000);
}

type SlashCancelOrigin = EnsureOneOf<
    AccountId,
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionAtLeast<_3, _4, AccountId, CouncilCollective>,
>;

impl pallet_staking::Trait for Runtime {
    type Currency = Balances;
    type UnixTime = Timestamp;
    type CurrencyToVote = CurrencyToVoteHandler;
    type RewardRemainder = Treasury;
    type Event = Event;
    type Slash = Treasury;
    type Reward = ();
    type SessionsPerEra = SessionsPerEra;
    type BondingDuration = BondingDuration;
    type SlashDeferDuration = SlashDeferDuration;
    // A super-majority of the council can cancel the slash.
    type SlashCancelOrigin = SlashCancelOrigin;
    type SessionInterface = Self;
    type RewardCurve = RewardCurve;
    type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
    type NextNewSession = Session;
    type ElectionLookahead = ElectionLookahead;
    type Call = Call;
    type UnsignedPriority = StakingUnsignedPriority;
    type MaxIterations = MaxIterations;
    type MinSolutionScoreBump = MinSolutionScoreBump;
    type WeightInfo = weights::pallet_staking::WeightInfo;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
    Call: From<LocalCall>,
{
    fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
        call: Call,
        public: <Signature as traits::Verify>::Signer,
        account: AccountId,
        nonce: Index,
    ) -> Option<(
        Call,
        <UncheckedExtrinsic as traits::Extrinsic>::SignaturePayload,
    )> {
        // take the biggest period possible.
        let period = BlockHashCount::get()
            .checked_next_power_of_two()
            .map(|c| c / 2)
            .unwrap_or(2) as u64;
        let current_block = System::block_number()
            .saturated_into::<u64>()
            // The `System::block_number` is initialized with `n+1`,
            // so the actual block number is `n`.
            .saturating_sub(1);
        let tip = 0;
        let extra: SignedExtra = (
            frame_system::CheckSpecVersion::<Runtime>::new(),
            frame_system::CheckTxVersion::<Runtime>::new(),
            frame_system::CheckGenesis::<Runtime>::new(),
            frame_system::CheckEra::<Runtime>::from(generic::Era::mortal(period, current_block)),
            frame_system::CheckNonce::<Runtime>::from(nonce),
            frame_system::CheckWeight::<Runtime>::new(),
            rio_payment::ChargeTransactionPayment::<Runtime>::from(tip, None),
            BaseFilter,
        );
        let raw_payload = SignedPayload::new(call, extra)
            .map_err(|e| {
                debug::warn!("Unable to create signed payload: {:?}", e);
            })
            .ok()?;
        let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;
        let address = Indices::unlookup(account);
        let (call, extra, _) = raw_payload.deconstruct();
        Some((call, (address, signature.into(), extra)))
    }
}

impl frame_system::offchain::SigningTypes for Runtime {
    type Public = <Signature as traits::Verify>::Signer;
    type Signature = Signature;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
    Call: From<C>,
{
    type Extrinsic = UncheckedExtrinsic;
    type OverarchingCall = Call;
}

parameter_types! {
    pub OffencesWeightSoftLimit: Weight = Perbill::from_percent(60) * MaximumBlockWeight::get();
}

impl pallet_offences::Trait for Runtime {
    type Event = Event;
    type IdentificationTuple = pallet_session_historical::IdentificationTuple<Self>;
    type OnOffenceHandler = Staking;
    type WeightSoftLimit = OffencesWeightSoftLimit;
}

impl pallet_session_historical::Trait for Runtime {
    type FullIdentification = pallet_staking::Exposure<AccountId, Balance>;
    type FullIdentificationOf = pallet_staking::ExposureOf<Runtime>;
}

parameter_types! {
    pub const Offset: BlockNumber = 0;
    pub const Period: BlockNumber = 50;
    pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(17);
}

impl pallet_session::Trait for Runtime {
    type Event = Event;
    type ValidatorId = AccountId;
    type ValidatorIdOf = pallet_staking::StashOf<Self>;
    // todo change in future
    type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
    type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
    type SessionManager = pallet_session::historical::NoteHistoricalRoot<Self, Staking>;
    type SessionHandler = <SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
    type Keys = SessionKeys;
    type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
    type WeightInfo = weights::pallet_session::WeightInfo;
}

impl pallet_grandpa::Trait for Runtime {
    type Event = Event;
    type Call = Call;

    type KeyOwnerProofSystem = Historical;

    type KeyOwnerProof =
        <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;

    type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
        KeyTypeId,
        GrandpaId,
    )>>::IdentificationTuple;

    type HandleEquivocation = ();
    type WeightInfo = ();
}

parameter_types! {
    pub WindowSize: BlockNumber = pallet_finality_tracker::DEFAULT_WINDOW_SIZE.into();
    pub ReportLatency: BlockNumber = pallet_finality_tracker::DEFAULT_REPORT_LATENCY.into();
}

impl pallet_finality_tracker::Trait for Runtime {
    type OnFinalizationStalled = ();
    type WindowSize = WindowSize;
    type ReportLatency = ReportLatency;
}

parameter_types! {
    /// Babe use EPOCH_DURATION_IN_SLOTS here, we use Aura.
    pub const SessionDuration: BlockNumber = Period::get();
    pub const ImOnlineUnsignedPriority: TransactionPriority = TransactionPriority::max_value();
    /// We prioritize im-online heartbeats over election solution submission.
    pub const StakingUnsignedPriority: TransactionPriority = TransactionPriority::max_value() / 2;
}

impl pallet_im_online::Trait for Runtime {
    type AuthorityId = ImOnlineId;
    type Event = Event;
    type SessionDuration = SessionDuration;
    type ReportUnresponsiveness = Offences;
    type UnsignedPriority = ImOnlineUnsignedPriority;
    type WeightInfo = weights::pallet_im_online::WeightInfo;
}

// Governance stuff.
parameter_types! {
    pub const LaunchPeriod: BlockNumber = 28 * DAYS;
    pub const VotingPeriod: BlockNumber = 7 * DAYS;
    pub const FastTrackVotingPeriod: BlockNumber = 3 * HOURS;
    pub const MinimumDeposit: Balance = 100 * DOLLARS;
    pub const EnactmentPeriod: BlockNumber = 28 * DAYS;
    pub const CooloffPeriod: BlockNumber = 7 * DAYS;
    // One cent: $10,000 / MB
    pub const PreimageByteDeposit: Balance = 1 * CENTS;
    pub const InstantAllowed: bool = true;
    pub const MaxVotes: u32 = 100;
}

impl pallet_democracy::Trait for Runtime {
    type Proposal = Call;
    type Event = Event;
    type Currency = Balances;
    type EnactmentPeriod = EnactmentPeriod;
    type LaunchPeriod = LaunchPeriod;
    type VotingPeriod = VotingPeriod;
    type MinimumDeposit = MinimumDeposit;
    /// A straight majority of the council can decide what their next motion is.
    type ExternalOrigin =
        pallet_collective::EnsureProportionAtLeast<_1, _2, AccountId, CouncilCollective>;
    /// A super-majority can have the next scheduled referendum be a straight majority-carries vote.
    type ExternalMajorityOrigin =
        pallet_collective::EnsureProportionAtLeast<_3, _4, AccountId, CouncilCollective>;
    /// A unanimous council can have the next scheduled referendum be a straight default-carries
    /// (NTB) vote.
    type ExternalDefaultOrigin =
        pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, CouncilCollective>;
    /// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
    /// be tabled immediately and with a shorter voting/enactment period.
    type FastTrackOrigin =
        pallet_collective::EnsureProportionAtLeast<_2, _3, AccountId, TechnicalCollective>;
    type InstantOrigin =
        pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, TechnicalCollective>;
    type InstantAllowed = InstantAllowed;
    type FastTrackVotingPeriod = FastTrackVotingPeriod;
    // To cancel a proposal which has been passed, 2/3 of the council must agree to it.
    type CancellationOrigin =
        pallet_collective::EnsureProportionAtLeast<_2, _3, AccountId, CouncilCollective>;
    // Any single technical committee member may veto a coming council proposal, however they can
    // only do it once and it lasts only for the cooloff period.
    type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechnicalCollective>;
    type CooloffPeriod = CooloffPeriod;
    type PreimageByteDeposit = PreimageByteDeposit;
    type OperationalPreimageOrigin = pallet_collective::EnsureMember<AccountId, CouncilCollective>;
    type Slash = Treasury;
    type Scheduler = Scheduler;
    type PalletsOrigin = OriginCaller;
    type MaxVotes = MaxVotes;
    type WeightInfo = weights::pallet_democracy::WeightInfo;
}

parameter_types! {
    pub const CouncilMotionDuration: BlockNumber = 3 * DAYS;
    pub const CouncilMaxProposals: u32 = 100;
    pub const CouncilMaxMembers: u32 = 100;
}

type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Trait<CouncilCollective> for Runtime {
    type Origin = Origin;
    type Proposal = Call;
    type Event = Event;
    type MotionDuration = CouncilMotionDuration;
    type MaxProposals = CouncilMaxProposals;
    type MaxMembers = CouncilMaxMembers;
    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type WeightInfo = weights::pallet_collective::WeightInfo;
}

parameter_types! {
    pub const CandidacyBond: Balance = 100 * DOLLARS;
    pub const VotingBond: Balance = 5 * DOLLARS;
    /// Weekly council elections; scaling up to monthly eventually.
    pub const TermDuration: BlockNumber = 7 * DAYS;
    pub const DesiredMembers: u32 = 5;
    pub const DesiredRunnersUp: u32 = 14;
    pub const ElectionsPhragmenModuleId: LockIdentifier = *b"phrelect";
}

// Make sure that there are no more than `MAX_MEMBERS` members elected via elections-phragmen.
const_assert!(DesiredMembers::get() <= CouncilMaxMembers::get());

impl pallet_elections_phragmen::Trait for Runtime {
    type Event = Event;
    type ModuleId = ElectionsPhragmenModuleId;
    type Currency = Balances;
    type ChangeMembers = Council;
    // NOTE: this implies that council's genesis members cannot be set directly and must come from
    // this module.
    type InitializeMembers = Council;
    type CurrencyToVote = CurrencyToVoteHandler;
    type CandidacyBond = CandidacyBond;
    type VotingBond = VotingBond;
    type LoserCandidate = ();
    type BadReport = ();
    type KickedMember = ();
    type DesiredMembers = DesiredMembers;
    type DesiredRunnersUp = DesiredRunnersUp;
    type TermDuration = TermDuration;
    type WeightInfo = weights::pallet_elections_phragmen::WeightInfo;
}

parameter_types! {
    pub const TechnicalMotionDuration: BlockNumber = 7 * DAYS;
    pub const TechnicalMaxProposals: u32 = 100;
    pub const TechnicalMaxMembers: u32 = 100;
}

type TechnicalCollective = pallet_collective::Instance2;
impl pallet_collective::Trait<TechnicalCollective> for Runtime {
    type Origin = Origin;
    type Proposal = Call;
    type Event = Event;
    type MotionDuration = TechnicalMotionDuration;
    type MaxProposals = TechnicalMaxProposals;
    type MaxMembers = TechnicalMaxMembers;
    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type WeightInfo = weights::pallet_collective::WeightInfo;
}

type EnsureRootOrHalfCouncil = EnsureOneOf<
    AccountId,
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>,
>;
impl pallet_membership::Trait<pallet_membership::Instance1> for Runtime {
    type Event = Event;
    type AddOrigin = EnsureRootOrHalfCouncil;
    type RemoveOrigin = EnsureRootOrHalfCouncil;
    type SwapOrigin = EnsureRootOrHalfCouncil;
    type ResetOrigin = EnsureRootOrHalfCouncil;
    type PrimeOrigin = EnsureRootOrHalfCouncil;
    type MembershipInitialized = TechnicalCommittee;
    type MembershipChanged = TechnicalCommittee;
}

parameter_types! {
    pub const ProposalBond: Permill = Permill::from_percent(5);
    pub const ProposalBondMinimum: Balance = 100 * DOLLARS;
    pub const SpendPeriod: BlockNumber = 24 * DAYS;
    pub const Burn: Permill = Permill::from_percent(1);
    pub const TreasuryModuleId: ModuleId = ModuleId(*b"py/trsry");

    pub const TipCountdown: BlockNumber = 1 * DAYS;
    pub const TipFindersFee: Percent = Percent::from_percent(20);
    pub const TipReportDepositBase: Balance = 1 * DOLLARS;

    pub const DataDepositPerByte: Balance = 1 * CENTS;
    pub const BountyDepositBase: Balance = 1 * DOLLARS;
    pub const BountyDepositPayoutDelay: BlockNumber = 8 * DAYS; // TODO may need to change this
    pub const BountyUpdatePeriod: BlockNumber = 90 * DAYS; // TODO may need to change this
    pub const MaximumReasonLength: u32 = 16384;
    pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
    pub const BountyValueMinimum: Balance = 10 * DOLLARS;
}

impl pallet_treasury::Trait for Runtime {
    type ModuleId = TreasuryModuleId;
    type Currency = Balances;
    type ApproveOrigin = EnsureOneOf<
        AccountId,
        EnsureRoot<AccountId>,
        pallet_collective::EnsureMembers<_4, AccountId, CouncilCollective>,
    >;
    type RejectOrigin = EnsureOneOf<
        AccountId,
        EnsureRoot<AccountId>,
        pallet_collective::EnsureMembers<_2, AccountId, CouncilCollective>,
    >;
    type Tippers = Elections;
    type TipCountdown = TipCountdown;
    type TipFindersFee = TipFindersFee;
    type TipReportDepositBase = TipReportDepositBase;
    type DataDepositPerByte = DataDepositPerByte;
    type Event = Event;
    type OnSlash = Treasury;
    type ProposalBond = ProposalBond;
    type ProposalBondMinimum = ProposalBondMinimum;
    type SpendPeriod = SpendPeriod;
    type Burn = Burn;
    type BountyDepositBase = BountyDepositBase;
    type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
    type BountyUpdatePeriod = BountyUpdatePeriod;
    type BountyCuratorDeposit = BountyCuratorDeposit;
    type BountyValueMinimum = BountyValueMinimum;
    type MaximumReasonLength = MaximumReasonLength;
    type BurnDestination = ();
    type WeightInfo = weights::pallet_treasury::WeightInfo;
}

parameter_types! {
    // Minimum 4 CENTS/byte
    pub const BasicDeposit: Balance = deposit(1, 258);
    pub const FieldDeposit: Balance = deposit(0, 66);
    pub const SubAccountDeposit: Balance = deposit(1, 53);
    pub const MaxSubAccounts: u32 = 100;
    pub const MaxAdditionalFields: u32 = 100;
    pub const MaxRegistrars: u32 = 20;
}

impl pallet_identity::Trait for Runtime {
    type Event = Event;
    type Currency = Balances;
    type BasicDeposit = BasicDeposit;
    type FieldDeposit = FieldDeposit;
    type SubAccountDeposit = SubAccountDeposit;
    type MaxSubAccounts = MaxSubAccounts;
    type MaxAdditionalFields = MaxAdditionalFields;
    type MaxRegistrars = MaxRegistrars;
    type Slashed = Treasury;
    type ForceOrigin = EnsureRootOrHalfCouncil;
    type RegistrarOrigin = EnsureRootOrHalfCouncil;
    type WeightInfo = weights::pallet_identity::WeightInfo;
}

parameter_types! {
    pub const CandidateDeposit: Balance = 10 * DOLLARS;
    pub const WrongSideDeduction: Balance = 2 * DOLLARS;
    pub const MaxStrikes: u32 = 10;
    pub const RotationPeriod: BlockNumber = 80 * HOURS;
    pub const PeriodSpend: Balance = 500 * DOLLARS;
    pub const MaxLockDuration: BlockNumber = 36 * 30 * DAYS;
    pub const ChallengePeriod: BlockNumber = 7 * DAYS;
    pub const SocietyModuleId: ModuleId = ModuleId(*b"py/socie");
}

impl pallet_society::Trait for Runtime {
    type Event = Event;
    type ModuleId = SocietyModuleId;
    type Currency = Balances;
    type Randomness = RandomnessCollectiveFlip;
    type CandidateDeposit = CandidateDeposit;
    type WrongSideDeduction = WrongSideDeduction;
    type MaxStrikes = MaxStrikes;
    type PeriodSpend = PeriodSpend;
    type MembershipChanged = ();
    type RotationPeriod = RotationPeriod;
    type MaxLockDuration = MaxLockDuration;
    type FounderSetOrigin =
        pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>;
    type SuspensionJudgementOrigin = pallet_society::EnsureFounder<Runtime>;
    type ChallengePeriod = ChallengePeriod;
}

parameter_types! {
    pub const ConfigDepositBase: Balance = 5 * DOLLARS;
    pub const FriendDepositFactor: Balance = 50 * CENTS;
    pub const MaxFriends: u16 = 9;
    pub const RecoveryDeposit: Balance = 5 * DOLLARS;
}

impl pallet_recovery::Trait for Runtime {
    type Event = Event;
    type Call = Call;
    type Currency = Balances;
    type ConfigDepositBase = ConfigDepositBase;
    type FriendDepositFactor = FriendDepositFactor;
    type MaxFriends = MaxFriends;
    type RecoveryDeposit = RecoveryDeposit;
}

impl pallet_utility::Trait for Runtime {
    type Event = Event;
    type Call = Call;
    type WeightInfo = weights::pallet_utility::WeightInfo;
}

parameter_types! {
    // One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
    pub const DepositBase: Balance = deposit(1, 88);
    // Additional storage item size of 32 bytes.
    pub const DepositFactor: Balance = deposit(0, 32);
    pub const MaxSignatories: u16 = 100;
}

impl pallet_multisig::Trait for Runtime {
    type Event = Event;
    type Call = Call;
    type Currency = Balances;
    type DepositBase = DepositBase;
    type DepositFactor = DepositFactor;
    type MaxSignatories = MaxSignatories;
    type WeightInfo = weights::pallet_multisig::WeightInfo;
}

impl pallet_sudo::Trait for Runtime {
    type Event = Event;
    type Call = Call;
}

// orml
parameter_types! {
    pub const GetNativeCurrencyId: CurrencyId = rio_protocol::RFUEL;
}

impl orml_currencies::Trait for Runtime {
    type Event = Event;
    type MultiCurrency = RioAssets;
    type NativeCurrency = BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;
    type GetNativeCurrencyId = GetNativeCurrencyId;
    type WeightInfo = ();
}

parameter_types! {
    pub const MinVestedTransfer: Balance = 100 * DOLLARS;
}

pub struct EnsureRootOrTreasury;
impl EnsureOrigin<Origin> for EnsureRootOrTreasury {
    type Success = AccountId;

    fn try_origin(o: Origin) -> Result<Self::Success, Origin> {
        Into::<Result<RawOrigin<AccountId>, Origin>>::into(o).and_then(|o| match o {
            RawOrigin::Root => Ok(TreasuryModuleId::get().into_account()),
            RawOrigin::Signed(caller) => {
                if caller == TreasuryModuleId::get().into_account() {
                    Ok(caller)
                } else {
                    Err(Origin::from(Some(caller)))
                }
            }
            r => Err(Origin::from(r)),
        })
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn successful_origin() -> Origin {
        Origin::from(RawOrigin::Signed(Default::default()))
    }
}

impl orml_vesting::Trait for Runtime {
    type Event = Event;
    type Currency = Balances;
    type MinVestedTransfer = MinVestedTransfer;
    type VestedTransferOrigin = EnsureRootOrTreasury;
    type WeightInfo = ();
}

parameter_types! {
    pub const MinimumCount: u32 = 1;
    pub const ExpiresIn: Moment = 1000 * 60 * 60; // 60 mins
    pub RootOperatorAccountId: AccountId = AccountId::from([0u8; 32]); // TODO use a account id
}

impl orml_oracle::Trait for Runtime {
    type Event = Event;
    type OnNewData = ();
    type CombineData = orml_oracle::DefaultCombineData<Runtime, MinimumCount, ExpiresIn>;
    type Time = Timestamp;
    type OracleKey = CurrencyId;
    type OracleValue = Price;
    type RootOperatorAccountId = RootOperatorAccountId;
    type WeightInfo = ();
}

pub type TimeStampedPrice = orml_oracle::TimestampedValueOf<Runtime>;

// rio
impl rio_assets::Trait for Runtime {
    type Event = Event;
    type Balance = Balance;
    type Amount = Amount;
    type CurrencyId = CurrencyId;
    type OnReceived = (); // module_accounts::Module<Runtime>;
    type WeightInfo = ();
}

impl rio_assets_ext::Trait for Runtime {
    type Event = Event;
}

impl rio_payment_fee::Trait for Runtime {
    type Event = Event;
    type Currency = Balances;
    type MultiCurrency = RioAssets;
    type WeightInfo = ();
}

parameter_types! {
    pub const TransactionByteFee: Balance = 0; // TODO change in future
}
impl rio_payment::Trait for Runtime {
    type Currency = Balances;
    // todo, may deposit tu author?
    type OnTransactionPayment = RioPaymentFee;
    type TransactionByteFee = TransactionByteFee;
    /// weight is weight, weight is not fee
    type WeightToFee = IdentityFee<Balance>;
    type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
    type BuyFeeAsset = RioPaymentFee;
}

impl rio_gateway::Trait for Runtime {
    type Event = Event;
    type Currency = Currencies;
    type WeightInfo = ();
}

type EnsureRootOrHalfGeneralCouncil = EnsureOneOf<
    AccountId,
    EnsureRoot<AccountId>,
    EnsureRoot<AccountId>, // todo use council module to replace it.
>;

impl rio_prices::Trait for Runtime {
    type Event = Event;
    type Source = Oracle;
    type LockOrigin = EnsureRootOrHalfGeneralCouncil;
}

type EnsureRootOrManager =
    EnsureOneOf<AccountId, EnsureRoot<AccountId>, EnsureSignedBy<RioRoot, AccountId>>;
impl rio_root::Trait for Runtime {
    type Event = Event;
    type Currency = RioAssets;
    type RootOrigin = EnsureRootOrManager;
    type WeightInfo = ();
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = rio_primitives::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: frame_system::{Module, Call, Config, Storage, Event<T>},
        RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Module, Call, Storage},
        Scheduler: pallet_scheduler::{Module, Call, Storage, Event<T>},

        Aura: pallet_aura::{Module, Inherent},

        Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
        Indices: pallet_indices::{Module, Call, Storage, Config<T>, Event<T>},
        Balances: pallet_balances::{Module, Storage, Config<T>, Event<T>},

        // Consensus support.
        Authorship: pallet_authorship::{Module, Call, Storage, Inherent},
        Staking: pallet_staking::{Module, Call, Storage, Config<T>, Event<T>, ValidateUnsigned},
        Offences: pallet_offences::{Module, Call, Storage, Event},
        Historical: pallet_session_historical::{Module},
        Session: pallet_session::{Module, Call, Storage, Event, Config<T>},
        FinalityTracker: pallet_finality_tracker::{Module, Call, Storage, Inherent},
        Grandpa: pallet_grandpa::{Module, Call, Storage, Event},
        ImOnline: pallet_im_online::{Module, Call, Storage, Event<T>, ValidateUnsigned, Config<T>},

        // Governance stuff.
        Democracy: pallet_democracy::{Module, Call, Storage, Config, Event<T>},
        Council: pallet_collective::<Instance1>::{Module, Call, Storage, Origin<T>, Event<T>, Config<T>},
        TechnicalCommittee: pallet_collective::<Instance2>::{Module, Call, Storage, Origin<T>, Event<T>, Config<T>},
        Elections: pallet_elections_phragmen::{Module, Call, Storage, Event<T>, Config<T>},
        TechnicalMembership: pallet_membership::<Instance1>::{Module, Call, Storage, Event<T>, Config<T>},
        Treasury: pallet_treasury::{Module, Call, Storage, Config, Event<T>},

        Identity: pallet_identity::{Module, Call, Storage, Event<T>},
        Society: pallet_society::{Module, Call, Storage, Event<T>, Config<T>},
        Recovery: pallet_recovery::{Module, Call, Storage, Event<T>},

        Utility: pallet_utility::{Module, Call, Event},
        Multisig: pallet_multisig::{Module, Call, Storage, Event<T>},
        Sudo: pallet_sudo::{Module, Call, Config<T>, Storage, Event<T>},

        Currencies: orml_currencies::{Module, Call, Event<T>},
        Vesting: orml_vesting::{Module, Storage, Call, Event<T>, Config<T>},
        Oracle: orml_oracle::{Module, Storage, Call, Config<T>, Event<T>},

        RioAssets: rio_assets::{Module, Storage, Config<T>, Event<T>},
        RioAssetsExt: rio_assets_ext::{Module, Storage, Call, Event<T>},
        RioPayment: rio_payment::{Module, Storage},
        RioPaymentFee: rio_payment_fee::{Module, Call, Storage, Config<T>, Event<T>},

        RioGateway: rio_gateway::{Module, Call, Storage, Config<T>, Event<T>},
        RioPrices: rio_prices::{Module, Storage, Call, Event},

        RioRoot: rio_root::{Module, Storage, Call, Config<T>, Event<T>},
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
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    rio_payment::ChargeTransactionPayment<Runtime>,
    BaseFilter,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<Call, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllModules,
>;

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

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(
            block: Block,
            data: sp_inherents::InherentData,
        ) -> sp_inherents::CheckInherentsResult {
            data.check_extrinsics(&block)
        }

        fn random_seed() -> <Block as BlockT>::Hash {
            RandomnessCollectiveFlip::random_seed()
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: TransactionSource,
            tx: <Block as BlockT>::Extrinsic,
        ) -> TransactionValidity {
            Executive::validate_transaction(source, tx)
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
        fn slot_duration() -> u64 {
            Aura::slot_duration()
        }

        fn authorities() -> Vec<AuraId> {
            Aura::authorities()
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            SessionKeys::generate(seed)
        }

        fn decode_session_keys(
            encoded: Vec<u8>,
        ) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
            SessionKeys::decode_into_raw_public_keys(&encoded)
        }
    }

    impl fg_primitives::GrandpaApi<Block> for Runtime {
        fn grandpa_authorities() -> GrandpaAuthorityList {
            Grandpa::grandpa_authorities()
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            equivocation_proof: fg_primitives::EquivocationProof<
                <Block as BlockT>::Hash,
                NumberFor<Block>,
            >,
            key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            let key_owner_proof = key_owner_proof.decode()?;

            Grandpa::submit_unsigned_equivocation_report(
                equivocation_proof,
                key_owner_proof,
            )
        }

        fn generate_key_ownership_proof(
            _set_id: fg_primitives::SetId,
            authority_id: GrandpaId,
        ) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
            Historical::prove((fg_primitives::KEY_TYPE, authority_id))
                .map(|p| p.encode())
                .map(fg_primitives::OpaqueKeyOwnershipProof::new)
        }
    }

    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
        fn account_nonce(account: AccountId) -> Index {
            System::account_nonce(account)
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<
        Block,
        Balance,
    > for Runtime {
        fn query_info(uxt: <Block as BlockT>::Extrinsic, len: u32) -> RuntimeDispatchInfo<Balance> {
            RioPayment::query_info(uxt, len)
        }
    }

    impl orml_oracle_rpc_runtime_api::OracleApi<
        Block,
        DataProviderId,
        CurrencyId,
        TimeStampedPrice,
    > for Runtime {
        fn get_value(_provider_id: DataProviderId, _key: CurrencyId) -> Option<TimeStampedPrice> {
            None
        }

        fn get_all_values(_provider_id: DataProviderId) -> Vec<(CurrencyId, Option<TimeStampedPrice>)> {
            Vec::new()
        }
    }

    impl rio_gateway_rpc_runtime_api::GatewayApi<
        Block,
        CurrencyId,
        AccountId,
        Balance,
    > for Runtime {
        fn withdraw_list() -> BTreeMap<u64, (WithdrawItem<CurrencyId, AccountId, Balance>, Balance)> {
            RioGateway::withdraw_list()
        }

        fn pending_withdraw_list() -> BTreeMap<u64, (WithdrawItem<CurrencyId, AccountId, Balance>, Balance)> {
            RioGateway::pending_withdraw_list()
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    impl frame_benchmarking::Benchmark<Block> for Runtime {
        fn dispatch_benchmark(
            pallet: Vec<u8>,
            benchmark: Vec<u8>,
            lowest_range_values: Vec<u32>,
            highest_range_values: Vec<u32>,
            steps: Vec<u32>,
            repeat: u32,
            extra: bool,
        ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, RuntimeString> {
            use frame_benchmarking::{Benchmarking, BenchmarkBatch, add_benchmark};

            impl frame_system_benchmarking::Trait for Runtime {}

            // let whitelist: Vec<TrackedStorageKey> = vec![
                // // Block Number
                // hex_literal::hex!("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx").to_vec().into(),
                // // Total Issuance
                // hex_literal::hex!("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx").to_vec().into(),
                // // Execution Phase
                // hex_literal::hex!("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx").to_vec().into(),
                // // Event Count
                // hex_literal::hex!("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx").to_vec().into(),
                // // System Events
                // hex_literal::hex!("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx").to_vec().into(),
                // // Treasury Account
                // hex_literal::hex!("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx").to_vec().into(),
            // ];

            let mut batches = Vec::<BenchmarkBatch>::new();
            let params = (
                &pallet,
                &benchmark,
                &lowest_range_values,
                &highest_range_values,
                &steps,
                repeat,
                &Vec::new(),
                extra,
            );
            // Substrate
            add_benchmark!(params, batches, pallet_balances, Balances);

            if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
            Ok(batches)
        }
    }
}
