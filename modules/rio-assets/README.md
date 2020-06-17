Rio-assets是直接扩展的frame/pallet-generic-assets。在其之上包装了很简单的一层封装，增加了asset symbol的功能。同时限制了除了transfer之外的操作均只能由root代理，限制了用户自己创建asset的功能。

## 模块定义的storage：
Symbols : asset id对应的symbol

## 模块的接口：
create(initial_balance, symbol) : 创建一个新的asset， 需要root权限， root账户拥有该asset所有的权限

transfer(asset_id, to, balance) : 用户转账，每笔收取0.1RFUEL的固定手续费

mint(asset_id, to, amount) : 需要root权限

burn(asset_id, from, amount) : 需要root权限

## 模块的事件：
Rio-assets没有自定义的事件
