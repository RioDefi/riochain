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
const ALICE = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
const HORIZON = "----------------------------------------------";

const rio_types = {
  "PhaseId": "u32",
  "LoanPackageId": "u64",
  "LoanId": "u64",
  "LTV": "u32",
  "PhaseInfo": {
    "id": "PhaseId",
    "quota": "u128",
    "exchange": "u128",
    "iou_asset_id": "Option<u32>"
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
  "ReleaseTrigger": {
    "_enum": {
      "PhaseChange": null,
      "BlockNumber": "u64"
    }
  },
  "SharePackage": {
    "terms_left": "u32",
    "terms_total": "u32",
    "per_term": "Balance"
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
  "Loan": {
    "id": "LoanId",
    "package_id": "LoanPackageId",
    "who": "AccountId",
    "due": "Moment",
    "due_extend": "Moment",
    "collateral_balance_original": "Balance",
    "collateral_balance_available": "Balance",
    "loan_balance_total": "Balance"
  },
  "LoanPackageStatus": {
    "_enum": [
      "Active",
      "Inactive"
    ]
  },
  "LoanHealth": {
    "_enum": {
      "Well": null,
      "Warning": "u32",
      "Liquidating": "u32",
      "Extended": null,
      "Expired": null
    }
  },
  "PriceReport": {
    "reporter": "AccountId",
    "price": "Price"
  },
  "Price": "u128",
  "Ledger": {
    "active": "Balance",
    "unbonds": "Vec<Unbind>"
  },
  "Unbind": {
    "amount": "Balance",
    "era": "BlockNumber"
  },
  "CollateralBalanceOriginal": "Balance",
  "CollateralBalanceAvailable": "Balance",
  "TotalLoanBalance": "Balance",
  "AuctionBalance": "Balance",
  "LineNumber": "u32",
  "ExtrinsicIndex": "u32"
};

async function main() {
  // Create our API with a default connection to the local node
  const api = await ApiPromise.create({
    // provider: new WsProvider("wss://node.staging.riodefi.com"),
    types: rio_types
  });

  const keyring = new Keyring({ type: "sr25519" });
  const accounts = [];
  const total = 1000;
  const accounts_per_loop = 200;
  const alice = keyring.addFromUri('//Alice');
  const AMOUNT = 100000000;

  async function initial_balances(start, end, init_amount) {
    let ps = [];
    let batch_address = [];
    let nonce = (await api.query.system.accountNonce(alice.address)).toNumber();

    for (let i = start; i < end; i++) {
      accounts.push(keyring.addFromUri("//fake_user/" + i));
      let recv_addr = accounts[i].address;
      await api.tx.balances
        .transfer(recv_addr, init_amount)
        .signAndSend(alice, { nonce }).then(async _ => {
          nonce += 1;
          let prev_balance = await api.query.balances.freeBalance(recv_addr);
          let y = new Promise(async (resolve, reject) => {
            let h = setTimeout(function() {reject("timeout");}, 20000);
            const unsub = await api.query.balances.freeBalance(recv_addr, (balance) => {
              if (!prev_balance.sub(balance).isZero()) {
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

  async function mint_sbtc(start, end, amount) {
    let ps = [];
    let nonce = (await api.query.system.accountNonce(alice.address)).toNumber();
    const sbtc_id = 1;

    for (let i = start; i < end; i++) {
      let recv_addr = accounts[i].address;
      await api.tx.rioAssets
        .mint(sbtc_id, recv_addr, amount)
        .signAndSend(alice, { nonce }).then(async _ => {
          nonce += 1;
          let prev_balance = await api.query.rioAssetsQuery.freeBalance(sbtc_id, recv_addr);
          let y = new Promise(async (resolve, _) => {
            const unsub = await api.query.rioAssetsQuery.freeBalance(sbtc_id, recv_addr, (balance) => {
              if (!prev_balance.sub(balance).isZero()) {
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
          let prev_nonce = await api.query.system.accountNonce(loanee.address);
          let y = new Promise(async (resolve, _) => {
            const unsub = await api.query.system.accountNonce(loanee.address, (nonce) => {
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

  async function create_loan_packages() {
    let prev_nonce = await api.query.system.accountNonce(alice.address);
    await api.tx.rioLoan.createPackage(10, 100, 1).signAndSend(alice);
    await new Promise(async (resolve, reject) => {
      let unsub = await api.query.system.accountNonce(alice.address, (nonce) => {
        if (nonce.toNumber() > prev_nonce.toNumber()) {
          unsub();
          resolve();
        }
      });
    });
  }

  console.log(HORIZON, "create loan packages", HORIZON);
  await create_loan_packages();

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

    console.log(HORIZON);
    console.log(`mint ${AMOUNT / 1000000} SBTC for each account`);
    then = new Date();
    await mint_sbtc(start, end, AMOUNT * 100);
    elapse = (new Date()) - then;
    console.log(`Done, ${accounts_per_loop} transactions, ${elapse} milliseconds`);
    console.log(HORIZON);

    console.log(HORIZON);
    console.log("applying for loan");
    then = new Date();
    await apply_for_loan(start, end, 100000000, 1);
    elapse = (new Date()) - then;
    console.log(`Done, ${accounts_per_loop} transactions, ${elapse} milliseconds`);
    console.log(HORIZON);
  }

  // for (let loop = 0; loop < total / accounts_per_loop + 1; loop++) {
  //   let start = loop * accounts_per_loop;
  //   let end = (start + accounts_per_loop) <= total ? (start + accounts_per_loop) : total;
  //   if (start >= end) {
  //     break;
  //   }


  // }
}

main().catch(console.error).finally(() => process.exit());
