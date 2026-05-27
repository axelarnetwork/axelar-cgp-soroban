# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.4](https://github.com/axelarnetwork/axelar-amplifier-stellar/compare/stellar-token-manager-v1.1.3...stellar-token-manager-v1.1.4)

### ⚙️ Miscellaneous Tasks

- Updated the following local packages: stellar-axelar-std, stellar-axelar-std - ([0000000](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/0000000))

## [1.1.3](https://github.com/axelarnetwork/axelar-amplifier-stellar/compare/stellar-token-manager-v1.1.2...stellar-token-manager-v1.1.3)

### ⚙️ Miscellaneous Tasks

- Updated the following local packages: stellar-axelar-std, stellar-axelar-std - ([0000000](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/0000000))

## [1.1.2](https://github.com/axelarnetwork/axelar-amplifier-stellar/compare/stellar-token-manager-v1.1.1...stellar-token-manager-v1.1.2)

### 📚 Documentation

- Add READMEs to all crates ([#340](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/340)) - ([606affd](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/606affdb67172cb6d6812f8d08f43b8f4ae6df95))

### ⚙️ Miscellaneous Tasks

- *(contracts)* Remove v1.1 migration handlers ([#339](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/339)) - ([4f6e796](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/4f6e79649d2ac370bb11818cb79a3d9be2c30d01))

### Contributors

* @nbayindirli

## [1.1.1](https://github.com/axelarnetwork/axelar-amplifier-stellar/compare/stellar-token-manager-v1.1.0...stellar-token-manager-v1.1.1)

### ⚙️ Miscellaneous Tasks

- Bump up patch versions ([#331](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/331)) - ([75a25c2](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/75a25c23e2103baa0c9a723380c716ebad6e8798))

### Contributors

* @ahramy

## [1.1.0](https://github.com/axelarnetwork/axelar-amplifier-stellar/compare/stellar-token-manager-v1.0.0...stellar-token-manager-v1.1.0)

### ⛰️ Features

- *(axelar-std-derive)* Globally extend instance TTL on all contract endpoints ([#310](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/310)) - ([ea69b2d](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/ea69b2d5f403a50ba75e3c5f28aa9f694aff7acb))
- Block regular contract endpoints during migration ([#279](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/279)) - ([7444057](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/7444057f85f73ff8a65eedbd5ae0aad77c2e7ad4))
- Add only_owner and only_operator macros ([#240](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/240)) - ([bf26705](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/bf267059dd047475c7efb7e9bee47b40eaec4bbd))

### 🐛 Bug Fixes

- Update compile gh action to build and test all contracts separately ([#322](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/322)) - ([f6d3623](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/f6d3623d79655a9f48dbb1db77f48aa08545b651))
- Avoid ignoring dead_code warnings ([#257](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/257)) - ([05c4b8a](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/05c4b8ae47cdf8383dad5fd2b29f9dbe6fcc9026))

### 🚜 Refactor

- *(axelar-std-derive)* Simplify upgradable macro ([#256](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/256)) - ([e5fee26](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/e5fee262c1ff0a848a94d4a4109c45901283dcc7))
- *(interchain-token-service)* Migrate token-manager, interchain-token & destruct flow keys ([#280](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/280)) - ([8990431](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/89904314cf900e161241c516b98e923cb1ee605e))
- *(token-manager)* Remove soroban-sdk ([#290](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/290)) - ([1058471](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/1058471a7f96bbe9819a24291312d8385fa8bd51))
- Move the run_migration function into a clearly defined interface ([#239](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/239)) - ([7bd306d](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/7bd306d9d2d4f1045814decd569188c29486d924))

### 📚 Documentation

- Fix docs publish action ([#236](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/236)) - ([cbbc410](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/cbbc41005435baf20809c892b196f468c55b84d1))

### Contributors

* @ahramy
* @nbayindirli
* @cgorenflo
* @TanvirDeol
* @milapsheth

## [1.0.0](https://github.com/axelarnetwork/axelar-cgp-stellar/compare/stellar-token-manager-v0.1.0...stellar-token-manager-v1.0.0)

### ⚙️ Miscellaneous Tasks

- Update package descriptions ([#226](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/226)) - ([1881ec7](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/1881ec723644734f0c19c32db143e7a539f74ad3))

### Contributors

* @ahramy

## [0.1.0]

### ⛰️ Features

- *(token-manager)* Add token manager for ITS ([#215](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/215)) - ([42d7b34](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/42d7b348a4b419ce77c35688f93ba803c2e5ef1e))

### Contributors

* @milapsheth
