#!/bin/bash
set -e

./target/release/rust-raptor &
SERVER_PID=$!
trap 'kill $SERVER_PID 2>/dev/null || true' EXIT

echo "Waiting 5 seconds for server startup"
sleep 5
echo "Starting benchmark"

for c in 1 2 4 8 16 32; do
    ./.tools/ghz \
        --insecure \
        --proto ./proto/raptor.proto \
        --concurrency "$c" \
        --total 1000 \
        --call 'routing.RoutingService/GetRoute' \
        --data '{"startStopId":"{{randomInt 1 30001}}","destinationStopId":"{{randomInt 1 30001}}","rounds":10}' \
        127.0.0.1:50052
done


