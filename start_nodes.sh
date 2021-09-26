#!/bin/sh


./target/release/node-template \
  --base-path /tmp/node01 \
  --chain=./node/res/homework_spec_raw.json \
  --node-key=a58b84ac9dcb481d656bf8b5171396b05485f8dd39be2ea652a6a8f11755a724 \
  --port 30333 \
  --ws-port 9944 \
  --rpc-port 9933 \
  --validator \
  --rpc-methods Unsafe \
  --name node01 &

./target/release/node-template \
  --base-path /tmp/node02 \
  --chain=./node/res/homework_spec_raw.json \
  --node-key=be789a22b3d6616d6f836cc3bc55e82fe0fd04351b6997d94845a65ebc7e0129 \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWPq12sbGQPFWHdgNVsQJpRboyNwFdAahkCK5iZNgiYr6A \
  --port 30334 \
  --ws-port 9945 \
  --rpc-port 9934 \
  --validator \
  --rpc-methods Unsafe \
  --name node02 &
