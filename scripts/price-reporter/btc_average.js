const ccxt = require('ccxt');
const BN = require('bn.js');
const Decimal = require('decimal.js');

// const moment = require('moment');
const { ApiPromise, WsProvider } = require('@polkadot/api');
const { Keyring } = require('@polkadot/keyring');
const types = require('./types.json')
const config = require('./config')

let price = {};
let fetchingPrice = false;
let exchanges = {
    binance: '',
    okex: '',
    bitfinex: '',
    bitstamp: '',
    coinmarketcap: ''
}
let requestMethod = '';
let api = ''

if (!config.adminKey.PHRASE) {
    throw new Error("Invalid Oracle Key");
}


const provider = new WsProvider(config.wsProvider);
let oracleKey = '';

/**
 * Only required on local development
 */
if (config.proxy) {
    const HttpsProxyAgent = require('https-proxy-agent');
    const fetch = require('node-fetch');
    const proxy = config.proxyAddress;
    const agent = new HttpsProxyAgent(proxy);
    requestMethod = function (url, options) {
        return fetch(url, Object.assign({}, options, { agent: agent }))
    }
}

async function loadApi() {
    api = await ApiPromise.create(
        {
            provider,
            types: types
        })
    const keyring = new Keyring({ type: 'sr25519' });
    oracleKey = keyring.addFromUri(config.adminKey.PHRASE);
}

async function loadExchanges() {
    exchanges = {
        binance: '',
        okex: '',
        bitfinex: '',
        bitstamp: '',
        coinmarketcap: ''
    }
    console.log('ready to init exchanges', Object.keys(exchanges))
    const exchangeConfig = {
        timeout: 30000
    }
    if (config.proxy) {
        exchangeConfig.fetchImplementation = requestMethod
    }

    for (let item in exchanges) {
        exchanges[item] = new ccxt[item](exchangeConfig)
        try {
            await exchanges[item].loadMarkets();
            console.log('inited', item);
        } catch (e) {
            console.log('Failed to load market of', item, e.message)
            // loadExchanges()
        }
    }
}

async function getPrices() {
    let averagePrice = 0, sumPrice = 0, priceLength = 0;

    if (!fetchingPrice) {
        console.log('ready to fetch prices');
        fetchingPrice = true;
        price = {};
        for (let item in exchanges) {
            const currentUnit = (item == 'bitstamp' || item == 'coinmarketcap') ? 'BTC/USD' : 'BTC/USDT'
            try {
                let currentTicker = await exchanges[item].fetchTicker(currentUnit);
                console.log(Date(), item, 'price:', currentTicker.last)
                if (currentTiccker.last < config.maxPrice && currentTicker.last > config.minPrice) {
                    price[item] = currentTicker.last;
                }
            } catch (e) {
                console.log('Failed to fetch ticker of', item, e.message)
                // getPrices()
            }
        }
        priceLength = Object.keys(price).length
        for (let item in price) {
            sumPrice = new Decimal(sumPrice).plus(price[item])
        }
        console.log(Date(), 'average price is', new Decimal(sumPrice).div(priceLength).mul(10000).round().toString())
        averagePrice = new Decimal(sumPrice).div(priceLength).mul(10000).round().toString()
        toChain(averagePrice)
    }
}

async function toChain(price) {
    console.log("pusing price--", price.toString())
    let price_report = api.tx.rioPrice.report(price)
    api.tx.rioOracleMembers.execute(price_report).signAndSend(oracleKey, ({ events = [], status }) => {
        console.log("pushed price", price.toString(), status.toString(), status.toString())
    })
    fetchingPrice = false
}

async function getPriceInterval() {
    setInterval(await getPrices, 30000)
}

async function main() {
    await loadApi()
    await loadExchanges()
    await getPriceInterval()
}

main().catch(() => process.exit()).finally(console.error);
