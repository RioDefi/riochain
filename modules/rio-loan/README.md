## 模块定义的storage：

PawnShop : 用户抵押btc存放的账户

ProfitPool : 和RioSaving一样的收集分红的账户

CollateralAssetId : 抵押的asset id，现在是SBTC

LoanAssetId : 抵押置换出来的asset，现在是RIO

GlobalLTVLimit : 创建一个loan package的时候的最大LTV
        
GlobalLiquidationThreshold : 触发清算的LTV阈值
        
GlobalWarningThreshold : 触发告警的LTV阈值
        
NextLoanPackageId : 下一个产生的loan package id
        
ActiveLoanPackages : 当前有效的loan package，这里存的都是loan package id
        
LoanPackages : 所有的loan package
        
NextLoanId : 下一个产生的loan id
        
Loans : 系统中还没有被偿还或这清算完成的loan
        
LoansByAccount : 每个用户所拥有的loan
        
CurrentBTCPrice : 平台当前的BTC价格
        
TotalLoan : 平台当前总的loan金额
        
TotalCollateral : 平台在pawnshop收到的总的抵押资产
        
TotalProfit : 平台总的利息
        
PenaltyRate : 当loan过期发生自动续借的时候，要扣除一定的罚没金，这个是罚没金比例
        
LiquidationAccount : 操作清算的账户
        
LiquidatingLoans : 所有处于被清算状态的loan id
        
LoanCap : 平台全局的loan金额的硬顶，可以不设，就表示没有硬顶
        
Paused : 暂停开关
        
MinimumCollateral : loan的最小抵押金额
        
LiquidationPenalty : 被清算的loan还要被扣一个清算罚金，这个是罚金的比例

## 模块的接口：

pause() : 暂停，需要root权限

resume() : 恢复，pause的逆操作，需要root权限

set_collateral_asset_id(asset_id) : 需要root权限

set_global_ltv_limit(limit) : 需要root权限

set_loan_asset_id(asset_id) : 需要root权限

set_global_liquidation_threshold(thresold) : 需要root权限

set_global_warning_threshold(threshold) : 需要root权限

set_loan_cap(balance) : 需要root权限

set_liquidation_account(account_id) : 需要root权限

set_penalty_rate(rate) : 需要root权限

create_package(terms, interest_rate, min_rio) : 需要root权限, 参数分别是借贷的天数，小时利率和最小借出的金额

disable_package(package_id) : 需要root权限

set_price(price) : 需要root权限, 手动设置btc价格，一般情况价格会自动更新，这个是一个后备

repay(loan_id) : 要偿还的借贷的id，用户负责保证自己的账户里有足够的asset去偿还

apply(collateral_amount, loan_amount, package_id) : 借贷，用户要指定借贷的金额，抵押的金额，以及使用哪一个loan package

mark_liquidated(loan_id, auction_balance) : 标记清算完成，只有清算账户可以操作，提供偿还的loanid和清算拍卖所得的balance，补足了系统借出的RIO之后，按比例扣掉罚金，剩余的返还给用户

add_collateral(loan_id, amount) : 补充抵押，减小LTV

draw(loan_id, amount) : 从一个LTV不足limit的loan中继续贷出RIO

## 模块的事件：

PackageCreated(loan package id) : 创建package的时候触发

PackageDisabled(loan package id) : disable 一个package的时候触发

LoanCreated(loan) : 创建一个loan的时候，包含了loan的所有信息

LoanRepaid(loanid) : 偿还loan的时候触发

LoanDrawn(loanid) : 从loan中再借的时候触发

Expired(loanid, accountid) : loan错过了延长期依然没有偿还，过期的时候触发

Extended(loanid, accountid) : loan 进入延长期的时候触发

Warning(loanid, LTV) : 告警的时候触发

Liquidating(loanid, accountid, available_collateral_balance, total_loan_balance) : loan
开始被清算的时候触发

Liquidated(loanid, original_collateral_balance, available_collateral_balance, auction_balance, total_loan_balance) : 清算完成触发

AddCollateral(loanid, balance) : 补充抵押触发

