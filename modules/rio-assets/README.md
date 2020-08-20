## Rio-Assets
Rio-assets is an extension for frame/pallet-generic-assets and we did a simple wrap based on that. We added asset symbol and removed the permission for common users to create their toke.

## storage：
Symbols : asset id with symbol

## interface：
create(initial_balance, symbol) : create a new asset, root only (until we have a better acl system)

transfer(asset_id, to, balance) : user transfer

mint(asset_id, to, amount) : root only (until we have a better acl system)

burn(asset_id, from, amount) :  root only (until we have a better acl system)

## event：
follow generic asset module
