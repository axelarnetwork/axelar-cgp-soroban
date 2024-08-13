#!/bin/sh

for file in contracts/*; do
    name=$( echo "${file#contracts/}" | tr '-' '_' )
    wasm_file="target/wasm32-unknown-unknown/release/${name}.wasm"
    if [ -f "$wasm_file" ]; then
        soroban contract optimize --wasm "$wasm_file"    
    fi
done
