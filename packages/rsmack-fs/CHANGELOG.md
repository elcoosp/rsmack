# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [1.0.0] - 2026-11-14

### Bug Fixes
- readd deps for doc tests by @elcoosp ([c1c180c](https://github.com/elcoosp/rsmack/commit/c1c180cd5ed4b5b28784a4fbe7a1603f822e7484))
- fix invalid documentation by @elcoosp ([05ac0c7](https://github.com/elcoosp/rsmack/commit/05ac0c78150ffdefd9e37a1c165b178e0f71c164))
- version in cargo.toml by @elcoosp ([59c44b6](https://github.com/elcoosp/rsmack/commit/59c44b6b26f904ae138098a303917de48ead5be8))
- mising version on dep by @elcoosp ([7485bd3](https://github.com/elcoosp/rsmack/commit/7485bd32e805f5c7cbf60ba49821eb81a00ec2ee))
- mising descriptio & license by @elcoosp ([0f0455e](https://github.com/elcoosp/rsmack/commit/0f0455eead20df94c53eb2e12cdb838224ee53fe))
- update rust crate darling to 0.21.0 by @renovate[bot] in [#10](https://github.com/elcoosp/rsmack/pull/10) ([213bd28](https://github.com/elcoosp/rsmack/commit/213bd287dd020c76894680e00220e395e68f29ec))
- version in cargo.toml by @elcoosp ([bbe5b6c](https://github.com/elcoosp/rsmack/commit/bbe5b6ce2fbb27fb71b68ab215210464172020df))
- run clippy by @elcoosp ([8cbd0f0](https://github.com/elcoosp/rsmack/commit/8cbd0f0786fa7caed3b638f05fdddf41c8e7a8fd))
- up cargo.toml v by @elcoosp ([725876d](https://github.com/elcoosp/rsmack/commit/725876dbee38eec564780dad7a144f3f88d7f46a))
- up cargo.toml version by @elcoosp ([3b12c3f](https://github.com/elcoosp/rsmack/commit/3b12c3f590f21689b5e4c2792660cfcbc3ba3bb0))
- incorrect path fs mod by @elcoosp ([1c18c5c](https://github.com/elcoosp/rsmack/commit/1c18c5c0dd0c151b34cfcce696ac1326f060f67e))

### CI
- test then release by @elcoosp ([5980aa0](https://github.com/elcoosp/rsmack/commit/5980aa05439ca40f2392d397b27a34b7ca55e22f))

### Documentation
- add Vercel documentation link to Cargo.toml by @elcoosp ([0c60871](https://github.com/elcoosp/rsmack/commit/0c60871a81d8f94ee14005b098499c8d2631d321))
- author standalone folder mirroring documentation reference by @elcoosp ([16b467b](https://github.com/elcoosp/rsmack/commit/16b467b4bfb0bbfa98055042021a32eee4c44c12))
- rewrite architectural installation tour for v2 protocol by @elcoosp ([3a201a3](https://github.com/elcoosp/rsmack/commit/3a201a3229a0f1c9d688fb60c58eee3d7f1bc081))
- refresh README with new features, benchmarks, and error style guide by @elcoosp ([1b35a83](https://github.com/elcoosp/rsmack/commit/1b35a830fd84a5558450ae2bdca50d3a3ca7c93d))
- completely rewrite README with logo, badges, and improved structure by @elcoosp ([6d622e7](https://github.com/elcoosp/rsmack/commit/6d622e730e9fa2abae8d0a6ae49848e7054cb9d6))
- fix doc versions by @elcoosp ([3233495](https://github.com/elcoosp/rsmack/commit/32334957965c626e2ea91a1237346a1c9599f9fc))
- add seanum doc by @elcoosp ([d85aec4](https://github.com/elcoosp/rsmack/commit/d85aec413fcb6362515b2ecde7d87a701206ccaa))
- megamac readme fix incorrect doc by @elcoosp ([2ebadbc](https://github.com/elcoosp/rsmack/commit/2ebadbcf68ee50addca6d53ba0a7854d77e68697))
- improve megamac readme by @elcoosp ([3f294c7](https://github.com/elcoosp/rsmack/commit/3f294c73dea58eaadd64a3acf39a4040723cb959))
- fix readme install by @elcoosp ([a77f0d9](https://github.com/elcoosp/rsmack/commit/a77f0d9a7307551a7bae34d8b53d064f77be0742))
- add README by @elcoosp ([32456f3](https://github.com/elcoosp/rsmack/commit/32456f341e25b3e3bab04208e114ac7a54ecc694))
- add impls doc by @elcoosp ([4ebfa19](https://github.com/elcoosp/rsmack/commit/4ebfa1914950e28c6eda53433dc6cbc59235940f))

### Features
- rewrite folder iso struct for inline expansion-time emission by @elcoosp ([9fc5c9e](https://github.com/elcoosp/rsmack/commit/9fc5c9e883021992e87aec67db16ba948d9f9419))
- add recursive folder mirroring and visibility control by @elcoosp ([c4b6f03](https://github.com/elcoosp/rsmack/commit/c4b6f03570c3a22b618eeca1ece279e7472e1b1f))
- #[doc = include_str!(../README.md)] for each macro by @elcoosp ([5e2f8c6](https://github.com/elcoosp/rsmack/commit/5e2f8c695f52344642a6eaf74fff0b25614efd56))
- add schemars::impl_json_schema by @elcoosp ([82ed388](https://github.com/elcoosp/rsmack/commit/82ed388fa729cb71bceb6442de60afccea0e0b1e))
- stable base derive support by @elcoosp ([1bf4675](https://github.com/elcoosp/rsmack/commit/1bf4675521fd90d475de1e47c632ff24e57349a4))
- split get_macro_impl_file_ast & auto gen macro doc with megamac by @elcoosp ([cd0fcbb](https://github.com/elcoosp/rsmack/commit/cd0fcbb050c0784c3e3dd46ec67ae1e305a848da))
- add way to get the call site source file via unstable, pass env struct to exec by @elcoosp ([c7c80f6](https://github.com/elcoosp/rsmack/commit/c7c80f69ce2c127b31f6d2e2012d52aee072651c))
- add megamac crate to remove remove proc-macro decl boilerplate by @elcoosp ([f742b34](https://github.com/elcoosp/rsmack/commit/f742b34427685ea9b5fb9327c95629fdf1705323))
- complement logr, use own file for exec & use proc-macro-error2 by @elcoosp ([ca09d79](https://github.com/elcoosp/rsmack/commit/ca09d7989eb60fe726156963d29639726f3f168a))
- better error handling for impls using exec via Logr struct by @elcoosp ([b26cb92](https://github.com/elcoosp/rsmack/commit/b26cb92dc0b33cfe5a59b062c5dacf29fd3d8fed))
- folder_iso_struct builder proc_macro do nothing for now by @elcoosp ([112998c](https://github.com/elcoosp/rsmack/commit/112998ce7687212ddad49b1626820c1e3b430cd7))

### Miscellaneous
- inject missing metadata and point to root README by @elcoosp ([8b88046](https://github.com/elcoosp/rsmack/commit/8b8804660571e1475cf6f94a0db8bad425523201))
- inject precisely curated keywords for crates.io by @elcoosp ([b3b79e8](https://github.com/elcoosp/rsmack/commit/b3b79e8763d28348b81c6ba285cf8aa9841d7fbf))
- 0.17.0 by @elcoosp ([a4a8d3b](https://github.com/elcoosp/rsmack/commit/a4a8d3b0be71c0822baae42f8ed46e186ac2e24c))
- 0.16.0 by @elcoosp ([7df3982](https://github.com/elcoosp/rsmack/commit/7df39828d2d98876275bfde91ad46d08c380178b))
- v0.15.0 by @elcoosp ([90758ae](https://github.com/elcoosp/rsmack/commit/90758ae5cfa4b1bd66f7af00650dd0254d5bd469))
- 0.14.0 by @elcoosp ([fb54b73](https://github.com/elcoosp/rsmack/commit/fb54b73cc8bd9b23e5d6f425e1483d19407f0305))
- 0.13.0 by @elcoosp ([0c56d5c](https://github.com/elcoosp/rsmack/commit/0c56d5c4194ee3861b9c54792715dc19e47e1f13))
- v0.3.0 by @elcoosp ([20696ab](https://github.com/elcoosp/rsmack/commit/20696ab5fc50e8ff141acfd168a6d2fd499fc095))
- run clippy pedantic by @elcoosp ([99ff717](https://github.com/elcoosp/rsmack/commit/99ff71755d68f709c392dbdec10c5955e99dddb2))

### Refactor
- enrich crate root documentation with inline usages by @elcoosp ([043edbe](https://github.com/elcoosp/rsmack/commit/043edbeb22a11d908272f4fcc5de93f9be5f0e56))
- update megamac invocation to use readme docs by @elcoosp ([a12f2e2](https://github.com/elcoosp/rsmack/commit/a12f2e2db0aa4f9b49ae8d8cefe4d5173442531a))
- clean up macro declaration and imports by @elcoosp ([15ea0b6](https://github.com/elcoosp/rsmack/commit/15ea0b6f1876d90dda0e834694160ac0bbe17556))
- improve error handling and accept string paths by @elcoosp ([5e5c1ea](https://github.com/elcoosp/rsmack/commit/5e5c1ea8fd845005ac1d4b821bb980af112ec11f))

### Testing
- adapt integration suite for inline folder scanning assertions by @elcoosp ([32741c1](https://github.com/elcoosp/rsmack/commit/32741c1587bcd7d0a0c803793c8e4a5d9266439a))
- seed module fixture elements for folders mirror tests by @elcoosp ([c60b008](https://github.com/elcoosp/rsmack/commit/c60b0082127df3f48d16b9fce9ea5be51d327ed6))
- seed module fixture elements for folders mirror tests by @elcoosp ([19bd165](https://github.com/elcoosp/rsmack/commit/19bd165994513be2b35b680f38cabf19528d9806))
- seed module fixture elements for folders mirror tests by @elcoosp ([5f5a90b](https://github.com/elcoosp/rsmack/commit/5f5a90b00938a097781989b497641a2dd536c2d4))
- add integration test for folder_iso_struct macro by @elcoosp ([7d54c7e](https://github.com/elcoosp/rsmack/commit/7d54c7ed98669c10093090285220de0b9977532c))

### build
- enable syn full feature for parsing by @elcoosp ([a6ce674](https://github.com/elcoosp/rsmack/commit/a6ce674e9a1a0177ae1d3817986083dbb906a3da))
- migrate to workspace dependencies and remove run_script by @elcoosp ([5ee077c](https://github.com/elcoosp/rsmack/commit/5ee077c5414be07af1d41328dd331e3d176309ea))
**Full Changelog**: https://github.com/elcoosp/rsmack/releases/tag/v1.0.0
