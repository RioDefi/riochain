use std::collections::HashMap;

use aura_primitives::sr25519::AuthorityId as AuraId;
use grandpa_primitives::AuthorityId as GrandpaId;
use hex_literal::hex;
use primitives::crypto::UncheckedInto;
use primitives::{sr25519, Pair, Public};
#[allow(unused_imports)]
use runtime::constants::{currency::*, time::*};
use runtime::types::*;
use runtime::{
    self, AuraConfig, GenesisConfig, GrandpaConfig, IndicesConfig, RioAssetsConfig,
    RioBridgeConfig, RioFeeConfig, SudoConfig, SystemConfig, WASM_BINARY,
};
use sc_service;
use serde_json as json;
use sp_runtime::traits::{IdentifyAccount, Verify};
use tel::TelemetryEndpoints;

// Note this is the URL for the telemetry server
//const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::ChainSpec<GenesisConfig>;

/// The chain specification option. This is expected to come in from the CLI and
/// is little more than one of a number of alternatives which can easily be converted
/// from a string (`--chain=...`) into a `ChainSpec`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Alternative {
    /// Whatever the current runtime is, with just Alice as an auth.
    Development,
    /// Whatever the current runtime is, with simple Alice/Bob auths.
    LocalTestnet,
    /// The Rio network.
    Testnet,
    /// Titan network
    Beta,
}

lazy_static::lazy_static! {
    static ref CHAIN_TYPE: HashMap<Alternative, (&'static str, &'static str)> = {
        let mut m = HashMap::new();
        // value is (name, id)
        m.insert(Alternative::Development, ("Development", "dev"));
        m.insert(Alternative::LocalTestnet, ("Local Testnet", "local_testnet"));
        m.insert(Alternative::Testnet, ("Rio Defi Testnet", "moniker"));
        m.insert(Alternative::Beta, ("Rio Beta", "beta"));
        m
    };
}

