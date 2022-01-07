/* eslint-disable @typescript-eslint/require-await */
/* eslint-disable @typescript-eslint/unbound-method */
/* eslint-disable @typescript-eslint/no-var-requires */
// Import the API
const {
  ApiPromise,
  WsProvider,
  Keyring
} = require('@polkadot/api');

// Our address for Alice on the dev chain
const ALICE = 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx';
const HORIZON = "----------------------------------------------";

const rio_types = {
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
  "ChainAddress": "Text",
  "Memo": "Text",
  "WithdrawInfo": {
    "asset_id": "AssetId",
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
  },
  "String": "Text",
  "WithdrawItemForRpc": {
    "currency_id": "CurrencyId",
    "applicant": "AccountId",
    "value": "String",
    "addr": "String",
    "memo": "String",
    "state": "WithdrawState"
  },
  "AccountDepositAddr": {
    "deposit_addr_info": "DepositAddrInfo",
    "index": "Option<u64>"
  },
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
};

async function main() {
  // Create our API with a default connection to the local node
  const api = await ApiPromise.create({
    provider: new WsProvider("wss://xxxx-xxxx.xx.xxxxxxx.xxxxxxxx.xx"),
    types: rio_types
  });

  const keyring = new Keyring({ type: "sr25519" });
  const accounts = [];
  const total = 10000;
  const accounts_per_loop = 200;
  const alice = keyring.addFromUri('//Alice');
  const AMOUNT = 1000000000000;

  async function initial_balances(start, end, init_amount) {
    let ps = [];
    let batch_address = [];
    let account = await api.query.system.account(alice.address);
    let nonce = account.nonce.toNumber();

    for (let i = start; i < end; i++) {
      accounts.push(keyring.addFromUri("//fake_user/" + i));
      let recv_addr = accounts[i].address;
      await api.tx.currencies
        .transfer(recv_addr, 0, init_amount)
        .signAndSend(alice, { nonce }).then(async _ => {
          nonce += 1;
          const account = await api.query.rioAssets.accounts(recv_addr, 0);
          let prev_balance = Number(JSON.parse(String(account)).free)
          let y = new Promise(async (resolve, reject) => {
            let h = setTimeout(function() { reject("timeout"); }, 20000);
            const unsub = await api.query.rioAssets.accounts(recv_addr, 0, (balance) => {
              const freeBalance = Number(JSON.parse(String(balance)).free)
              if (prev_balance != freeBalance) {
                clearTimeout(h);
                unsub();
                resolve(recv_addr);
              }
            });
          });
          ps.push(y);
          batch_address.push(recv_addr);
        });
    }
    console.log(batch_address);
    await Promise.all(ps);
  }

  async function mint_RBTC(start, end, amount) {
    let ps = [];
    let account = await api.query.system.account(alice.address);
    let nonce = account.nonce.toNumber();
    const RBTC_id = 100;

    for (let i = start; i < end; i++) {
      let recv_addr = accounts[i].address;
      await api.tx.currencies
        .updateBalance(recv_addr, RBTC_id, amount)
        .signAndSend(alice, { nonce }).then(async _ => {
          nonce += 1;
          const account = await api.query.rioAssets.accounts(recv_addr, 0);
          let prev_balance = Number(JSON.parse(String(account)).free);
          let y = new Promise(async (resolve, _) => {
            const unsub = await api.query.rioAssets.accounts(recv_addr, RBTC_id, (balance) => {
              const freeBalance = Number(JSON.parse(String(balance)).free)
              if (prev_balance != freeBalance) {
                unsub();
                resolve(recv_addr);
              }
            });
          });
          ps.push(y);
        });
    }
    await Promise.all(ps);
  }

  async function apply_for_loan(start, end, amount, package_id) {
    let ps = [];

    for (let i = start; i < end; i++) {
      let loanee = accounts[i];
      await api.tx.rioLoan
        .apply(amount, 0, package_id)
        .signAndSend(loanee).then(async _ => {
          let account = await api.query.system.account(loanee.address);
          let prev_nonce = account.nonce.toNumber();
          let y = new Promise(async (resolve, _) => {
            const unsub = await api.query.system.account(loanee.address, (account) => {
              let nonce = account.nonce;
              if (nonce.toNumber() > prev_nonce.toNumber()) {
                unsub();
                resolve(loanee.address);
              }
            });
          });
          ps.push(y);
        });
    }
    await Promise.all(ps);
  }

  async function do_something(start, end) {
    let ps = [];
    for (let i = start; i < end; i++) {
      ps.push(api.tx.root.doSomething().signAndSend(accounts[i], { nonce: 0 }));
    }
    await Promise.all(ps);
  }

  let then;
  let elapse;

  console.log(HORIZON, `Create ${total} testing accounts`, HORIZON);
  for (let loop = 0; loop < total / accounts_per_loop + 1; loop++) {
    let start = loop * accounts_per_loop;
    let end = (start + accounts_per_loop) <= total ? (start + accounts_per_loop) : total;
    if (start >= end) {
      break;
    }

    console.log(HORIZON);
    console.log(`creating ${accounts_per_loop} accounts with balance ${AMOUNT / 100000000}`);
    then = new Date();
    await initial_balances(start, end, AMOUNT);
    elapse = (new Date()) - then;
    console.log(`Done, ${accounts_per_loop} transactions, ${elapse} milliseconds`);
    console.log(HORIZON);

  }

  for (let loop = 0; loop < total / accounts_per_loop + 1; loop++) {
    let start = loop * accounts_per_loop;
    let end = (start + accounts_per_loop) <= total ? (start + accounts_per_loop) : total;
    if (start >= end) {
      break;
    }
    console.log(HORIZON);
    console.log(`do something`);
    await do_something(start, end);
    console.log(`Done`);
    console.log(HORIZON);
  }
}

main().catch(console.error).finally(() => process.exit());
