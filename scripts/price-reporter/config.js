module.exports = {
    //Enable local proxy
    proxy: true,
    //Local proxy address
    proxyAddress: 'http://127.0.0.1:1087',
    //Lowest BTC/USD price to be accepted
    minPrice: 2000,
    //Highest BTC/USD price to be accepted
    maxPrice: 20000,
    //WsProvider address
    wsProvider: 'wss://node.riochain.io/',

    adminKey: {
        PHRASE: '//Bob',
    }
}
