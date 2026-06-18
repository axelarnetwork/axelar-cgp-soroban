# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.1.1](https://github.com/axelarnetwork/axelar-amplifier-stellar/compare/stellar-interchain-token-service-v2.1.0...stellar-interchain-token-service-v2.1.1)

### ⚙️ Miscellaneous Tasks

- *(its)* Reconstruct canonical wXRP state on Stellar after v2.0.0 … ([#384](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/384)) - ([6321e0d](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/6321e0d6e6d7b2fc1018bd205908b26bbf605241))

### Contributors

* @MakisChristou

## [2.1.0](https://github.com/axelarnetwork/axelar-amplifier-stellar/compare/stellar-interchain-token-service-v2.0.0...stellar-interchain-token-service-v2.1.0)

### ⛰️ Features

- Add flow limiter role to its ([#381](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/381)) - ([e181c89](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/e181c8980f2fe259e56269d8bca3c99ee167c835))

### Contributors

* @rista404

### ⛰️ Features

- [**breaking**] *(interchain-token-service)* Add per-token flow limiter role. The contract operator can `add_flow_limiter`, `remove_flow_limiter`, and `transfer_flow_limiter` per `token_id`. Approved flow limiters can call `set_flow_limit` for their token in addition to the operator. `set_flow_limit` signature now takes a leading `caller: Address` parameter; callers must update their call sites accordingly.

## [2.0.0](https://github.com/axelarnetwork/axelar-amplifier-stellar/compare/stellar-interchain-token-service-v1.4.1...stellar-interchain-token-service-v2.0.0)

### ⛰️ Features

- [**breaking**] Upgrade soroban sdk to v25.3.0 ([#371](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/371)) - ([95190d2](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/95190d26120da22f8defdf37566c1cd8fd1f3e77))

### ⚙️ Miscellaneous Tasks

- *(interchain-token-service)* Update comments in the interface ([#370](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/370)) - ([c3376f3](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/c3376f3be16f6e4b652302dd32eb8265769021c4))

### Contributors

* @rista404
* @ahramy

## [1.4.1](https://github.com/axelarnetwork/axelar-amplifier-stellar/compare/stellar-interchain-token-service-v1.4.0...stellar-interchain-token-service-v1.4.1)

### 🐛 Bug Fixes

- *(interchain-token-service)* Add mint burn from when receiving msg from the hub ([#367](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/367)) - ([f64fd11](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/f64fd11495148122b4336bd70f3b877c2d165fd3))

### Contributors

* @ahramy

## [1.4.0](https://github.com/axelarnetwork/axelar-amplifier-stellar/compare/stellar-interchain-token-service-v1.3.0...stellar-interchain-token-service-v1.4.0)

### ⛰️ Features

- *(interchain-token-service)* Add mint burn from token manager type ([#364](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/364)) - ([a874ba8](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/a874ba85057edd6f859085b3ba6d5e4e7c11b633))
- *(interchain-token-service)* Add transfer token admin function ([#361](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/361)) - ([feddf68](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/feddf68048113a6f25e9acad67e5af3c465687a5))
- *(interchain-token-service)* Use mint for mint burn token manager type ([#360](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/360)) - ([7ad2342](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/7ad2342b7722aecc4f8aaf1a57127604865e8ae6))
- *(interchain-token-service)* Add execute link token message ([#356](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/356)) - ([04d5629](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/04d5629c951a998737da621e9458213962fa042e))
- *(interchain-token-service)* Add link token ([#355](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/355)) - ([c2c0ede](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/c2c0ede0325e9f58d0d17626012f055ac542d913))
- *(interchain-token-service)* Add mint burn token manager type ([#353](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/353)) - ([d2edc10](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/d2edc10d08a17e1e53159538dc8f3c15cdb883dd))
- *(interchain-token-service)* Add register custom token ([#352](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/352)) - ([1005d67](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/1005d67b7e5a43c7f472af5022fd53c0512d6d10))
- *(interchain-token-service)* Add register_token_metadata hub message type ([#351](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/351)) - ([e3b3394](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/e3b3394c3f0eea41154d1115c16ed8a71a129ab5))

### 🚜 Refactor

- *(interchain-token-service)* Encapsulate token manager post deployment setup ([#357](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/357)) - ([0c17a03](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/0c17a03b058ebb47e10bd0403e3de53016c47dd2))

### Contributors

* @ahramy

## [1.3.0](https://github.com/axelarnetwork/axelar-amplifier-stellar/compare/stellar-interchain-token-service-v1.2.0...stellar-interchain-token-service-v1.3.0)

### ⛰️ Features

- *(interchain-token-service)* Add link token and register token metadata abi ([#347](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/347)) - ([2274e6a](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/2274e6ad2b1ebb9db55bbb5807c518d1fcdc99dd))

### Contributors

* @ahramy

## [1.2.0](https://github.com/axelarnetwork/axelar-amplifier-stellar/compare/stellar-interchain-token-service-v1.1.2...stellar-interchain-token-service-v1.2.0)

### ⛰️ Features

- *(interchain-token-service)* Bump up minor version ([#343](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/343)) - ([a87882c](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/a87882cc870e37df984d2bea48c9971dd6afd06e))

### 🐛 Bug Fixes

- *(interchain-token-service)* Emit data_hash instead of data in InterchainTransferSentEvent ([#342](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/342)) - ([2b90431](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/2b904316b3b209732c3ba3733b2e0e5719b32a52))

### 📚 Documentation

- Add READMEs to all crates ([#340](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/340)) - ([606affd](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/606affdb67172cb6d6812f8d08f43b8f4ae6df95))

### ⚙️ Miscellaneous Tasks

- *(contracts)* Remove v1.1 migration handlers ([#339](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/339)) - ([4f6e796](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/4f6e79649d2ac370bb11818cb79a3d9be2c30d01))

### Contributors

* @ahramy
* @nbayindirli

## [1.1.2](https://github.com/axelarnetwork/axelar-amplifier-stellar/compare/stellar-interchain-token-service-v1.1.1...stellar-interchain-token-service-v1.1.2)

### 🐛 Bug Fixes

- *(interchain-token-service)* Optimize code for creating upgrade auth entires ([#336](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/336)) - ([c86b316](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/c86b31621b4b1022f33e6bb5e80358ca6658e8d7))
- *(interchain-token-service)* Use authorize_as_current_contract in migrate_token for upgrade auth ([#334](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/334)) - ([53a2fee](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/53a2feefde1a6b715c55b7613bbbaf0ef0dac3e4))

### Contributors

* @ahramy
* @nbayindirli

## [1.1.1](https://github.com/axelarnetwork/axelar-amplifier-stellar/compare/stellar-interchain-token-service-v1.1.0...stellar-interchain-token-service-v1.1.1)

### ⚙️ Miscellaneous Tasks

- Bump up patch versions ([#331](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/331)) - ([75a25c2](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/75a25c23e2103baa0c9a723380c716ebad6e8798))

### Contributors

* @ahramy

## [1.1.0](https://github.com/axelarnetwork/axelar-amplifier-stellar/compare/stellar-interchain-token-service-v1.0.0...stellar-interchain-token-service-v1.1.0)

### ⛰️ Features

- *(interchain-token-service)* Allow operator to manage trusted chains ([#306](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/306)) - ([9fa5203](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/9fa52038525e8d9b193534c3ce309d358a5ec338))
- *(interchain-token-service)* Make gas token optional for its and example ([#261](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/261)) - ([72b67d1](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/72b67d192636fbbbcec87a737ed95eb0877eacd9))
- Optimize token metadata to skip external calls for native token ([#301](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/301)) - ([568d832](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/568d832b4327d21d5b702058fd6606d1812386cf))
- Block regular contract endpoints during migration ([#279](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/279)) - ([7444057](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/7444057f85f73ff8a65eedbd5ae0aad77c2e7ad4))
- Add custom executable interface for AxelarExecutableInterface for ease of use ([#265](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/265)) - ([53103fe](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/53103febaab2bf0c5e9a1a7df4f38336e0a4f50d))
- Add only_owner and only_operator macros ([#240](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/240)) - ([bf26705](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/bf267059dd047475c7efb7e9bee47b40eaec4bbd))

### 🐛 Bug Fixes

- *(axelar-std-derive)* Enforce contractstorage enums are private ([#267](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/267)) - ([b9c5688](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/b9c568830c5207f68104bf9c9156e0c851722b98))
- *(interchain-token)* Validate minter existence in add_minter and remove_minter ([#298](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/298)) - ([682bd12](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/682bd12c1827497161f900898b34f6f608f3772b))
- *(interchain-token-service)* Use v0.0.0 for test WASMs to avoid release clash ([#318](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/318)) - ([048237d](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/048237d5fb21931dd0de00c05dac23bf43c83018))
- *(interchain-token-service)* Handle large flow limits ([#313](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/313)) - ([3c19a97](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/3c19a97a0fe2b10774dbc83d839005c24a52e8d5))
- *(interchain-token-service)* Ensure deploy interchain token requires non zero supply or a minter ([#299](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/299)) - ([70a71ec](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/70a71ec5348d9d3068aff778b9c0071b19fbbe3d))
- *(interchain-token-service)* Cast execute_contract_with_token to Val to prevent stuck funds ([#302](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/302)) - ([2c48c5a](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/2c48c5aa52bf50dec43e031024b4cb4ef9281d27))
- *(interchain-token-service)* Emit data_hash instead of data in InterchainTransferReceivedEvent ([#296](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/296)) - ([0ea6545](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/0ea654549bb67088e5d73400a497b6ea9de9c2da))
- *(interchain-token-service)* Remove redundant token_id_config_with_extended_ttl function ([#297](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/297)) - ([600a1d3](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/600a1d35bc6211ae02e3b53be7e7fb13dd2d8002))
- *(interchain-token-service)* Ensure token metadata has valid ASCII encoding ([#263](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/263)) - ([3a351b5](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/3a351b5583c482501d404b3cb8a59a65616a29e5))
- *(interchain-token-service)* Validate token metadata on registerCanonicalToken ([#260](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/260)) - ([c8de54c](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/c8de54c9fef79bdf94b0dd3ab6bf670a5de27fd0))
- Update compile gh action to build and test all contracts separately ([#322](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/322)) - ([f6d3623](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/f6d3623d79655a9f48dbb1db77f48aa08545b651))
- Rename its token address and manager queries ([#273](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/273)) - ([f696af5](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/f696af50dc1bd1b6d1d4db1ae7f588c8ea43976f))
- Remove redundant ttl extensions ([#259](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/259)) - ([573ea7b](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/573ea7bbbaa2811e9d569c810dd5988c3f3e5d2b))
- Avoid ignoring dead_code warnings ([#257](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/257)) - ([05c4b8a](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/05c4b8ae47cdf8383dad5fd2b29f9dbe6fcc9026))

### 🚜 Refactor

- *(axelar-std-derive)* Simplify upgradable macro ([#256](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/256)) - ([e5fee26](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/e5fee262c1ff0a848a94d4a4109c45901283dcc7))
- *(interchain-token-service)* Migrate token-manager, interchain-token & destruct flow keys ([#280](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/280)) - ([8990431](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/89904314cf900e161241c516b98e923cb1ee605e))
- *(interchain-token-service)* Remove soroban-sdk ([#291](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/291)) - ([e4d6e8e](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/e4d6e8eb7ffe6f1ace612c03f238a91c20fd8ab0))
- *(interchain-token-service)* Use contractstorage ([#246](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/246)) - ([94b47ef](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/94b47ef469c84048eb3b56e7adc951effc3f3733))
- Move the run_migration function into a clearly defined interface ([#239](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/239)) - ([7bd306d](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/7bd306d9d2d4f1045814decd569188c29486d924))

### 📚 Documentation

- *(interchain-token-service)* Add INTEGRATION.md ([#235](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/235)) - ([037343a](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/037343a6f2eca3885eb76c24b606285ec4e8ec02))
- Fix docs publish action ([#236](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/236)) - ([cbbc410](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/cbbc41005435baf20809c892b196f468c55b84d1))

### 🎨 Styling

- *(interchain-token-service)* Get token_id_config from storage ([#250](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/250)) - ([a25f343](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/a25f3436e6b4c5565170905689f0d66cd6654cc9))

### ⚙️ Miscellaneous Tasks

- Update auth invocation macros ([#269](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/269)) - ([8161812](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/816181212d2cf9c4794f4faf5c754f0832047092))
- Remove all unused derive macros ([#258](https://github.com/axelarnetwork/axelar-amplifier-stellar/pull/258)) - ([46a36d5](https://github.com/axelarnetwork/axelar-amplifier-stellar/commit/46a36d57359bc1a4854261f88953f6f40d399b51))

### Contributors

* @ahramy
* @nbayindirli
* @milapsheth
* @AttissNgo
* @TanvirDeol
* @cgorenflo

## [1.0.0](https://github.com/axelarnetwork/axelar-cgp-stellar/compare/stellar-interchain-token-service-v0.3.1...stellar-interchain-token-service-v1.0.0)

### ⚙️ Miscellaneous Tasks

- Update package descriptions ([#226](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/226)) - ([1881ec7](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/1881ec723644734f0c19c32db143e7a539f74ad3))

### Contributors

* @ahramy

## [0.3.1](https://github.com/axelarnetwork/axelar-cgp-stellar/compare/stellar-interchain-token-service-v0.3.0...stellar-interchain-token-service-v0.3.1)

### 🐛 Bug Fixes

- *(interchain-token)* Unimplemented notice for SAC methods ([#225](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/225)) - ([8c31f8e](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/8c31f8e6f56ebed5909c0e448e2758ce988aadbe))

### 🧪 Testing

- *(example)* Increase ITS coverage ([#221](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/221)) - ([996f2bc](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/996f2bc8524d4c25005f5bd2b5b026b0dfaef2da))

### Contributors

* @nbayindirli
* @milapsheth

## [0.3.0](https://github.com/axelarnetwork/axelar-cgp-stellar/compare/stellar-interchain-token-service-v0.2.2...stellar-interchain-token-service-v0.3.0)

### ⛰️ Features

- *(axelar-std)* [**breaking**] Add pausable interface ([#204](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/204)) - ([0d4af95](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/0d4af958562e502df15dcd6bc50ec4ec66cbae46))
- *(axelar-std-derive)* Add macro to execute when contract is not paused ([#214](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/214)) - ([03d1a48](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/03d1a48b8ad9d0f4f87fc18d1ffbe6405c814fb5))
- *(interchain-token-service)* Add spender require auth for deploying remote canonical token ([#217](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/217)) - ([9f3a1f3](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/9f3a1f36080d2dd297f066b355b52b59641a2ed4))
- *(token-manager)* Add token manager for ITS ([#215](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/215)) - ([42d7b34](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/42d7b348a4b419ce77c35688f93ba803c2e5ef1e))

### 🚜 Refactor

- *(interchain-token-service)* Separate ITS logic into modules ([#219](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/219)) - ([86c7bac](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/86c7bac9cf2e52d515c841dc6c4e571e12645e90))

### 📚 Documentation

- Add docs to contract interfaces ([#175](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/175)) - ([2f17e32](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/2f17e32b33e6d04609c3014e161ce07f9dbbef63))

### ⚙️ Miscellaneous Tasks

- *(interchain-token-service)* Verify input validation ([#208](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/208)) - ([f160c1d](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/f160c1d28ef30cf28b90c0a5a4942372fa7d5f24))

### Contributors

* @TanvirDeol
* @milapsheth
* @ahramy
* @AttissNgo

## [0.2.2](https://github.com/axelarnetwork/axelar-cgp-stellar/compare/stellar-interchain-token-service-v0.2.1...stellar-interchain-token-service-v0.2.2)

### 🚜 Refactor

- Move test modules into lib.rs ([#199](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/199)) - ([51a638a](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/51a638a52bdaebc4928aab9e191b28a90e73f338))

### ⚙️ Miscellaneous Tasks

- *(its)* Remove unused IntoVal imports ([#200](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/200)) - ([d68aa73](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/d68aa730fb3f44e559387001a6ebe528fe087666))

### Contributors

* @nbayindirli
* @AttissNgo

## [0.2.1](https://github.com/axelarnetwork/axelar-cgp-stellar/compare/stellar-interchain-token-service-v0.2.0...stellar-interchain-token-service-v0.2.1)

### ⚙️ Miscellaneous Tasks

- Update description for packages ([#196](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/196)) - ([a20b6ab](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/a20b6ab2633b3ca407c440b9ce35ff0071384638))

### Contributors

* @ahramy

## [0.2.0](https://github.com/axelarnetwork/axelar-cgp-stellar/compare/stellar-interchain-token-service-v0.1.0...stellar-interchain-token-service-v0.2.0)

### 🚜 Refactor

- [**breaking**] Rename packages and move tests under src ([#185](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/185)) - ([804c962](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/804c962a667a7889c447decf8155c4f56c7b1bdb))

### Contributors

* @ahramy

## [0.1.0]

### ⛰️ Features

- *(ITS)* Add chain name and token deploy salt and token sdk to ITS ([#95](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/95)) - ([017d421](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/017d421eb8c131a84de1b49fca89a45b094e2da9))
- *(ITS)* Add ITS message ABI encoding/decoding support ([#65](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/65)) - ([9c49a73](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/9c49a73346d161b8bd52060ef764db04464ceb80))
- *(ITS)* Allow owner to add/remove trusted addresses ([#39](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/39)) - ([6dddb12](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/6dddb124a73c40de17b0e88aa570edeb6db4efc5))
- *(axelar-soroban-std)* Allow typed matching of emitted events ([#92](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/92)) - ([7a410ca](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/7a410cab94a280777361e75c73675431b2c1be2f))
- *(interchain-token-service)* Add flow limit ([#130](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/130)) - ([7da3677](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/7da36774cfa4f44d7bd66de9ac04f8ed0d4c6160))
- *(interchain-token-service)* Library export ([#126](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/126)) - ([7adc13d](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/7adc13d91bd322c9d62cebcca11aa63c7d9c5cbf))
- *(interchain-token-service)* Deploy remote canonical token ([#123](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/123)) - ([bec2a07](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/bec2a0723a4e42a6c1db0c435cc65f5a07898326))
- *(interchain-token-service)* Process deploy interchain token message ([#122](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/122)) - ([bfc7ded](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/bfc7ded743699b4e6d50d721876cd5c2f7db293b))
- *(interchain-token-service)* Remote interchain token ([#118](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/118)) - ([6ec2622](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/6ec26221bd6a7583b65bde93a2f69a7abb4dacb9))
- *(interchain-token-service)* Register canonical interchain token ([#119](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/119)) - ([7cce18f](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/7cce18fbf483648456c11c799c334430d6475c46))
- *(interchain-token-service)* Add interchain token executable ([#120](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/120)) - ([daf70ec](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/daf70ec484eb6c50fcf30ac0e0ca874fdea729ff))
- *(interchain-token-service)* Add interchain_transfer implementation ([#115](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/115)) - ([ff1f206](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/ff1f2068702f09babb3d0b3afe4a5ebee7f7bbdf))
- *(interchain-token-service)* Store token id as token data ([#112](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/112)) - ([1ddfef5](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/1ddfef51c8b9fc7689e90b233b2f3d9754d8d942))
- *(interchain-token-service)* Deploy interchain token ([#99](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/99)) - ([bdf9443](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/bdf9443d55f142a333a5d39a059c9f7479327ce4))
- *(interchain-token-service)* Add message routing ([#90](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/90)) - ([e06032c](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/e06032cb8b90e2e8c78cd0a93a4dad4da588df75))
- *(interchain-token-service,interchain-token)* Add contract constructor ([#74](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/74)) - ([4cbaab3](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/4cbaab3f1fed2878a1ad5259c40d361b85a4747f))
- *(its)* Update event test with new event test utils ([#114](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/114)) - ([1761b73](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/1761b733706f3e522d7cf060b04b697da878bee7))
- *(its)* Add skeleton code for its token deployment ([#88](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/88)) - ([b062cf1](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/b062cf1eb9f26ef2ceeebeded732fd40e58f48f4))
- *(its)* Add ITS message types ([#55](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/55)) - ([9b62384](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/9b6238401ccdd087c8a8d9a6516fc2537dcac8ba))
- The execute function of the ExecutableInterface trait returns a result ([#132](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/132)) - ([47d92ee](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/47d92eec27cf9dc5d7a850a1e4a70f810a75da06))
- Add extend ttl to all contracts ([#124](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/124)) - ([ab4361c](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/ab4361c58daffebd099ab386910b55a4d56d152f))
- Add macros for shared interfaces ([#105](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/105)) - ([4f513f9](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/4f513f933d290cc9cc5944e5e39bcda13a136906))
- Add upgrade capabilities to all contracts ([#87](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/87)) - ([9785e8b](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/9785e8bebea93e987af664cedea3234241675d96))
- Raise clippy lint level to `clippy::nursery` and apply lints ([#47](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/47)) - ([52951e1](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/52951e11f500b83f6cb31a3cadb845c4841af6a4))

### 🐛 Bug Fixes

- *(interchain-token-service)* Add destination validation checks ([#150](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/150)) - ([1b77756](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/1b77756e62f35f7ac016dab7fbba9e2a06375e87))
- *(interchain-token-service)* Update error handling for token id config ([#121](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/121)) - ([8430b93](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/8430b939fe3a330161ee7318218a85ee6721254d))
- *(interchain-token-service)* Revert on execute error ([#116](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/116)) - ([91533fc](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/91533fc4ff61aa3d7a3fc005cc52d8efbaf1f2ad))
- *(its)* Move ITS test directory to the correct level ([#86](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/86)) - ([6639369](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/6639369c916e2b839faa2227fc783e49704cb926))

### 🚜 Refactor

- *(interchain-token-service)* Use IntoEvent derive macro ([#143](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/143)) - ([5800cf7](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/5800cf7bed4a7efefc7bebd037635968eff3ee99))
- *(interchain-token-service)* Change trusted address to trusted chain ([#110](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/110)) - ([8798898](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/8798898467d5c4e2d3e0de071386b3ca6944a53a))
- *(its)* Add BytesExt trait ([#108](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/108)) - ([0b2d38a](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/0b2d38ab5cc8895e6038113db2e1f391e555b4fd))
- Update mock auth macro to support non root auth  ([#134](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/134)) - ([7b6a553](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/7b6a55385fc0bdcbd7d6bf065ddaa0f81dceb51f))
- Rename assert_auth macros ([#138](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/138)) - ([8239e41](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/8239e4126cdccb4156f737dd6e20fad5c2bfc239))
- [**breaking**] Update package name and references for release ([#145](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/145)) - ([bb19538](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/bb195386eeda9c75d4da33eb0cf29fd9cb9b621c))
- Restrict exports to contract and contract clients ([#103](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/103)) - ([4c25023](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/4c250237afce95fcd687f74e350b6b272a3d295d))
- Extract ownership management into sharable interface ([#97](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/97)) - ([df2d7d8](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/df2d7d8106e26c143757d26dfc321ffd5778d23b))
- Move shared interfaces in preparation of ownership trait extraction ([#96](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/96)) - ([e63006a](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/e63006a4f17abccbd1922389f1c03cc1735220b3))

### 📚 Documentation

- *(interchain-token-service)* Move entrypoint docstring to interface ([#146](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/146)) - ([f9ec5a6](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/f9ec5a66e8eb351ddea77fa23a48f57c7888adcf))

### 🧪 Testing

- Add expected invoke auth error for unit tests ([#52](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/52)) - ([890bfcf](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/890bfcfc92badf0ffed2c90aa581efdac4ce81dc))

### ⚙️ Miscellaneous Tasks

- *(its)* Move test.rs to tests folder ([#76](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/76)) - ([193c6ba](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/193c6bab049b90d80f80bc17de4d75ddd2d517bb))
- Use rustfmt nightly build to introduce opinionated imports ordering ([#141](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/141)) - ([e19f588](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/e19f5887dcb7f648d1aacb0fedbd6dfa9bf45eb2))
- Simplify auth test utils ([#125](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/125)) - ([8eba319](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/8eba3199c09180f7db446ddcc25580ad935fbfcc))
- Switch rust actions in workflows ([#107](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/107)) - ([480cb04](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/480cb04f311786545669a6f39a8a3a55950245e7))
- Add the support for release pipeline ([#54](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/54)) - ([90d4368](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/90d436811258b54ee8efbac074da515e977eb47e))
- Rename dev_dependencies to dev-dependencies ([#61](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/61)) - ([47c6576](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/47c657656cf83105c46b64b98d85c0653212d528))
- Remove `axelar-soroban-interfaces` crate ([#46](https://github.com/axelarnetwork/axelar-cgp-stellar/pull/46)) - ([514d8a4](https://github.com/axelarnetwork/axelar-cgp-stellar/commit/514d8a441ab30587dd953004894596147298fec7))

### Contributors

* @ahramy
* @nbayindirli
* @AttissNgo
* @TanvirDeol
* @cgorenflo
* @milapsheth
* @hydrobeam
* @apolikamixitos
