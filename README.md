# Axelar Cross-chain Gateway Protocol for Soroban

This repo implements Axelar's [cross-chain gateway protocol](https://github.com/axelarnetwork/cgp-spec/tree/main/solidity) in Soroban for use on Stellar. The reference Solidity contracts can be found [here](https://github.com/axelarnetwork/cgp-spec/tree/main/solidity#design).

## Install

Install Soroban CLI

```bash
cargo install --locked soroban-cli --features opt
```

## Build

```bash
cargo wasm

cargo test
```

## Deploy

```bash
soroban contract build

./optimize.sh

soroban contract deploy --wasm target/wasm32-unknown-unknown/release/[contract].optimized.wasm --source wallet --network testnet
```
