# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.0.2](https://github.com/axelarnetwork/axelar-amplifier-stellar/compare/stellar-axelar-std-v2.0.1...stellar-axelar-std-v2.0.2)

### 🐛 Bug Fixes

- *(axelar-std-derive)* Avoid internal aliases in contract specs ([#387](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/387)) - ([1c9026f](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/1c9026f47a823cd5937b92a04cf3e5b5e06f1238))

### ⚙️ Miscellaneous Tasks

- Bump rust toolchain to 1.93 ([#386](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/386)) - ([8af0fd7](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/8af0fd7a700486040c41cb09db0f134be0caab3e))

### Contributors

* @rista404

## [2.0.1](https://github.com/axelarnetwork/axelar-amplifier-stellar/compare/stellar-axelar-std-v2.0.0...stellar-axelar-std-v2.0.1)

### ⚙️ Miscellaneous Tasks

- Updated the following local packages: stellar-axelar-std-derive, stellar-axelar-std-derive - ([0000000](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/0000000))

## [2.0.0](https://github.com/axelarnetwork/axelar-amplifier-stellar/compare/stellar-axelar-std-v1.1.2...stellar-axelar-std-v2.0.0)

### ⛰️ Features

- [**breaking**] Upgrade soroban sdk to v25.3.0 ([#371](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/371)) - ([95190d2](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/95190d26120da22f8defdf37566c1cd8fd1f3e77))

### Contributors

* @rista404

## [1.1.2](https://github.com/axelarnetwork/axelar-amplifier-stellar/compare/stellar-axelar-std-v1.1.1...stellar-axelar-std-v1.1.2)

### 📚 Documentation

- Add READMEs to all crates ([#340](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/340)) - ([606affd](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/606affdb67172cb6d6812f8d08f43b8f4ae6df95))

### Contributors

* @nbayindirli

## [1.1.1](https://github.com/axelarnetwork/axelar-amplifier-stellar/compare/stellar-axelar-std-v1.1.0...stellar-axelar-std-v1.1.1)

### ⚙️ Miscellaneous Tasks

- Bump up patch versions ([#331](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/331)) - ([75a25c2](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/75a25c23e2103baa0c9a723380c716ebad6e8798))

### Contributors

* @ahramy

## [1.1.0](https://github.com/axelarnetwork/axelar-amplifier-stellar/compare/stellar-axelar-std-v1.0.0...stellar-axelar-std-v1.1.0)

### ⛰️ Features

- *(axelar-std-derive)* Add status support to contractstorage macro ([#234](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/234)) - ([4ffb0cb](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/4ffb0cbef155ce4185a4c41ab45258c10527f598))
- *(axelar-std-derive)* Add contractstorage attribute macro ([#216](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/216)) - ([94632d8](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/94632d86cc36315c19750777aa0bf5724d104d7f))
- Check authorization at the root in the upgrader contract ([#294](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/294)) - ([ce4edb7](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/ce4edb7c7e63176e801d2eddb808a68e087cb4fd))
- Make the soroban-sdk available through stellar-axelar-std ([#284](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/284)) - ([4360324](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/4360324ec3728fefa16ece7058889142a7fcb5c2))
- Block regular contract endpoints during migration ([#279](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/279)) - ([7444057](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/7444057f85f73ff8a65eedbd5ae0aad77c2e7ad4))
- Add custom executable interface for AxelarExecutableInterface for ease of use ([#265](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/265)) - ([53103fe](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/53103febaab2bf0c5e9a1a7df4f38336e0a4f50d))
- Increase threshold for extending storage rent ([#266](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/266)) - ([305c527](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/305c52772ac240da84c86a08eddef71c7703a85a))
- Add only_owner and only_operator macros ([#240](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/240)) - ([bf26705](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/bf267059dd047475c7efb7e9bee47b40eaec4bbd))

### 🐛 Bug Fixes

- *(interchain-token-service)* Ensure token metadata has valid ASCII encoding ([#263](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/263)) - ([3a351b5](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/3a351b5583c482501d404b3cb8a59a65616a29e5))

### 🚜 Refactor

- *(axelar-operators)* Migrate Operators to Operator ([#252](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/252)) - ([dc76cb3](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/dc76cb3b6b11f13e906c54d1179c2fa157a4449d))
- *(axelar-std)* Use contractstorage for macro storage ([#276](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/276)) - ([8bb4dba](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/8bb4dba9aa2675d53332b53a4cbae52099af5143))
- *(axelar-std-derive)* Simplify upgradable macro ([#256](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/256)) - ([e5fee26](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/e5fee262c1ff0a848a94d4a4109c45901283dcc7))
- *(axelar-std-derive)* Simplify contractstorage macro ([#241](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/241)) - ([bdc02e6](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/bdc02e640c07a81758e487269a5473fcccf54b37))
- *(interchain-token-service)* Use contractstorage ([#246](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/246)) - ([94b47ef](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/94b47ef469c84048eb3b56e7adc951effc3f3733))
- *(upgrader)* Remove soroban-sdk ([#295](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/295)) - ([473fb39](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/473fb390e142e2452343f91baa19b93924640389))
- Move ownable and operatable modules to new test files ([#262](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/262)) - ([494ae90](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/494ae908201039a1376870e70dca3abaf4d183ef))
- Move the run_migration function into a clearly defined interface ([#239](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/239)) - ([7bd306d](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/7bd306d9d2d4f1045814decd569188c29486d924))

### ⚙️ Miscellaneous Tasks

- Update auth invocation macros ([#269](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/269)) - ([8161812](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/816181212d2cf9c4794f4faf5c754f0832047092))
- Remove all unused derive macros ([#258](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/258)) - ([46a36d5](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/46a36d57359bc1a4854261f88953f6f40d399b51))

### Contributors

* @cgorenflo
* @ahramy
* @nbayindirli
* @TanvirDeol

## [1.0.0](https://github.com/axelarnetwork/axelar-cgp-stellar/compare/stellar-axelar-std-v0.3.0...stellar-axelar-std-v1.0.0)

### ⚙️ Miscellaneous Tasks

- Update package descriptions ([#226](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/226)) - ([1881ec7](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/1881ec723644734f0c19c32db143e7a539f74ad3))

### Contributors

* @ahramy

## [0.3.0](https://github.com/axelarnetwork/axelar-cgp-stellar/compare/stellar-axelar-std-v0.2.2...stellar-axelar-std-v0.3.0)

### ⛰️ Features

- *(axelar-gateway)* Add more queries ([#207](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/207)) - ([ca3b486](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/ca3b4861a1a26b63cad5f12daa86a71a29107cee))
- *(axelar-std)* [**breaking**] Add pausable interface ([#204](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/204)) - ([0d4af95](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/0d4af958562e502df15dcd6bc50ec4ec66cbae46))
- *(axelar-std-derive)* Add macro to execute when contract is not paused ([#214](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/214)) - ([03d1a48](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/03d1a48b8ad9d0f4f87fc18d1ffbe6405c814fb5))
- *(token-manager)* Add token manager for ITS ([#215](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/215)) - ([42d7b34](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/42d7b348a4b419ce77c35688f93ba803c2e5ef1e))

### 🐛 Bug Fixes

- *(axelar-std-derive)* Cleanup dependencies ([#213](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/213)) - ([c986ce8](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/c986ce8f689d118e78f6d1435bbe7bffd42ad3fd))

### 🚜 Refactor

- *(interchain-token-service)* Separate ITS logic into modules ([#219](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/219)) - ([86c7bac](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/86c7bac9cf2e52d515c841dc6c4e571e12645e90))

### Contributors

* @milapsheth

## [0.2.2](https://github.com/axelarnetwork/axelar-cgp-stellar/compare/stellar-axelar-std-v0.2.1...stellar-axelar-std-v0.2.2)

### 🚜 Refactor

- Move test modules into lib.rs ([#199](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/199)) - ([51a638a](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/51a638a52bdaebc4928aab9e191b28a90e73f338))

### Contributors

* @AttissNgo

## [0.2.1](https://github.com/axelarnetwork/axelar-cgp-stellar/compare/stellar-axelar-std-v0.2.0...stellar-axelar-std-v0.2.1)

### ⚙️ Miscellaneous Tasks

- Update description for packages ([#196](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/196)) - ([a20b6ab](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/a20b6ab2633b3ca407c440b9ce35ff0071384638))

### Contributors

* @ahramy

## [0.2.0](https://github.com/axelarnetwork/axelar-cgp-stellar/compare/stellar-axelar-std-v0.1.0...stellar-axelar-std-v0.2.0)

### 🚜 Refactor

- [**breaking**] Rename packages and move tests under src ([#185](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/185)) - ([804c962](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/804c962a667a7889c447decf8155c4f56c7b1bdb))

### Contributors

* @ahramy

## [0.1.0]

### ⛰️ Features

- Simplify event definition via IntoEvent derive macro ([#136](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/136)) - ([9052c78](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/9052c7886b8d2ea12f33a1fdcceaa7d159890c4e))

### 🚜 Refactor

- Update mock auth macro to support non root auth  ([#134](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/134)) - ([7b6a553](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/7b6a55385fc0bdcbd7d6bf065ddaa0f81dceb51f))
- Rename assert_auth macros ([#138](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/138)) - ([8239e41](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/8239e4126cdccb4156f737dd6e20fad5c2bfc239))
- [**breaking**] Update package name and references for release ([#145](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/145)) - ([bb19538](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/bb195386eeda9c75d4da33eb0cf29fd9cb9b621c))

### 🧪 Testing

- Check auth is used in assert_auth ([#151](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/151)) - ([4d8e920](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/4d8e92065d528cd48a08319449b80f32322e5b08))

### Contributors

* @milapsheth
* @ahramy
* @nbayindirli
* @TanvirDeol
