#!/bin/sh

for file in contracts/*; do name=$( echo "${file#contracts/}" | tr '-' '_' ); soroban contract optimize --wasm "target/wasm32-unknown-unknown/release/${name}.wasm" ; done
