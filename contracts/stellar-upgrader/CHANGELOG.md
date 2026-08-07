# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.5](https://github.com/axelarnetwork/axelar-amplifier-stellar/compare/stellar-upgrader-v1.1.4...stellar-upgrader-v1.1.5)

### ⚙️ Miscellaneous Tasks

- Updated the following local packages: stellar-axelar-std, stellar-axelar-std - ([0000000](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/0000000))

## [1.1.4](https://github.com/axelarnetwork/axelar-amplifier-stellar/compare/stellar-upgrader-v1.1.3...stellar-upgrader-v1.1.4)

### ⚙️ Miscellaneous Tasks

- Updated the following local packages: stellar-axelar-std, stellar-axelar-std - ([0000000](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/0000000))

## [1.1.3](https://github.com/axelarnetwork/axelar-amplifier-stellar/compare/stellar-upgrader-v1.1.2...stellar-upgrader-v1.1.3)

### ⚙️ Miscellaneous Tasks

- Updated the following local packages: stellar-axelar-std, stellar-axelar-std - ([0000000](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/0000000))

## [1.1.2](https://github.com/axelarnetwork/axelar-amplifier-stellar/compare/stellar-upgrader-v1.1.1...stellar-upgrader-v1.1.2)

### 📚 Documentation

