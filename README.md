# RioChain

Rio Defi Blockchain (https://www.riochain.io/)

Rio DeFi is a Blockchain technology company. Our mission is to accelerate the mass adoption of digital assets by bridging traditional and decentralized finance.

Our vision is a world in which everyone has access to decentralized financial (DeFi) services. To that end, we develop applications that connect people to digital assets, mobile payments, and DeFi services such as savings and lending. Our solutions enable lower transaction fees, faster confirmations, energy efficiency, secure storage, and global reach.


## Notice

This is the beta version of RioChain. And we are focusing on developing RioChain V1 which is going to open source soon.

## Build

Install Rust:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Install required tools:

```bash
./scripts/init.sh
```

Build Wasm and native code:

```bash
cargo build
```

## Run

### Single node development chain

You can start a development chain with:

```bash
cargo run -- --dev
```

Detailed logs may be shown by running the node with the following environment variables set: `RUST_LOG=debug RUST_BACKTRACE=1 cargo run -- --dev`.

### Multi-node local testnet

If you want to see the multi-node consensus algorithm in action locally, then you can create a local testnet with two validator nodes for Alice and Bob, who are the initial authorities of the genesis chain that have been endowed with testnet units.

Optionally, give each node a name and expose them so they are listed on the Polkadot [telemetry site](https://telemetry.polkadot.io/#/Local%20Testnet).

You'll need two terminal windows open.

We'll start Alice's rio chain node first on default TCP port 30333 with her chain database stored locally at `/tmp/alice`. The bootnode ID of her node is `QmRpheLN4JWdAnY7HGJfWFNbfkQCb6tFf4vvA6hgjMZKrR`, which is generated from the `--node-key` value that we specify below:

```bash
cargo run -- \
  --base-path /tmp/alice \
  --chain=local \
  --alice \
  --node-key 0000000000000000000000000000000000000000000000000000000000000001 \
  --telemetry-url ws://stats.riochain.io:1024 \
  --validator
```

In the second terminal, we'll start Bob's rio chain node on a different TCP port of 30334, and with his chain database stored locally at `/tmp/bob`. We'll specify a value for the `--bootnodes` option that will connect his node to Alice's bootnode ID on TCP port 30333:

```bash
cargo run -- \
  --base-path /tmp/bob \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/QmRpheLN4JWdAnY7HGJfWFNbfkQCb6tFf4vvA6hgjMZKrR \
  --chain=local \
  --bob \
  --port 30334 \
  --telemetry-url ws://stats.riochain.io:1024 \
  --validator
```

Additional CLI usage options are available and may be shown by running `cargo run -- --help`.

## types

```json
{
    "FeeExchangeV1": {
        "max_payment": "Compact<Balance>"
    },
    "FeeExchange": {
        "_enum": {
            "V1": "Compact<FeeExchangeV1>"
        }
    },
    "Restriction": {
        "_enum": [
            "Transferable"
        ]
    },
    "DclCoinSymbol": {
        "_enum": [
            "USDT",
            "EMC",
            "ELC",
            "ETG"
        ]
    },
    "VipLevelPrice": {
        "amount": "Balance",
        "asset_id": "AssetId"
    },
    "DclAuth": {
        "_enum": [
            "All",
            "AddCoin",
            "Task",
            "None"
        ]
    },
    "VipLevel": {
        "_enum": [
            "Normal",
            "Level1",
            "Level2",
            "Level3",
            "Level4",
            "Level5",
            "Level6"
        ]
    },
    "TxHash": "H256",
    "Deposit": {
        "account_id": "AccountId",
        "tx_hash": "Option<TxHash>",
        "amount": "Balance"
    },
    "Auth": {
        "_enum": [
            "All",
            "Deposit",
            "Withdraw",
            "Refund",
            "Mark"
        ]
    },
    "BlackOrWhite": {
        "_enum": [
            "Black",
            "White"
        ]
    },
    "ExtrinsicIndex": "u32",
    "LineNumber": "u32",
    "AuctionBalance": "Balance",
    "TotalLoanBalance": "Balance",
    "CollateralBalanceAvailable": "Balance",
    "CollateralBalanceOriginal": "Balance",
    "Price": "u128",
    "PriceReport": {
        "reporter": "AccountId",
        "price": "Price"
    },
    "LTV": "u64",
    "LoanId": "u64",
    "LoanPackageId": "u64",
    "PhaseId": "u32",
    "LoanHealth": {
        "_enum": {
        "Well": null,
        "Warning": "LTV",
        "Liquidating": "LTV",
        "Extended": null,
        "Expired": null
        }
    },
    "LoanPackageStatus": {
        "_enum": [
            "Active",
            "Inactive"
        ]
    },
    "Loan": {
        "id": "LoanId",
        "package_id": "LoanPackageId",
        "who": "AccountId",
        "due": "Moment",
        "due_extend": "Moment",
        "collateral_balance_original": "Balance",
        "collateral_balance_available": "Balance",
        "loan_balance_total": "Balance",
        "status": "LoanHealth"
    },
    "LoanPackage": {
        "id": "LoanPackageId",
        "status": "LoanPackageStatus",
        "terms": "u32",
        "min": "Balance",
        "interest_rate_hourly": "u32",
        "collateral_asset_id": "AssetId",
        "loan_asset_id": "AssetId"
    },
    "SharePackage": {
        "terms_total": "u32",
        "terms_left": "u32",
        "per_term": "Balance"
    },
    "ReleaseTrigger": {
        "_enum": {
            "PhaseChange": null,
            "BlockNumber": "BlockNumber"
        }
    },
    "ShareReleasePack": {
        "asset_id": "AssetId",
        "phase_id": "PhaseId",
        "owner": "AccountId",
        "empty": "bool",
        "major": "SharePackage",
        "minor": "Option<SharePackage>",
        "release_trigger": "ReleaseTrigger"
    },
    "PhaseInfo": {
        "id": "PhaseId",
        "quota": "u128",
        "exchange": "u128",
        "iou_asset_id": "Option<u32>"
    },
    "GameWay": "u32",
    "RoundId": "u64",
    "GameControlItem": {
        "paused": "bool",
        "time_stamp": "Moment",
        "wait_time": "u32",
        "duration": "u32",
        "asset_id": "AssetId",
        "interval": "u32",
        "group": "u32"
    },
    "BetItem": {
        "account_id": "AccountId",
        "amount": "Balance",
        "asset_id": "AssetId",
        "is_root": "bool"
    },
    "BetType": {
        "_enum": [
            "Short",
            "Long",
            "Draw"
        ]
    },
    "BillingType": {
        "_enum": [
            "Win",
            "Lose",
            "Unchanging"
        ]
    },
    "BetPriceLimit": {
        "min_bet": "Balance",
        "max_bet": "Balance"
    },
    "GameChangeType": {
        "_enum": [
            "Add",
            "Update",
            "Remove",
            "Unchanging"
        ]
    },
    "GameWayInfo": {
        "game_way": "GameWay",
        "asset_id": "AssetId",
        "duration": "u32",
        "wait_time": "u32"
    },
    "GameStatusType": {
        "_enum": [
            "Begin",
            "End",
            "Paused",
            "Restart",
            "Unchanging"
        ]
    },
    "RecommendStatusType": {
        "_enum": [
            "Success",
            "Failure",
            "Binded",
            "Unchanging"
        ]
    },
    "ETHAddress": "H160",
    "ETHAuth": {
        "_enum": [
            "All",
            "Create",
            "Delete",
            "Deposit",
            "Withdraw",
            "Refund",
            "Mark",
            "KYC",
            "None"
        ]
    },
    "ERC20Token": {
        "asset_id": "AssetId",
        "contract_addr": "ETHAddress",
        "decimal": "u32",
        "total_supply": "Option<Balance>"
    },
    "ERC20TokenDeposit": {
        "account_id": "AccountId",
        "tx_hash": "Option<TxHash>",
        "amount": "Balance"
    },
    "MerchantReturnExtend": {
        "customer_transaction_id": "u32",
        "merchant_transaction_id": "u32"
    },
    "SendBCoinExtend": {
          "transaction_id": "u32"
    },
    "PayForVipExtend": {
          "transaction_id": "u32"
    },
    "TransferFailure": {
        "asset_id": "AssetId",
        "account_id": "AccountId",
        "amount": "Balance"
    },
    "DclCoinSymbolV2": {
        "_enum": [
            "USDT",
            "EMC",
            "ELC",
            "ETG",
            "RFUEL",
            "RFUELFREEZE"
        ]
    },
    "ExchangeLimitItem": {
        "time_stamp": "Moment",
        "asset_id": "AssetId",
        "valued": "Balance"
    }
}
```
