## Rio Gateway

`Rio Gateway` is the Generic Asset Gateway built as a RioChain Pallet. This module is to provide the basic and underlying support for Cross-Chain Asset Transfer.

The Cross-chain transfer will be implemented with a federated model. We have decideded to not integrate a light node in the chain to support more blockchain assets. This requires some trust in the federated nodes but have the benifit for less onchain cost and more generic cross-chain asset support.

## Cross-Chain Deposit Process

Notice: the admin account here can be a group of people or the governance mechanism.

1. The account with admin permission uses `set_xpub_of_asset_id` to add xpub key for an asset
2. The common user can use `apply_deposit_index` apply for an index in the system
3. The front end will show the deposit address by combining the Asset Type, Asset Path, User Deposit Index.
4. (Optional) Confirm the deposit address by using the api provided by RioChain
5. The user can deposit their asset to the address
6. Waiting for onchain confirmation
7. Receive the Cross Chain R-Assets on RioChain

## Cross-Chain Withdraw Process

1. The user can use `request_withdraw` api to initial a withdraw request
2. The admin account will review the withdrawal request
3. If the request is approved by using `approve_withdraw`, then it starts operating the withdraw process
4. If the request is rejected by using `reject_withdraw`, then it will reject the user's request and explain the reason.
5. If the withdraw is finished, the admin account will use `finish_withdraw` to mark it as finished and also put the transaction hash on chain
6. (Optional)If there is a rebroadcast occured due to the failed withdrawal transaction, it can use `rebroadcast` to emit an event to record the correct transaction hash on chain.

## Storage

- SupportedAssets
- Admins
- DepositXpubOfAssetId
- NextDepositIndex
- DepositHistory
- ActiveWithdrawStates: Active withdrawals included `Approved` and `Pending` Status.
- ...

## Interface

- apply_deposit_address: Apply deposit address according to the asset, xpubkey and index.
- deposit
- request_withdraw
- withdraw_finish
- approve_withdraw