- Add READMEs to all crates ([#340](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/340)) - ([606affd](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/606affdb67172cb6d6812f8d08f43b8f4ae6df95))

### Contributors

* @nbayindirli

## [1.1.1](https://github.com/axelarnetwork/axelar-amplifier-stellar/compare/stellar-upgrader-v1.1.0...stellar-upgrader-v1.1.1)

### ⚙️ Miscellaneous Tasks

- Bump up patch versions ([#331](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/331)) - ([75a25c2](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/75a25c23e2103baa0c9a723380c716ebad6e8798))

### Contributors

* @ahramy

## [1.1.0](https://github.com/axelarnetwork/axelar-amplifier-stellar/compare/stellar-upgrader-v1.0.0...stellar-upgrader-v1.1.0)

### ⛰️ Features

- Check authorization at the root in the upgrader contract ([#294](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/294)) - ([ce4edb7](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/ce4edb7c7e63176e801d2eddb808a68e087cb4fd))
- Block regular contract endpoints during migration ([#279](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/279)) - ([7444057](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/7444057f85f73ff8a65eedbd5ae0aad77c2e7ad4))
- Add only_owner and only_operator macros ([#240](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/240)) - ([bf26705](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/bf267059dd047475c7efb7e9bee47b40eaec4bbd))

### 🐛 Bug Fixes

- *(axelar-std-derive)* Enforce contractstorage enums are private ([#267](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/267)) - ([b9c5688](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/b9c568830c5207f68104bf9c9156e0c851722b98))
- *(upgrader)* Cast upgrader contract invoke_contract to val ([#305](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/305)) - ([114340f](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/114340fb8623e45622d7df5b2f16a75bfef9b38c))
- Update compile gh action to build and test all contracts separately ([#322](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/322)) - ([f6d3623](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/f6d3623d79655a9f48dbb1db77f48aa08545b651))

### 🚜 Refactor

- *(axelar-operators)* Migrate Operators to Operator ([#252](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/252)) - ([dc76cb3](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/dc76cb3b6b11f13e906c54d1179c2fa157a4449d))
- *(interchain-token-service)* Migrate token-manager, interchain-token & destruct flow keys ([#280](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/280)) - ([8990431](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/89904314cf900e161241c516b98e923cb1ee605e))
- *(upgrader)* Remove soroban-sdk ([#295](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/295)) - ([473fb39](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/473fb390e142e2452343f91baa19b93924640389))
- *(upgrader)* Use contractstorage ([#247](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/247)) - ([71a8cae](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/71a8cae51792ec812f2cca073a44836b825a97d0))

### ⚙️ Miscellaneous Tasks

- Update auth invocation macros ([#269](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/269)) - ([8161812](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/816181212d2cf9c4794f4faf5c754f0832047092))

### Contributors

* @ahramy
* @TanvirDeol
* @nbayindirli
* @cgorenflo

## [1.0.0](https://github.com/axelarnetwork/axelar-cgp-stellar/compare/stellar-upgrader-v0.2.4...stellar-upgrader-v1.0.0)

### ⚙️ Miscellaneous Tasks

- Update package descriptions ([#226](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/226)) - ([1881ec7](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/1881ec723644734f0c19c32db143e7a539f74ad3))

### Contributors

* @ahramy

## [0.2.4](https://github.com/axelarnetwork/axelar-cgp-stellar/compare/stellar-upgrader-v0.2.3...stellar-upgrader-v0.2.4)

### ⚙️ Miscellaneous Tasks

- Add build check to fail on warnings ([#227](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/227)) - ([af05c6f](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/af05c6f670f7d324eebbadb6f611412527761603))

### Contributors

* @TanvirDeol

## [0.2.3](https://github.com/axelarnetwork/axelar-cgp-stellar/compare/stellar-upgrader-v0.2.2...stellar-upgrader-v0.2.3)

### 📚 Documentation

- Add docs to contract interfaces ([#175](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/175)) - ([2f17e32](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/2f17e32b33e6d04609c3014e161ce07f9dbbef63))

### Contributors

* @TanvirDeol

## [0.2.2](https://github.com/axelarnetwork/axelar-cgp-stellar/compare/stellar-upgrader-v0.2.1...stellar-upgrader-v0.2.2)

### 🚜 Refactor

- Move test modules into lib.rs ([#199](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/199)) - ([51a638a](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/51a638a52bdaebc4928aab9e191b28a90e73f338))

### Contributors

* @AttissNgo

## [0.2.1](https://github.com/axelarnetwork/axelar-cgp-stellar/compare/stellar-upgrader-v0.2.0...stellar-upgrader-v0.2.1)

### ⚙️ Miscellaneous Tasks

- Update description for packages ([#196](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/196)) - ([a20b6ab](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/a20b6ab2633b3ca407c440b9ce35ff0071384638))

### Contributors

* @ahramy

## [0.2.0](https://github.com/axelarnetwork/axelar-cgp-stellar/compare/stellar-upgrader-v0.1.0...stellar-upgrader-v0.2.0)

### 🚜 Refactor

- [**breaking**] Rename packages and move tests under src ([#185](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/185)) - ([804c962](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/804c962a667a7889c447decf8155c4f56c7b1bdb))

### Contributors

* @ahramy

## [0.1.0]

### ⛰️ Features

- *(axelar-gateway)* Emit event when upgrade is completed ([#85](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/85)) - ([7c17383](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/7c17383e77b925e8f9d52f8d362b4e1918a6f377))
- *(upgrader)* Add generalized atomic upgrade and migration capabilities ([#77](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/77)) - ([48507e2](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/48507e256ef91a89b0a7da1fb88cbb1a5ad5ebea))
- Add upgrade capabilities to all contracts ([#87](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/87)) - ([9785e8b](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/9785e8bebea93e987af664cedea3234241675d96))

### 🚜 Refactor

- *(upgrader)* Clean up the UpgraderInterface and simplify the contract ([#84](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/84)) - ([8f298ff](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/8f298ff7585a29e6adef7cf29fdbf71c0c1e146b))
- Update mock auth macro to support non root auth  ([#134](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/134)) - ([7b6a553](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/7b6a55385fc0bdcbd7d6bf065ddaa0f81dceb51f))
- [**breaking**] Update package name and references for release ([#145](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/145)) - ([bb19538](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/bb195386eeda9c75d4da33eb0cf29fd9cb9b621c))
- Restrict exports to contract and contract clients ([#103](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/103)) - ([4c25023](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/4c250237afce95fcd687f74e350b6b272a3d295d))
- Extract ownership management into sharable interface ([#97](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/97)) - ([df2d7d8](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/df2d7d8106e26c143757d26dfc321ffd5778d23b))
- Move shared interfaces in preparation of ownership trait extraction ([#96](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/96)) - ([e63006a](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/e63006a4f17abccbd1922389f1c03cc1735220b3))

### ⚙️ Miscellaneous Tasks

- Use rustfmt nightly build to introduce opinionated imports ordering ([#141](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/141)) - ([e19f588](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/e19f5887dcb7f648d1aacb0fedbd6dfa9bf45eb2))

### Contributors

* @ahramy
* @cgorenflo
* @milapsheth
