#!/usr/bin/bash

cargo build --release
sudo setcap cap_net_admin=eip target/release/handy_tcp
./target/release/handy_tcp &
pid=$!
ext=$?
if  [[ $ext -ne 0 ]]; then
    exit $ext
fi
sudo ip addr add 10.128.0.1/16 dev tun0
sudo ip link set up dev tun0
trap "kill "$pid"" SIGINT
wait $pid