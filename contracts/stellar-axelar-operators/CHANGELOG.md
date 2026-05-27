# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.4](https://github.com/axelarnetwork/axelar-amplifier-stellar/compare/stellar-axelar-operators-v1.1.3...stellar-axelar-operators-v1.1.4)

### ⚙️ Miscellaneous Tasks

- Updated the following local packages: stellar-axelar-std, stellar-axelar-std - ([0000000](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/0000000))

## [1.1.3](https://github.com/axelarnetwork/axelar-amplifier-stellar/compare/stellar-axelar-operators-v1.1.2...stellar-axelar-operators-v1.1.3)

### ⚙️ Miscellaneous Tasks

- Updated the following local packages: stellar-axelar-std, stellar-axelar-std - ([0000000](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/0000000))

## [1.1.2](https://github.com/axelarnetwork/axelar-amplifier-stellar/compare/stellar-axelar-operators-v1.1.1...stellar-axelar-operators-v1.1.2)

### 📚 Documentation

- Add READMEs to all crates ([#340](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/340)) - ([606affd](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/606affdb67172cb6d6812f8d08f43b8f4ae6df95))

### ⚙️ Miscellaneous Tasks

- *(contracts)* Remove v1.1 migration handlers ([#339](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/339)) - ([4f6e796](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/4f6e79649d2ac370bb11818cb79a3d9be2c30d01))

### Contributors

* @nbayindirli

## [1.1.1](https://github.com/axelarnetwork/axelar-amplifier-stellar/compare/stellar-axelar-operators-v1.1.0...stellar-axelar-operators-v1.1.1)

### ⚙️ Miscellaneous Tasks

- Bump up patch versions ([#331](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/331)) - ([75a25c2](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/75a25c23e2103baa0c9a723380c716ebad6e8798))

### Contributors

* @ahramy

## [1.1.0](https://github.com/axelarnetwork/axelar-amplifier-stellar/compare/stellar-axelar-operators-v1.0.0...stellar-axelar-operators-v1.1.0)

### ⛰️ Features

- Block regular contract endpoints during migration ([#279](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/279)) - ([7444057](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/7444057f85f73ff8a65eedbd5ae0aad77c2e7ad4))
- Add only_owner and only_operator macros ([#240](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/240)) - ([bf26705](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/bf267059dd047475c7efb7e9bee47b40eaec4bbd))

### 🐛 Bug Fixes

- *(axelar-std-derive)* Enforce contractstorage enums are private ([#267](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/267)) - ([b9c5688](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/b9c568830c5207f68104bf9c9156e0c851722b98))
- Update compile gh action to build and test all contracts separately ([#322](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/322)) - ([f6d3623](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/f6d3623d79655a9f48dbb1db77f48aa08545b651))
- Remove redundant ttl extensions ([#259](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/259)) - ([573ea7b](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/573ea7bbbaa2811e9d569c810dd5988c3f3e5d2b))
- Avoid ignoring dead_code warnings ([#257](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/257)) - ([05c4b8a](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/05c4b8ae47cdf8383dad5fd2b29f9dbe6fcc9026))

### 🚜 Refactor

- *(axelar-operators)* Remove soroban-sdk ([#293](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/293)) - ([2775c37](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/2775c37139bd900ff64024adaf6fc76c7823a21c))
- *(axelar-operators)* Migrate Operators to Operator ([#252](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/252)) - ([dc76cb3](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/dc76cb3b6b11f13e906c54d1179c2fa157a4449d))
- *(axelar-operators)* Rename Operators status to Operator ([#249](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/249)) - ([319cd74](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/319cd74f6123bfbeaa1f8f425bc52e639d4926b7))
- *(axelar-operators)* Use contractstorage ([#244](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/244)) - ([aa1f167](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/aa1f16704d1d2841b0e382443d8b1b42db341f3d))
- *(axelar-std-derive)* Simplify upgradable macro ([#256](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/256)) - ([e5fee26](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/e5fee262c1ff0a848a94d4a4109c45901283dcc7))
- Move the run_migration function into a clearly defined interface ([#239](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/239)) - ([7bd306d](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/7bd306d9d2d4f1045814decd569188c29486d924))

### 📚 Documentation

- Fix docs publish action ([#236](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/236)) - ([cbbc410](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/cbbc41005435baf20809c892b196f468c55b84d1))

### ⚙️ Miscellaneous Tasks

- Remove all unused derive macros ([#258](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/258)) - ([46a36d5](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/46a36d57359bc1a4854261f88953f6f40d399b51))

### Contributors

* @ahramy
* @cgorenflo
* @nbayindirli
* @TanvirDeol
* @milapsheth

## [1.0.0](https://github.com/axelarnetwork/axelar-cgp-stellar/compare/stellar-axelar-operators-v0.2.3...stellar-axelar-operators-v1.0.0)

### ⚙️ Miscellaneous Tasks

- Update package descriptions ([#226](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/226)) - ([1881ec7](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/1881ec723644734f0c19c32db143e7a539f74ad3))

### Contributors

* @ahramy

## [0.2.3](https://github.com/axelarnetwork/axelar-cgp-stellar/compare/stellar-axelar-operators-v0.2.2...stellar-axelar-operators-v0.2.3)

### 📚 Documentation

- Add docs to contract interfaces ([#175](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/175)) - ([2f17e32](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/2f17e32b33e6d04609c3014e161ce07f9dbbef63))

### Contributors

* @TanvirDeol

## [0.2.2](https://github.com/axelarnetwork/axelar-cgp-stellar/compare/stellar-axelar-operators-v0.2.1...stellar-axelar-operators-v0.2.2)

### 🚜 Refactor

- Move test modules into lib.rs ([#199](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/199)) - ([51a638a](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/51a638a52bdaebc4928aab9e191b28a90e73f338))

### Contributors

* @AttissNgo

## [0.2.1](https://github.com/axelarnetwork/axelar-cgp-stellar/compare/stellar-axelar-operators-v0.2.0...stellar-axelar-operators-v0.2.1)

### ⚙️ Miscellaneous Tasks

- Update description for packages ([#196](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/196)) - ([a20b6ab](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/a20b6ab2633b3ca407c440b9ce35ff0071384638))

### Contributors

* @ahramy

## [0.2.0](https://github.com/axelarnetwork/axelar-cgp-stellar/compare/stellar-axelar-operators-v0.1.0...stellar-axelar-operators-v0.2.0)

### 🚜 Refactor

- [**breaking**] Rename packages and move tests under src ([#185](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/185)) - ([804c962](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/804c962a667a7889c447decf8155c4f56c7b1bdb))

### Contributors

* @ahramy

## [0.1.0]

### ⛰️ Features

- *(Operators)* Execute method ([#15](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/15)) - ([20bbf95](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/20bbf95a6ba486f48c6ec116e31d34110f912880))
- *(axelar-gateway)* Increase gateway coverage and add coverage report ([#25](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/25)) - ([2c6f9f9](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/2c6f9f96f59b74d521aec090d9e31908ab307134))
- *(axelar-operators)* Improve error handling ([#24](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/24)) - ([c063879](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/c063879442a39a0ea43beb5387516a10aee96670))
- *(interchain-token)* Update interchain token ([#71](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/71)) - ([6440cf8](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/6440cf86ea665ed72e8515c0fb01d4fc93f2f63d))
- *(operators)* Add contract constructor ([#69](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/69)) - ([8db4014](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/8db4014376ea5e2fc00c4a7d39e56e4952b01a9e))
- Add extend ttl to all contracts ([#124](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/124)) - ([ab4361c](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/ab4361c58daffebd099ab386910b55a4d56d152f))
- Add macros for shared interfaces ([#105](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/105)) - ([4f513f9](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/4f513f933d290cc9cc5944e5e39bcda13a136906))
- Add upgrade capabilities to all contracts ([#87](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/87)) - ([9785e8b](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/9785e8bebea93e987af664cedea3234241675d96))
- Raise clippy lint level to `clippy::nursery` and apply lints ([#47](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/47)) - ([52951e1](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/52951e11f500b83f6cb31a3cadb845c4841af6a4))
- Operators contract ([#14](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/14)) - ([81a8f0e](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/81a8f0e91d89fbae4c61d9fb5790250c892ff6a7))

### 🚜 Refactor

- *(axelar-gateway)* [**breaking**] Use more readable event symbols ([#41](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/41)) - ([3e7d28a](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/3e7d28a8806fec2c689989b2e50de1860587190c))
- *(gas-service,operators)* Move out of `axelar-soroban-interfaces` ([#43](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/43)) - ([c7a9d9f](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/c7a9d9f6b2f346efa4b1f836f00bd591eea84be8))
- Rename assert_auth macros ([#138](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/138)) - ([8239e41](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/8239e4126cdccb4156f737dd6e20fad5c2bfc239))
- [**breaking**] Update package name and references for release ([#145](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/145)) - ([bb19538](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/bb195386eeda9c75d4da33eb0cf29fd9cb9b621c))
- Restrict exports to contract and contract clients ([#103](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/103)) - ([4c25023](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/4c250237afce95fcd687f74e350b6b272a3d295d))
- Extract ownership management into sharable interface ([#97](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/97)) - ([df2d7d8](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/df2d7d8106e26c143757d26dfc321ffd5778d23b))
- Move shared interfaces in preparation of ownership trait extraction ([#96](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/96)) - ([e63006a](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/e63006a4f17abccbd1922389f1c03cc1735220b3))
- Move contract tests to integration tests ([#49](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/49)) - ([5ed9513](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/5ed95130e5cc11690d0738c427adaa2b61ad4c90))

### 🧪 Testing

- Check auth is used in assert_auth ([#151](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/151)) - ([4d8e920](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/4d8e92065d528cd48a08319449b80f32322e5b08))

### ⚙️ Miscellaneous Tasks

- Use rustfmt nightly build to introduce opinionated imports ordering ([#141](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/141)) - ([e19f588](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/e19f5887dcb7f648d1aacb0fedbd6dfa9bf45eb2))
- Add the support for release pipeline ([#54](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/54)) - ([90d4368](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/90d436811258b54ee8efbac074da515e977eb47e))
- Rename dev_dependencies to dev-dependencies ([#61](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/61)) - ([47c6576](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/47c657656cf83105c46b64b98d85c0653212d528))
- Remove `axelar-soroban-interfaces` crate ([#46](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/46)) - ([514d8a4](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/514d8a441ab30587dd953004894596147298fec7))

### Contributors

* @milapsheth
* @TanvirDeol
* @ahramy
* @cgorenflo
* @AttissNgo
* @hydrobeam
* @apolikamixitos
* @re1ro
* @deanamiel