pub fn get_alternative_from_id(id: &str) -> Result<Alternative, String> {
    for (k, v) in CHAIN_TYPE.iter() {
        if v.1 == id {
            return Ok(*k);
        }
    }
    Err(format!("no support id in current `Alternative`:{:}", id))
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate stash, controller and session key from seed
pub fn get_authority_keys_from_seed(seed: &str) -> (AuraId, GrandpaId) {
    (
        get_from_seed::<AuraId>(seed),
        get_from_seed::<GrandpaId>(seed),
    )
}

const DEFAULT_PROPERTIES_MAINNET: &str = r#"
{
"tokenSymbol": "RFUEL",
"tokenDecimals": 8,
"ss58Format": 42
}
"#;

const DEFAULT_PROPERTIES_TESTNET: &str = r#"
{
"tokenSymbol": "RFUEL",
"tokenDecimals": 8,
"ss58Format": 42
}
"#;

impl Alternative {
    /// Get an actual chain config from one of the alternatives.
    pub(crate) fn load(self) -> Result<ChainSpec, String> {
        let (name, id) = CHAIN_TYPE
            .get(&self)
            .ok_or("not support for this Alternative")?;
        Ok(match self {
            Alternative::Development => ChainSpec::from_genesis(
                name,
                id,
                || {
                    testnet_genesis(
                        vec![get_authority_keys_from_seed("Alice")],
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        vec![
                            get_account_id_from_seed::<sr25519::Public>("Alice"),
                            get_account_id_from_seed::<sr25519::Public>("Bob"),
                            get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                            get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                        ],
                        true,
                    )
                },
                vec![],
                None,
                None,
                Some(json::from_str(DEFAULT_PROPERTIES_TESTNET).unwrap()),
                None,
            ),
            Alternative::LocalTestnet => ChainSpec::from_genesis(
                name,
                id,
                || {
                    testnet_genesis(
                        vec![
                            get_authority_keys_from_seed("Alice"),
                            get_authority_keys_from_seed("Bob"),
                        ],
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        vec![
                            get_account_id_from_seed::<sr25519::Public>("Alice"),
                            get_account_id_from_seed::<sr25519::Public>("Bob"),
                            get_account_id_from_seed::<sr25519::Public>("Charlie"),
                            get_account_id_from_seed::<sr25519::Public>("Dave"),
                            get_account_id_from_seed::<sr25519::Public>("Eve"),
                            get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                            get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                            get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                            get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                            get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                            get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                            get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                        ],
                        true,
                    )
                },
                vec![],
                None,
                None,
                Some(json::from_str(DEFAULT_PROPERTIES_TESTNET).unwrap()),
                None,
            ),
            Alternative::Testnet => ChainSpec::from_genesis(
                name,
                id,
                || {
                    testnet_genesis(
                        vec![
                            get_authority_keys_from_seed("Alice"),
                            get_authority_keys_from_seed("Bob"),
                        ],
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        vec![
                            get_account_id_from_seed::<sr25519::Public>("Alice"),
                            get_account_id_from_seed::<sr25519::Public>("Bob"),
                            get_account_id_from_seed::<sr25519::Public>("Charlie"),
                            get_account_id_from_seed::<sr25519::Public>("Dave"),
                            get_account_id_from_seed::<sr25519::Public>("Eve"),
                            get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                            get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                            get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                            get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                            get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                            get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                            get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                        ],
                        true,
                    )
                },
                vec![
                    "/ip4/47.52.149.251/tcp/30101/p2p/QmRpheLN4JWdAnY7HGJfWFNbfkQCb6tFf4vvA6hgjMZKrR".to_string()
                ],
                Some(
                    TelemetryEndpoints::new(
                        vec![
                            ("https://stats.staging.riodefi.com".to_string(), 0)
                        ]
                    )
                ),
                Some("alpha"),
                Some(json::from_str(DEFAULT_PROPERTIES_TESTNET).unwrap()),
                None,
            ),
            Alternative::Beta => ChainSpec::from_genesis(
                name,
                id,
                || {
                    beta_genesis(
                        // initial_authorities
                        vec![(
                            // 5GRNaQjxizf2NtrJ7CA2EpqtHUQ6RK3iTnn1LGRoWCsEAryn
                            hex!["3c748c52c521a8ccb85a0c5ee8f9479d45cb0348b5022ce97f35363689f3a84e"].unchecked_into(),
                            // 5G6FhGVzzik1hepRSzFKJPPUszEhV9Q2Gaqx3bugWTGbqwjC
                            hex!["b3cd4432d77141225c98502e2a2a461486e3b01c25a61ee54e6761c7c31570a8"].unchecked_into(),
                        ), (
                            // 5HQixUgqjLbD6AeB9Lfqa5V3ye3QjcSNam8KoWsrrqFf9FgH
                            hex!["328f2204c644361866c92996fc5acffc67e86e6da1af358b38a3de3241bfa276"].unchecked_into(),
                            // 5DqVgX24AquCfUbL2SWXYSJH5PixAL8ngAmoGkQeVKMMUwLZ
                            hex!["a47355db185f4de899055d90f0b6889fb080dd4e5c310a62390d760bc3602c9a"].unchecked_into(),
                        ), (
                            // 5HQixUgqjLbD6AeB9Lfqa5V3ye3QjcSNam8KoWsrrqFf9FgH
                            hex!["78f7655aa22a8f6b54fb7fea4e62f11f365dd67703bf0b5b3ed8338a10373b19"].unchecked_into(),
                            // 5DqVgX24AquCfUbL2SWXYSJH5PixAL8ngAmoGkQeVKMMUwLZ
                            hex!["60e004988514b9075192585ea8acb70652df7928d4f04a1175067b294510d8eb"].unchecked_into(),
                        )],
                        hex!["20cd1afa4f95b59b7f61a97360e8bc74a26a6fc13712e6f2eef3a1e020bbcd68"].into(), // 5CoiKRg4hQopwaHxvjdk7C2Gq1pbdvJhsLiYrVHAcsHNkV8m
                        vec![
                            // 5Ca8HLb1EkJgsDMSpifh33rprTCn9m1DpRkjXNTe7czU1UA5
                            hex!["167056fa07ae7bc1d36da5520dd8c4c06cb5a5db557986f0c6ae2af0030d4c44"].into(),
                            // 5G9xD8Bn32KALTZWafYP97e9TwnnzqacwUgSNdLB8mvbvRt1
                            hex!["b4f179ee3e5e2498eb6d59d1d899bcb0157a917f9cfd8523101e4f78c8a52050"].into()
                        ],
                        true,
                    )
                },
                vec![
                    "/ip4/47.244.206.150/tcp/30333/p2p/QmNh3WxHeGATKcEjRSv97HTT6UpmjFdzYSFpn2UeR2DFok".to_string()
                ],
                Some(
                    TelemetryEndpoints::new(
                        vec![
                            ("https://stats.riochain.io".to_string(), 0)
                        ]
                    )
                ),
                Some("beta"),
                Some(json::from_str(DEFAULT_PROPERTIES_MAINNET).unwrap()),
                None,
            ),
        })
    }

    pub(crate) fn from(s: &str) -> Option<Self> {
        match s {
            "dev" => Some(Alternative::Development),
            "local" => Some(Alternative::LocalTestnet),
            "test" => Some(Alternative::Testnet),
            "beta" => Some(Alternative::Beta),
            _ => None,
        }
    }
}

fn testnet_genesis(
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    enable_println: bool,
) -> GenesisConfig {
    let num_endowed_accounts = endowed_accounts.len();
    GenesisConfig {
        system: Some(SystemConfig {
            code: WASM_BINARY.to_vec(),
            changes_trie_config: Default::default(),
        }),
        indices: Some(IndicesConfig {
            ids: endowed_accounts.clone(),
        }),
        sudo: Some(SudoConfig {
            key: root_key.clone(),
        }),
        aura: Some(AuraConfig {
            authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
        }),
        grandpa: Some(GrandpaConfig {
            authorities: initial_authorities
                .iter()
                .map(|x| (x.1.clone(), 1))
                .collect(),
        }),
        rio_assets: Some(RioAssetsConfig {
            root: root_key.clone(),
            symbols: vec![
                // asset id defined in protocol
                (
                    AssetId::from(runtime::RFUEL),
                    b"RFUEL".to_vec(),
                    vec![],
                    vec![],
                ),
                (
                    AssetId::from(runtime::LOCKED_RFUEL),
                    b"Locked RFUEL".to_vec(),
                    vec![runtime::permissions::Restriction::Transferable],
                    vec![],
                ),
                (
                    AssetId::from(runtime::SBTC),
                    b"SBTC".to_vec(),
                    vec![],
                    vec![],
                ),
                (
                    AssetId::from(runtime::SUSDT),
                    b"S-USDT".to_vec(),
                    vec![],
                    vec![],
                ),
                (AssetId::from(101 as u32), b"RBTC".to_vec(), vec![], vec![]),
                (AssetId::from(102 as u32), b"RSC1".to_vec(), vec![], vec![]),
                (AssetId::from(103 as u32), b"RSC2".to_vec(), vec![], vec![]),
                (AssetId::from(104 as u32), b"RSC3".to_vec(), vec![], vec![]),
                (AssetId::from(105 as u32), b"RSC4".to_vec(), vec![], vec![]),
                (AssetId::from(106 as u32), b"RSC5".to_vec(), vec![], vec![]),
            ],
        }),
        // rio_loan: Some(RioLoanConfig {
        //     current_btc_price: 8000_0000,
        //     collateral_asset_id: 10,
        //     loan_asset_id: 1000,
        //     global_ltv_limit: 6500,
        //     global_liquidation_threshold: 9000,
        //     global_warning_threshold: 8000,
        //     next_loan_id: 1,
        //     next_loan_package_id: 1,
        //     pawn_shop: get_account_id_from_seed::<sr25519::Public>("999999999999"),
        //     profit_pool: get_account_id_from_seed::<sr25519::Public>("88888888"),
        //     penalty_rate: 200,
        //     liquidation_account: get_account_id_from_seed::<sr25519::Public>("Bob"),
        //     minimum_collateral: 2000_0000,
        //     liquidation_penalty: 1300,
        // }),
        // rio_saving: Some(RioSavingConfig {
        //     current_phase_id: 1,
        //     collection_asset_id: 10,
        //     share_asset_id: 2,
        //     phase_infos: vec![
        //         (100_00000000, 10000, 3),
        //         (400_00000000, 8000, 4),
        //         (1000_00000000, 5000, 5),
        //         (5000_00000000, 2000, 6),
        //         (10000_00000000, 1000, 7),
        //     ],
        //     collection_account_id: get_account_id_from_seed::<sr25519::Public>("Alice"),
        //     team_account_id: get_account_id_from_seed::<sr25519::Public>("Team"),
        //     profit_pool: get_account_id_from_seed::<sr25519::Public>("88888888"),
        //     profit_asset_id: 8,
        //     reserved_mint_wallet: get_account_id_from_seed::<sr25519::Public>("reserved wallet"),
        //     reserved_mint_asset_id: 8,
        // }),
        rio_bridge: Some(RioBridgeConfig {
            asset_id: AssetId::from(runtime::SBTC),
            threshold: 30_0000_0000,
            admins: vec![(
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                runtime::rio_bridge::Auth::All,
            )],
            pending_withdraw_vault: get_account_id_from_seed::<sr25519::Public>("withdraw vault"),
        }),
        rio_fee: Some(RioFeeConfig {
            account_id: get_account_id_from_seed::<sr25519::Public>("Eve"),
        }),
    }
}

fn beta_genesis(
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    enable_println: bool,
) -> GenesisConfig {
    let num_endowed_accounts = endowed_accounts.len();
    GenesisConfig {
        system: Some(SystemConfig {
            code: WASM_BINARY.to_vec(),
            changes_trie_config: Default::default(),
        }),
        indices: Some(IndicesConfig {
            ids: endowed_accounts.clone(),
        }),
        sudo: Some(SudoConfig {
            key: root_key.clone(),
        }),
        aura: Some(AuraConfig {
            authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
        }),
        grandpa: Some(GrandpaConfig {
            authorities: initial_authorities
                .iter()
                .map(|x| (x.1.clone(), 1))
                .collect(),
        }),
        rio_assets: Some(RioAssetsConfig {
            root: root_key.clone(),
            symbols: vec![
                // asset id defined in protocol
                (
                    AssetId::from(runtime::RFUEL),
                    b"RFUEL".to_vec(),
                    vec![],
                    vec![],
                ),
                (
                    AssetId::from(runtime::LOCKED_RFUEL),
                    b"Locked RFUEL".to_vec(),
                    vec![runtime::permissions::Restriction::Transferable],
                    vec![],
                ),
                (
                    AssetId::from(runtime::SBTC),
                    b"SBTC".to_vec(),
                    vec![],
                    vec![],
                ),
                (
                    AssetId::from(runtime::SUSDT),
                    b"S-USDT".to_vec(),
                    vec![],
                    vec![],
                ),
                (AssetId::from(101 as u32), b"RBTC".to_vec(), vec![], vec![]),
                (AssetId::from(102 as u32), b"RSC1".to_vec(), vec![], vec![]),
                (AssetId::from(103 as u32), b"RSC2".to_vec(), vec![], vec![]),
                (AssetId::from(104 as u32), b"RSC3".to_vec(), vec![], vec![]),
                (AssetId::from(105 as u32), b"RSC4".to_vec(), vec![], vec![]),
                (AssetId::from(106 as u32), b"RSC5".to_vec(), vec![], vec![]),
            ],
        }),
        // rio_loan: Some(RioLoanConfig {
        //     current_btc_price: 8000_0000,
        //     collateral_asset_id: 1,
        //     loan_asset_id: 8,
        //     global_ltv_limit: 6500,
        //     global_liquidation_threshold: 9000,
        //     global_warning_threshold: 8000,
        //     next_loan_id: 1,
        //     next_loan_package_id: 1,
        //     pawn_shop: get_account_id_from_seed::<sr25519::Public>(
        //         "0x183606b482851e67aa356e598b2efb699391e26ec3c087f648e0fc1a22a83f98",
        //     ),
        //     profit_pool: get_account_id_from_seed::<sr25519::Public>(
        //         "0x183606b482851e67aa356e598b2efb699391e26ec3c087f648e0fc1a22a83f98",
        //     ),
        //     penalty_rate: 200,
        //     liquidation_account: get_account_id_from_seed::<sr25519::Public>(
        //         "0x183606b482851e67aa356e598b2efb699391e26ec3c087f648e0fc1a22a83f98",
        //     ),
        //     minimum_collateral: 2000_0000,
        //     liquidation_penalty: 1300,
        // }),
        // rio_saving: Some(RioSavingConfig {
        //     current_phase_id: 1,
        //     collection_asset_id: 1,
        //     share_asset_id: 2,
        //     phase_infos: vec![
        //         (100_00000000, 10000, 3),
        //         (400_00000000, 8000, 4),
        //         (1000_00000000, 5000, 5),
        //         (5000_00000000, 2000, 6),
        //         (10000_00000000, 1000, 7),
        //     ],
        //     collection_account_id: get_account_id_from_seed::<sr25519::Public>(
        //         "0x183606b482851e67aa356e598b2efb699391e26ec3c087f648e0fc1a22a83f98",
        //     ),
        //     team_account_id: get_account_id_from_seed::<sr25519::Public>(
        //         "0x183606b482851e67aa356e598b2efb699391e26ec3c087f648e0fc1a22a83f98",
        //     ),
        //     profit_pool: get_account_id_from_seed::<sr25519::Public>(
        //         "0x183606b482851e67aa356e598b2efb699391e26ec3c087f648e0fc1a22a83f98",
        //     ),
        //     profit_asset_id: 8,
        //     reserved_mint_wallet: get_account_id_from_seed::<sr25519::Public>(
        //         "0x183606b482851e67aa356e598b2efb699391e26ec3c087f648e0fc1a22a83f98",
        //     ),
        //     reserved_mint_asset_id: 8,
        // }),
        rio_bridge: Some(RioBridgeConfig {
            asset_id: AssetId::from(runtime::SBTC),
            threshold: 30_0000_0000,
            admins: vec![(
                // 5FJ4MeWhQtvBZxjFkgFm9ojbw2jhqjmjoYo9Jr3xSU7j2Uyk
                hex!["8ee3e606982e20e495b622d091a99d4cdbc669afd8c08efb1acfb375b2e9f61a"].into(),
                runtime::rio_bridge::Auth::All,
            )],
            pending_withdraw_vault: hex![
                "5e1379e6dbd99ec5914c127f7eada6a6a6c773ed47cadf4f023577b0deb3ab09"
            ]
            .into(), // 5EC49JXFCGud9PyhhKxviMdRNHXyn6ZYaAJjsVkLFXYG1WPz
        }),
        rio_fee: Some(RioFeeConfig {
            account_id: hex!["16bb3aef8cfefcf218ffa6d8722baa4ae808acb0de521e71ccbdeb3bf9cb2d1e"]
                .into(), //5CaWXft7prB292TAjbbFcTbQdrGR8KmwAD8iGPUMHHJPFVdy
        }),
    }
}
