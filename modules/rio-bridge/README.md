## 简介
该模块主要实现跨链资产的充值与提现，目前还需要与中心化的后台管理钱包配合使用。当中心化的钱包收到对应的充值提现请求后，经过验证将由指定权限的用户发送相关的deposit/withdrwal请求。

对于一次性超过30 BTC的请求，需要列入白名单中才可以正常充值。列入白名单的方式为调用mark_white的接口。如果一个地址提交了一个超过30 BTC的充值请求，但还未通过白名单，则会将其充值标记为pending状态。在将其列入白名单之后，将会自动完成之前的充值请求。

## 模块定义storage：
AssetId : SBTC的assetid

Paused : 全局暂停开关
        
List : 已知的KYC（白名单）列表，标记了用户白名单
        
Threshold : 触发KYC的阈值
        
Admins : KYC的管理账户，可以赋予不同的管理账号不同的权限
        
PendingDepositList : 触发了KYC还没有被认证的存款记录
        
DepositHistory : 存款历史
        
PendingWithdraws : 还未实施的提现请求
        
PendingWithdrawVault : 暂时保管待提现的SBTC的账户

## 模块的接口 ：

pause() : 暂停，需要root权限

resume() : 恢复，pause的逆操作，需要root权限

deposit(account_id, amount, tx_hash) : 存入BTC，要提供用户在RIO的用户accountid，金额和BTC的交易hash，只有bridge的admin可以调用

refund(account, amount) : 提现失败的时候返还用户SBTC, 只有bridge的admin可以调用

withdraw_finish(account, amount) : 用户提现成功，只有bridge的admin可以调用

withdraw(amount) : 用户提交提现申请，相应数量的SBTC会自动从用户账户转入vault

mark_black(account) : KYC标记account为黑名单，pending的deposit不会生成SBTC给用户

mark_white(account) : KYC标记account为白名单，pending的deposit会生成SBTC给用户

delete_admin(account) : 删除某一个admin账户，需要root权限

update_admin(account, auth) : 添加/修改admin账户和对应的auth，需要root权限

## 模块的事件：

AccountMarked(accountid, black_or_white) : KYC标记账户的时候触发

Deposit(accountid, balance, txhash) : 用户存款的时候触发

Pending(accountid, balance, txhash) : 触发KYC的时候触发

PendingWithdraw(accountid, balance) : 等待提现的时候触发

Refund(accountid, balance) : 提现失败返还SBTC触发

Withdraw(accountid, balance) : 请求提现的时候触发
