#!/bin/sh


./target/release/node-template \
  --base-path /tmp/node01 \
  --chain=./node/res/homework_spec_raw.json \
  --node-key-file=/tmp/node_key_01.txt \
  --port 30333 \
  --ws-port 9944 \
  --rpc-port 9933 \
  --validator \
  --rpc-methods Unsafe \
  --name node01 &

./target/release/node-template \
  --base-path /tmp/node02 \
  --chain=./node/res/homework_spec_raw.json \
  --node-key-file=/tmp/node_key_02.txt \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWPq12sbGQPFWHdgNVsQJpRboyNwFdAahkCK5iZNgiYr6A \
  --port 30334 \
  --ws-port 9945 \
  --rpc-port 9934 \
  --validator \
  --rpc-methods Unsafe \
  --name node02 &
