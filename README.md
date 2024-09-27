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
```

## Test

```bash
cargo test
```

## Coverage

```bash
cargo install cargo-llvm-cov
cargo llvm-cov
```

## Deploy

- Check configuration for CLI and Identity before deployment: https://developers.stellar.org/docs/build/smart-contracts/getting-started/setup

#### Build, Optimize and Deploy contract:

```bash
soroban contract build

./optimize.sh

soroban contract deploy --wasm target/wasm32-unknown-unknown/release/[contract].optimized.wasm --source wallet --network testnet
```
