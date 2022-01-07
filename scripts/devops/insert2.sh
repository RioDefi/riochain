# https://github.com/paritytech/substrate/blob/master/primitives/core/src/crypto.rs
# grandpa

curl http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -d \
  '{
    "jsonrpc":"2.0",
    "id":1,
    "method":"author_insertKey",
    "params": [
      "gran",
      "",
      ""
    ]
  }'