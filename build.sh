#!/bin/bash

# set -eux

cargo contract build --manifest-path az_groups/Cargo.toml --release
cargo contract build --manifest-path az_smart_contract_hub/Cargo.toml --release
