# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.1.0](https://github.com/axelarnetwork/axelar-amplifier-stellar/compare/stellar-axelar-std-derive-v2.0.0...stellar-axelar-std-derive-v2.1.0)

### ⛰️ Features

- Add flow limiter role to its ([#381](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/381)) - ([e181c89](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/e181c8980f2fe259e56269d8bca3c99ee167c835))

### Contributors

* @rista404

## [2.0.0](https://github.com/axelarnetwork/axelar-amplifier-stellar/compare/stellar-axelar-std-derive-v1.1.2...stellar-axelar-std-derive-v2.0.0)

### ⛰️ Features

- [**breaking**] Upgrade soroban sdk to v25.3.0 ([#371](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/371)) - ([95190d2](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/95190d26120da22f8defdf37566c1cd8fd1f3e77))

### Contributors

* @rista404

## [1.1.2](https://github.com/axelarnetwork/axelar-amplifier-stellar/compare/stellar-axelar-std-derive-v1.1.1...stellar-axelar-std-derive-v1.1.2)

### 📚 Documentation

- Add READMEs to all crates ([#340](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/340)) - ([606affd](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/606affdb67172cb6d6812f8d08f43b8f4ae6df95))

### Contributors

* @nbayindirli

## [1.1.1](https://github.com/axelarnetwork/axelar-amplifier-stellar/compare/stellar-axelar-std-derive-v1.1.0...stellar-axelar-std-derive-v1.1.1)

### ⚙️ Miscellaneous Tasks

- Bump up patch versions ([#331](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/331)) - ([75a25c2](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/75a25c23e2103baa0c9a723380c716ebad6e8798))

### Contributors

* @ahramy

## [1.1.0](https://github.com/axelarnetwork/axelar-amplifier-stellar/compare/stellar-axelar-std-derive-v1.0.0...stellar-axelar-std-derive-v1.1.0)

### ⛰️ Features

- *(axelar-std-derive)* Globally extend instance TTL on all contract endpoints ([#310](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/310)) - ([ea69b2d](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/ea69b2d5f403a50ba75e3c5f28aa9f694aff7acb))
- *(axelar-std-derive)* Add status support to contractstorage macro ([#234](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/234)) - ([4ffb0cb](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/4ffb0cbef155ce4185a4c41ab45258c10527f598))
- *(axelar-std-derive)* Add contractstorage attribute macro ([#216](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/216)) - ([94632d8](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/94632d86cc36315c19750777aa0bf5724d104d7f))
- Check authorization at the root in the upgrader contract ([#294](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/294)) - ([ce4edb7](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/ce4edb7c7e63176e801d2eddb808a68e087cb4fd))
- Make the soroban-sdk available through stellar-axelar-std ([#284](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/284)) - ([4360324](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/4360324ec3728fefa16ece7058889142a7fcb5c2))
- Add has_* storage method to contractstorage ([#285](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/285)) - ([dddd7bc](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/dddd7bc0c821d54e72d0909fcf025b21851505c8))
- Block regular contract endpoints during migration ([#279](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/279)) - ([7444057](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/7444057f85f73ff8a65eedbd5ae0aad77c2e7ad4))
- Add custom executable interface for AxelarExecutableInterface for ease of use ([#265](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/265)) - ([53103fe](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/53103febaab2bf0c5e9a1a7df4f38336e0a4f50d))
- Add only_owner and only_operator macros ([#240](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/240)) - ([bf26705](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/bf267059dd047475c7efb7e9bee47b40eaec4bbd))

### 🐛 Bug Fixes

- *(axelar-std-derive)* Fix syn macro dependency ([#321](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/321)) - ([ae7cb1c](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/ae7cb1cd4eba10bfd6e6db0c8e354abf6ad8dba0))
- *(axelar-std-derive)* Support datum in schema_impl formatting ([#312](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/312)) - ([5994693](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/5994693a3315a77755a03deccbc3fd11afcf8233))
- *(axelar-std-derive)* Enforce contractstorage enums are private ([#267](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/267)) - ([b9c5688](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/b9c568830c5207f68104bf9c9156e0c851722b98))
- *(interchain-token-service)* Remove redundant token_id_config_with_extended_ttl function ([#297](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/297)) - ([600a1d3](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/600a1d35bc6211ae02e3b53be7e7fb13dd2d8002))

### 🚜 Refactor

- *(axelar-operators)* Migrate Operators to Operator ([#252](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/252)) - ([dc76cb3](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/dc76cb3b6b11f13e906c54d1179c2fa157a4449d))
- *(axelar-std-derive)* Rename storage.rs to contractstorage.rs ([#309](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/309)) - ([0afb374](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/0afb374c1e2b2bbc54eeecc4ceee3e7cf878a794))
- *(axelar-std-derive)* Simplify upgradable macro ([#256](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/256)) - ([e5fee26](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/e5fee262c1ff0a848a94d4a4109c45901283dcc7))
- *(axelar-std-derive)* Simplify contractstorage macro ([#241](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/241)) - ([bdc02e6](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/bdc02e640c07a81758e487269a5473fcccf54b37))
- *(interchain-token)* Use contractstorage ([#245](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/245)) - ([0e7970b](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/0e7970b3f46f308c803874a7d9166e22da1f3a0f))
- *(interchain-token-service)* Use contractstorage ([#246](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/246)) - ([94b47ef](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/94b47ef469c84048eb3b56e7adc951effc3f3733))
- Move the run_migration function into a clearly defined interface ([#239](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/239)) - ([7bd306d](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/7bd306d9d2d4f1045814decd569188c29486d924))

### 📚 Documentation

- Fix docs publish action ([#236](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/236)) - ([cbbc410](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/cbbc41005435baf20809c892b196f468c55b84d1))

### 🧪 Testing

- *(interchain-token)* Add tests for token events ([#286](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/286)) - ([aab11b2](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/aab11b221c218986be728d696aea66380574c326))

### Contributors

* @ahramy
* @nbayindirli
* @TanvirDeol
* @cgorenflo
* @milapsheth

## [1.0.0](https://github.com/axelarnetwork/axelar-cgp-stellar/compare/stellar-axelar-std-derive-v0.3.0...stellar-axelar-std-derive-v1.0.0)

### ⚙️ Miscellaneous Tasks

- Update package descriptions ([#226](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/226)) - ([1881ec7](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/1881ec723644734f0c19c32db143e7a539f74ad3))

### Contributors

* @ahramy

## [0.3.0](https://github.com/axelarnetwork/axelar-cgp-stellar/compare/stellar-axelar-std-derive-v0.2.1...stellar-axelar-std-derive-v0.3.0)

### ⛰️ Features

- *(axelar-std)* [**breaking**] Add pausable interface ([#204](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/204)) - ([0d4af95](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/0d4af958562e502df15dcd6bc50ec4ec66cbae46))
- *(axelar-std-derive)* Add macro to execute when contract is not paused ([#214](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/214)) - ([03d1a48](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/03d1a48b8ad9d0f4f87fc18d1ffbe6405c814fb5))

### 🐛 Bug Fixes

- *(axelar-std-derive)* Cleanup dependencies ([#213](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/213)) - ([c986ce8](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/c986ce8f689d118e78f6d1435bbe7bffd42ad3fd))

### 🎨 Styling

- *(axelar-std-derive)* Conform IntoEvent nomenclature to other derive macros ([#206](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/206)) - ([b583c7f](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/b583c7f5d11f1e865ce3283dc7b762f51b89a2ae))

### Contributors

* @milapsheth
* @nbayindirli

## [0.2.1](https://github.com/axelarnetwork/axelar-cgp-stellar/compare/stellar-axelar-std-derive-v0.2.0...stellar-axelar-std-derive-v0.2.1)

### 🐛 Bug Fixes

- *(axelar-std)* Feature gate exported interfaces ([#192](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/192)) - ([f8e469a](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/f8e469a2cad2fea2f0e9aa8b2321329d0a1c560d))

### ⚙️ Miscellaneous Tasks

- Update description for packages ([#196](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/196)) - ([a20b6ab](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/a20b6ab2633b3ca407c440b9ce35ff0071384638))

### Contributors

* @ahramy
* @milapsheth

## [0.2.0](https://github.com/axelarnetwork/axelar-cgp-stellar/compare/stellar-axelar-std-derive-v0.1.1...stellar-axelar-std-derive-v0.2.0)

### 🚜 Refactor

- [**breaking**] Rename packages and move tests under src ([#185](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/185)) - ([804c962](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/804c962a667a7889c447decf8155c4f56c7b1bdb))

### ⚙️ Miscellaneous Tasks

- Update release plz and version for release ([#190](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/190)) - ([482b35b](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/482b35b50bf9542e30515bdccbc474305830ad2f))

### Contributors

* @ahramy

## [0.1.0]

### ⛰️ Features

- Simplify event definition via IntoEvent derive macro ([#136](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/136)) - ([9052c78](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/9052c7886b8d2ea12f33a1fdcceaa7d159890c4e))

### 🚜 Refactor

- Rename assert_auth macros ([#138](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/138)) - ([8239e41](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/8239e4126cdccb4156f737dd6e20fad5c2bfc239))
- [**breaking**] Update package name and references for release ([#145](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/145)) - ([bb19538](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/bb195386eeda9c75d4da33eb0cf29fd9cb9b621c))

### Contributors

* @nbayindirli
* @TanvirDeol
* @ahramy
