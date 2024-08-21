#!/usr/bin/env bash

set -e
set -o pipefail

# projectPath=/c/Projects/eris/liquid-staking-contracts
projectPath=$(dirname `pwd`) 
folderName=$(basename $(dirname `pwd`)) 

mkdir -p "../../$folderName-cache"
mkdir -p "../../$folderName-cache/target"
mkdir -p "../../$folderName-cache/registry"

docker run --rm -v "/$projectPath":/code \
  --mount type=bind,source=/$projectPath-cache/target,target=/target \
  --mount type=bind,source=/$projectPath-cache/registry,target=/usr/local/cargo/registry \
  cosmwasm/optimizer:0.16.0