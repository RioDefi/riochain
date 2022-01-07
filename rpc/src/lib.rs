// This file is part of Substrate.

// Copyright (C) 2019-2020 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! A collection of node-specific RPC methods.
//!
//! Since `substrate` core functionality makes no assumptions
//! about the modules used inside the runtime, so do
//! RPC methods defined in `sc-rpc` crate.
//! It means that `client/rpc` can't have any methods that
//! need some strong assumptions about the particular runtime.
//!
//! The RPCs available in this crate however can make some assumptions
//! about how the runtime is constructed and what FRAME pallets
//! are part of it. Therefore all node-runtime-specific RPCs can
//! be placed here or imported from corresponding FRAME RPC definitions.

#![warn(missing_docs)]

#[macro_use]
mod utils;

mod apis;
mod errors;
mod impls;
mod types;

use std::sync::Arc;

use sc_client_api::{backend::Backend, CallExecutor};
use sc_finality_grandpa::{
    FinalityProofProvider, GrandpaJustificationStream, SharedAuthoritySet, SharedVoterState,
};
use sc_finality_grandpa_rpc::GrandpaRpcHandler;
use sc_rpc::SubscriptionTaskExecutor;
pub use sc_rpc_api::DenyUnsafe;
use sc_service::client::Client;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_transaction_pool::TransactionPool;

use rio_primitives::{AccountId, Balance, Block, BlockNumber, CurrencyId, Hash, Index};

use crate::apis::RioApi;
use crate::impls::RioRpc;

/// Light client extra dependencies.
pub struct LightDeps<C, F, P> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// Remote access to the blockchain (async).
    pub remote_blockchain: Arc<dyn sc_client_api::light::RemoteBlockchain<Block>>,
    /// Fetcher instance.
    pub fetcher: Arc<F>,
}

/// Extra dependencies for GRANDPA
pub struct GrandpaDeps<B> {
    /// Voting round info.
    pub shared_voter_state: SharedVoterState,
    /// Authority set info.
    pub shared_authority_set: SharedAuthoritySet<Hash, BlockNumber>,
    /// Receives notifications about justification events from Grandpa.
    pub justification_stream: GrandpaJustificationStream<Block>,
    /// Executor to drive the subscription manager in the Grandpa RPC handler.
    pub subscription_executor: SubscriptionTaskExecutor,
    /// Finality proof provider.
    pub finality_provider: Arc<FinalityProofProvider<B, Block>>,
}

/// Full client dependencies.
pub struct FullDeps<BE, E, RA, P, B> {
    /// The client instance to use.
    pub client: Arc<Client<BE, E, Block, RA>>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// Whether to deny unsafe calls
    pub deny_unsafe: DenyUnsafe,
    /// GRANDPA specific dependencies.
    pub grandpa: GrandpaDeps<B>,
}

/// A IO handler that uses all Full RPC extensions.
pub type IoHandler = jsonrpc_core::IoHandler<sc_rpc::Metadata>;

/// Instantiate all Full RPC extensions.
pub fn create_full<BE, E, RA, P, B>(
    deps: FullDeps<BE, E, RA, P, B>,
) -> jsonrpc_core::IoHandler<sc_rpc_api::Metadata>
where
    BE: Backend<Block> + 'static,
    BE::State: sp_state_machine::backend::Backend<sp_runtime::traits::BlakeTwo256>,
    E: CallExecutor<Block> + Clone + Send + Sync,
    RA: Send + Sync + 'static,

    Client<BE, E, Block, RA>: ProvideRuntimeApi<Block>,
    Client<BE, E, Block, RA>:
        HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
    Client<BE, E, Block, RA>: Send + Sync + 'static,
    <Client<BE, E, Block, RA> as ProvideRuntimeApi<Block>>::Api:
        substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
    <Client<BE, E, Block, RA> as ProvideRuntimeApi<Block>>::Api:
        pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    <Client<BE, E, Block, RA> as ProvideRuntimeApi<Block>>::Api:
        rio_gateway_rpc_runtime_api::GatewayApi<Block, CurrencyId, AccountId, Balance>,
    <Client<BE, E, Block, RA> as ProvideRuntimeApi<Block>>::Api: BlockBuilder<Block>,
    P: TransactionPool + 'static,
    B: sc_client_api::Backend<Block> + Send + Sync + 'static,
    B::State: sc_client_api::backend::StateBackend<sp_runtime::traits::HashFor<Block>>,
{
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApi};
    use substrate_frame_rpc_system::{FullSystem, SystemApi};

    use rio_gateway_rpc::{Gateway, GatewayApi};

    let mut io = jsonrpc_core::IoHandler::default();
    let FullDeps {
        client,
        pool,
        deny_unsafe,
        grandpa,
    } = deps;
    let GrandpaDeps {
        shared_voter_state,
        shared_authority_set,
        justification_stream,
        subscription_executor,
        finality_provider,
    } = grandpa;
    io.extend_with(sc_finality_grandpa_rpc::GrandpaApi::to_delegate(
        GrandpaRpcHandler::new(
            shared_authority_set,
            shared_voter_state,
            justification_stream,
            subscription_executor,
            finality_provider,
        ),
    ));

    io.extend_with(SystemApi::to_delegate(FullSystem::new(
        client.clone(),
        pool,
        deny_unsafe,
    )));
    io.extend_with(TransactionPaymentApi::to_delegate(TransactionPayment::new(
        client.clone(),
    )));

    io.extend_with(GatewayApi::to_delegate(Gateway::new(client.clone())));
    io.extend_with(RioApi::to_delegate(RioRpc::new(client)));

    io
}

/// Instantiate all Light RPC extensions.
pub fn create_light<C, P, M, F>(deps: LightDeps<C, F, P>) -> jsonrpc_core::IoHandler<M>
where
    C: sp_blockchain::HeaderBackend<Block>,
    C: Send + Sync + 'static,
    F: sc_client_api::light::Fetcher<Block> + 'static,
    P: TransactionPool + 'static,
    M: jsonrpc_core::Metadata + Default,
{
    use substrate_frame_rpc_system::{LightSystem, SystemApi};

    let LightDeps {
        client,
        pool,
        remote_blockchain,
        fetcher,
    } = deps;
    let mut io = jsonrpc_core::IoHandler::default();
    io.extend_with(SystemApi::<Hash, AccountId, Index>::to_delegate(
        LightSystem::new(client, remote_blockchain, fetcher, pool),
    ));

    io
}
