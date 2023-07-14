#!/usr/bin/env bash

set -e
set -o pipefail

# projectPath=/c/Projects/eris/liquid-staking-contracts
projectPath=$(dirname `pwd`) 
folderName=$(basename $(dirname `pwd`)) 

echo "Applying $1"

find $projectPath -type f -name 'Cargo.toml' -exec echo {} +
find $projectPath -type f -name 'Cargo.toml' -exec sed -i "s/\"X-.*-X\"/\"X-$1-X\"/g" {} +
find $projectPath -type f -name 'Cargo.toml' -exec sed -i "s/\"eris-staking-hub-tokenfactory.*\"/\"eris-staking-hub-tokenfactory-$1\"/g" {} +
find $projectPath -type f -name 'Cargo.toml' -exec sed -i "s/\"eris-arb-vault.*\"/\"eris-arb-vault-$1\"/g" {} +