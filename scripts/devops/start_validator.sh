n=
port=
ws_port=
rpc_port=
./bin/riochain --chain mainnet --name riochain-n$n --port $port --ws-port $ws_port --rpc-port $rpc_port --base-path /home/node/n$n/data/ --rpc-external --ws-external --rpc-cors=all --pruning=archive --telemetry-url 'ws://172.31.200.11:8000/submit 1' --ws-max-connections 1024 --pool-limit 10000 --execution=NativeElseWasm