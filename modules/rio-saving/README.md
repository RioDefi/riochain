包含了所有与saving功能相关的代码逻辑。

## 模块包含的storage字段，每个字段都有相应的getter方法：

CollectionAssetId：用来做saving的assetid，例如SBTC的id

CollectionAccountId：接收用户saving的account，这个可以公示给大众监督

ShareAssetId：参与saving，置换出来用以计算分红权益的asset，例如RBTC

ShareAssetCollected：如果用户直接转账share asset到collection account，这个会有追踪记录，这个在用户退出saving取回自己btc时，作为凭证

IOUAssetPhaseId：assetid 到phaseid的映射

CurrentPhaseId：当前的phase

QuotaUsed：当前phase已经用掉的配额

NumOfPhasesLeft：剩余phase数量

NumOfPhases：总共的phase数量

PhaseInfos：定义的所有phase的信息，phaseid从1开始

ShareUnreleasedList：记录用户的未释放的share asset，会在每个phase切换的时候释放固定的比例

AccountShares：追踪所有参与saving的用户持有的share asset的比例

SharesCirculation：追踪总的share asset流通量

Paused：暂停功能的开关

LastBonusTime：上次分利息的时间

ProfitAssetId：作为派息的asset

ProfitPool：暂时汇总当前利息的账户

TeamAccountId：运营团队接收利息分红的账户

ReservedMintWallet：saving时兑换出来的RIO放在这个账户地址

ReservedMintAssetId：saving是兑换出来的RIO的id，这个是为了灵活性


## 模块接口的定义：
pause : 暂停，需要root权限

resume : 恢复，pause的逆操作，需要root权限

set_share_asset_id : 需要root权限

set_iou_asset_id_for_phase : 需要root权限

set_collection_account : 需要root权限

set_collection_asset_id : 需要root权限

set_profit_asset_id : 需要root权限

set_profit_pool : 需要root权限

set_team_account_id : 需要root权限

staking(asset_id, amount) : 参与saving的assetid和相应的数量，目前都是SBTC

redeem(iou_asset_id, amount) : 要赎回的saving，每个saving存入的时候都会1:1的置换出一个属于当前phase的所谓的RS Contract的asset，这个asset代表了1个sbtc参与saving时，所置换出来的RIO的兑换比例

force_release_bonus : 需要root权限，手动派发分红


## 模块的事件：

PhaseChanged(from_phase_id, to_phase_id) : phase变化的时候触发

Paused(line_number, block_Number, extrinsic_index) : 暂停的时候触发

Bonus() : 派息的时候

StakingCreated(account_id, RBTC balance, iou_asset_id, SBTC balance) : 用户存入

StakingRedeem(account_id, RBTC balance, iou_asset_id, SBTC balance) ： 用户取回
