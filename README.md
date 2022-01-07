# RioChain

Rio Defi Blockchain (https://www.riochain.io/)

Rio DeFi is a Blockchain technology company. Our mission is to accelerate the mass adoption of digital assets by bridging traditional and decentralized finance.

Our vision is a world in which everyone has access to decentralized financial (DeFi) services. To that end, we develop applications that connect people to digital assets, mobile payments, and DeFi services such as savings and lending. Our solutions enable lower transaction fees, faster confirmations, energy efficiency, secure storage, and global reach.


## Notice

The chainspec accounts have been removed from source code for security purposes. 

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

The chainspec accounts have been removed from source code for security purposes. After filling the chainspec account, you can start a development chain with:

```bash
cargo run -- --dev
```

Detailed logs may be shown by running the node with the following environment variables set: `RUST_LOG=debug RUST_BACKTRACE=1 cargo run -- --dev`.

### Multi-node local testnet

If you want to see the multi-node consensus algorithm in action locally, then you can create a local testnet with two validator nodes for Alice and Bob, who are the initial authorities of the genesis chain that have been endowed with testnet units.

Optionally, give each node a name and expose them so they are listed on the Polkadot [telemetry site](https://telemetry.polkadot.io/#/Local%20Testnet).

You'll need two terminal windows open.

The chainspec accounts have been removed from source code for security purposes. After filling the chainspec account, please use subkey to generate the bootnode ID. Please alse use `--node-key` to specify the corresponding keys:

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

## Build

Pull submodules

```bash
git submodule update --init --recursive
```

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

## Basic Asset Id Convention

It's better to register new assets here: `rio/protocol/src/lib.rs` and follow the following standard:

* 0 - 100:  System Token
* 101 - 10000:  System Cross-Chain Tokens
* 10001+:  User Tokens

## types

```json
{
    "LookupSource": "IndicesLookupSource",
    "Address": "LookupSource",
    "Amount": "i128",
    "AmountOf": "Amount",
    "CurrencyId": "u32",
    "CurrencyIdOf": "CurrencyId",
    "Price": "FixedU128",
    "OracleKey": "CurrencyId",
    "Chain": {
      "_enum": [
        "Rio",
        "Bitcoin",
        "Litecoin",
        "Ethereum",
        "EOS",
        "Polkadot",
        "Kusama",
        "ChainX"
      ]
    },
    "AssetInfo": {
        "chain": "Chain",
        "symbol": "Text",
        "name": "Text",
        "decimals": "u8",
        "desc": "Text"
    },
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
            "Transferable",
            "Depositable",  
            "Withdrawable",
            "Slashable",
            "Reservable",
            "Unreservable"
        ]
    },
    "TxHash": "H256",
    "Deposit": {
        "account_id": "AccountId",
        "amount": "Balance"
    },
    "Auths": {
        "mask": "u8"
    },
    "Auth": {
        "_enum": [
            "Register",
            "Deposit",
            "Withdraw",
            "Sudo"
        ]
    },
    "WithdrawState": {
        "_enum": {
            "Pending":  null,
            "Cancelled": null,
            "Rejected": null,
            "Approved": null,
            "Success": "TxHash",
            "ReBroadcasted": "TxHash"
        }
    },
    "ChainAddress": "Bytes",
    "Memo": "Text",
    "WithdrawInfo": {
        "currency_id": "CurrencyId",
        "who": "AccountId",
        "value": "Balance",
        "addr": "ChainAddress",
        "memo": "Text"
    },
    "WithdrawItem": {
        "currency_id": "CurrencyId",
        "applicant": "AccountId",
        "value": "Balance",
        "addr": "ChainAddress",
        "memo": "Text",
        "state": "WithdrawState"
    },
    "DepositAddrInfo": {
        "_enum": {
            "Bip32": "Bip32",
            "Create2": "Create2"
        }
    },
    "Bip32": {
        "x_pub": "Text",
        "path": "Text"
    },
    "Create2": {
        "creator_address": "Vec<u8>",
        "implementation_address": "Vec<u8>",
        "vault_address": "Vec<u8>"
    } 
}
```

rpc:

rpc_types:

```json
{
    "String": "Text",
    "WithdrawItemForRpc": {
        "currencyId": "CurrencyId",
        "applicant": "AccountId",
        "value": "String",
        "addr": "String",
        "memo": "String",
        "state": "WithdrawState",
        "fee": "String"
    },
    "AccountDepositAddr": {
        "deposit_addr_info": "DepositAddrInfo",
        "index": "Option<u64>"
    }
}
```

rpc_interface:

```json
{
    "riogateway": {
        "withdrawList": {
            "description": "get current withdraw list(include pending and approve)",
            "params": [
                {
                    "name": "at",
                    "isOptional": true
                }
            ],
            "type": "BTreeMap<u64, WithdrawItemForRpc>"
        },
        "pendingWithdrawList": {
            "description": "get current pending withdraw list",
            "params": [
                {
                    "name": "at",
                    "isOptional": true
                }
            ],
            "type": "BTreeMap<u64, WithdrawItemForRpc>"
        },
        "depositAddress": {
            "description": "get deposit address info for an account and asset, if this account have not apply, in bip32 path would return `nil`",
            "params": [
                {
                    "name": "at",
                    "who": "AccountId",
                    "currency_id": "CurrencyId",
                    "isOptional": true
                }
            ],
            "type": "DepositAddrForRpc"
        }
    }
}
```
