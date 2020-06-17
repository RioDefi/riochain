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
const rio_types = require('./types.json');


async function main() {
  // Create our API with a default connection to the local node
  const api = await ApiPromise.create({
    provider: new WsProvider("ws://127.0.0.1:9944"),
    types: rio_types
  });

  const keyring = new Keyring({ type: "sr25519" });
  const accounts = [];
  const total = 10000;
  const accounts_per_loop = 400;
  const alice = keyring.addFromUri('//Alice');
  const AMOUNT = 100000000;

  async function initial_balances(start, end, init_amount) {
    let ps = [];
    let batch_address = [];
    let nonce = (await api.query.system.accountNonce(alice.address)).toNumber();

    for (let i = start; i < end; i++) {
      accounts.push(keyring.addFromUri("//fake_user/a" + i));
      let recv_addr = accounts[i].address;
      await api.tx.rioAssets
        .transfer(0, recv_addr, init_amount)
        .signAndSend(alice, { nonce }).then(async _ => {
          nonce += 1;
          let prev_balance = await api.query.rioAssets.freeBalance(0, recv_addr);
          let y = new Promise(async (resolve, reject) => {
            let h = setTimeout(function() { reject("timeout"); }, 20000);
            const unsub = await api.query.rioAssets.freeBalance(0, recv_addr, (balance) => {
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
          let prev_balance = await api.query.rioAssets.freeBalance(sbtc_id, recv_addr);
          let y = new Promise(async (resolve, _) => {
            const unsub = await api.query.rioAssets.freeBalance(sbtc_id, recv_addr, (balance) => {
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

  async function do_something(start, end) {
    let ps = [];
    for (let i = start; i < end; i++) {
      ps.push(api.tx.rioAssets.transfer(0, ALICE, 10000000).signAndSend(accounts[i], { nonce: 0 }));
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
