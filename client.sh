#!/bin/bash

# Spin up a temporary container that runs in the same network as our Kind Node.
# and curl the NodePort.
for i in {1..50}; do
  docker run --rm --network kind curlimages/curl -s "http://172.21.0.2:30007/test?param=$i"
  echo
done