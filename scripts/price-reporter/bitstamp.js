"use strict";
 
const {TickerStream, OrderBookStream, Bitstamp, CURRENCY} = require("node-bitstamp");
const Decimal = require('decimal.js');
const moment = require('moment');
const { ApiPromise, WsProvider } = require('@polkadot/api');
const testKeyring = require('@polkadot/keyring/testing');
const BN = require('bn.js');
const [Alice, Charlie, Bob] = ["xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx", "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx", "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"]
const snooze = ms => new Promise(resolve => setTimeout(resolve, ms));
const types = require('./types.json')
const config = require('./config')

async function main() {
  const key = "xxxxxxxxxxxxxxxxxxxx";
  const secret = "xxxxxxxxxxxxxxxxxxxx";
  const clientId = "xxxxxx";

  const bitstamp = new Bitstamp({
    key,
    secret,
    clientId,
    timeout: 5000,
    rateLimit: true //turned on by default
  });

  const provider = new WsProvider(config.wsProvider);
  const api = await ApiPromise.create(
    { provider,
      types: types
    })
  const keyring = testKeyring.default();
  let sub_key = keyring.getPair(Bob);

  while(true) {
    let ticker = await bitstamp.ticker(CURRENCY.BTC_USD);
    console.log("pushing", ticker.body.last)
    let price = new BN(new Decimal(ticker.body.last).mul(10000).toString())
    let price_report = api.tx.rioPrice.report(price)
    await api.tx.rioOracleMembers.execute(price_report).signAndSend(sub_key, ({ events = [], status }) => {
      console.log("pushed price", price.toString(), status.toString(), status.toString())
    })
    await snooze(15000)
  }
}

main().catch(console.error).finally(() => process.exit());
