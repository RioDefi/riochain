# price-reporter

put the btc price onchain

## Configuration
You can enable proxy, set lowest and highest BTC price accepted and WsProvider in `config.js` file.

## Types
You can edit `types.json` to set types.

## Notice
Each script shold end up with the following code
```js
main().catch(() => process.exit()).finally(console.error);
```