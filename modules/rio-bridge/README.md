## Rio Bridge Module

version: 1.0 beta

This module is to implement the cross-chain deposit and withdraw for RioChain. This is the first version to make the  business logic work. The next version will be more decentralized.

Rio Bridge will implemented with a federated model, and we have decideded to not integrate a light node in the chain to support the generic cross-chain asset. This requires some trust in the federated nodes but have the benifit for less onchain cost and more generic cross-chain support.

## storage：
AssetId : assetid of sbtc

Paused : Overall swith
        
List : Known List of KYC (whitelist users)
        
Threshold : kyc threshold
        
Admins : admin account of kyc management
        
PendingDepositList : 触发了KYC还没有被认证的存款记录
        
DepositHistory : deposit history
        
PendingWithdraws : pending withdraws
        
PendingWithdrawVault : sbtc vault for withdraw

## interface ：

pause() : pause, root only

resume() : resume，root only

deposit(account_id, amount, tx_hash) : 存入BTC，要提供用户在RIO的用户accountid，金额和BTC的交易hash，只有bridge的admin可以调用

refund(account, amount) : 提现失败的时候返还用户SBTC, 只有bridge的admin可以调用

withdraw_finish(account, amount) : 用户提现成功，只有bridge的admin可以调用

withdraw(amount) : 用户提交提现申请，相应数量的SBTC会自动从用户账户转入vault

mark_black(account) : KYC标记account为黑名单，pending的deposit不会生成SBTC给用户

mark_white(account) : KYC标记account为白名单，pending的deposit会生成SBTC给用户

delete_admin(account) : 删除某一个admin账户，需要root权限

update_admin(account, auth) : 添加/修改admin账户和对应的auth，需要root权限

## event：(rename are required in the next version to follow a event name standard)

AccountMarked(accountid, black_or_white) : triggered when KYC user marked 

Deposit(accountid, balance, txhash) : triggered when KYC deposit

Pending(accountid, balance, txhash)

PendingWithdraw(accountid, balance) : triggered when a user submmited a pending withdraw request

Refund(accountid, balance) : trigged when a sbtc withdrawal request failed

Withdraw(accountid, balance) : triggered when user withdraw
