# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [1.0.0] - 2026-11-14

### Bug Fixes
- readd deps for doc tests by @elcoosp ([c1c180c](https://github.com/elcoosp/rsmack/commit/c1c180cd5ed4b5b28784a4fbe7a1603f822e7484))
- fix invalid documentation by @elcoosp ([05ac0c7](https://github.com/elcoosp/rsmack/commit/05ac0c78150ffdefd9e37a1c165b178e0f71c164))
- version in cargo.toml by @elcoosp ([59c44b6](https://github.com/elcoosp/rsmack/commit/59c44b6b26f904ae138098a303917de48ead5be8))
- license field by @elcoosp ([dd58fcd](https://github.com/elcoosp/rsmack/commit/dd58fcda80ffbfd2d8af34ffc57b9bf9954b68bd))
- mising version on dep by @elcoosp ([7485bd3](https://github.com/elcoosp/rsmack/commit/7485bd32e805f5c7cbf60ba49821eb81a00ec2ee))
- mising descriptio & license by @elcoosp ([0f0455e](https://github.com/elcoosp/rsmack/commit/0f0455eead20df94c53eb2e12cdb838224ee53fe))
- update rust crate darling to 0.21.0 by @renovate[bot] in [#10](https://github.com/elcoosp/rsmack/pull/10) ([213bd28](https://github.com/elcoosp/rsmack/commit/213bd287dd020c76894680e00220e395e68f29ec))
- version in cargo.toml by @elcoosp ([bbe5b6c](https://github.com/elcoosp/rsmack/commit/bbe5b6ce2fbb27fb71b68ab215210464172020df))
- up cargo.toml v by @elcoosp ([725876d](https://github.com/elcoosp/rsmack/commit/725876dbee38eec564780dad7a144f3f88d7f46a))
- up cargo.toml version by @elcoosp ([3b12c3f](https://github.com/elcoosp/rsmack/commit/3b12c3f590f21689b5e4c2792660cfcbc3ba3bb0))
- run clippy cf logr change by @elcoosp ([3d8aac3](https://github.com/elcoosp/rsmack/commit/3d8aac3d65c49d17c3944f1569f607487e53aaef))
- rename vars by @elcoosp ([8ff89ed](https://github.com/elcoosp/rsmack/commit/8ff89edf6a657f582595182b0ee33aa3e1785c7d))

### CI
- test then release by @elcoosp ([5980aa0](https://github.com/elcoosp/rsmack/commit/5980aa05439ca40f2392d397b27a34b7ca55e22f))

### Documentation
- add Vercel documentation link to Cargo.toml by @elcoosp ([0c60871](https://github.com/elcoosp/rsmack/commit/0c60871a81d8f94ee14005b098499c8d2631d321))
- rewrite documentation reference for universal wrappers by @elcoosp ([4e164eb](https://github.com/elcoosp/rsmack/commit/4e164ebfb2685b6b3a689e287c1e73d822a4d0e4))
- rewrite architectural installation tour for v2 protocol by @elcoosp ([3a201a3](https://github.com/elcoosp/rsmack/commit/3a201a3229a0f1c9d688fb60c58eee3d7f1bc081))
- refresh README with new features, benchmarks, and error style guide by @elcoosp ([1b35a83](https://github.com/elcoosp/rsmack/commit/1b35a830fd84a5558450ae2bdca50d3a3ca7c93d))
- completely rewrite README with logo, badges, and improved structure by @elcoosp ([6d622e7](https://github.com/elcoosp/rsmack/commit/6d622e730e9fa2abae8d0a6ae49848e7054cb9d6))
- fix doc versions by @elcoosp ([3233495](https://github.com/elcoosp/rsmack/commit/32334957965c626e2ea91a1237346a1c9599f9fc))
- add seanum doc by @elcoosp ([d85aec4](https://github.com/elcoosp/rsmack/commit/d85aec413fcb6362515b2ecde7d87a701206ccaa))
- megamac readme fix incorrect doc by @elcoosp ([2ebadbc](https://github.com/elcoosp/rsmack/commit/2ebadbcf68ee50addca6d53ba0a7854d77e68697))
- improve megamac readme by @elcoosp ([3f294c7](https://github.com/elcoosp/rsmack/commit/3f294c73dea58eaadd64a3acf39a4040723cb959))
- fix readme install by @elcoosp ([a77f0d9](https://github.com/elcoosp/rsmack/commit/a77f0d9a7307551a7bae34d8b53d064f77be0742))
- add README by @elcoosp ([32456f3](https://github.com/elcoosp/rsmack/commit/32456f341e25b3e3bab04208e114ac7a54ecc694))

### Features
- add per-field skip, with, and map attributes by @elcoosp ([5ef2151](https://github.com/elcoosp/rsmack/commit/5ef2151f607be33a36821a8aa7fd8e562fac1443))
- #[doc = include_str!(../README.md)] for each macro by @elcoosp ([5e2f8c6](https://github.com/elcoosp/rsmack/commit/5e2f8c695f52344642a6eaf74fff0b25614efd56))
- add schemars::impl_json_schema by @elcoosp ([82ed388](https://github.com/elcoosp/rsmack/commit/82ed388fa729cb71bceb6442de60afccea0e0b1e))
- working edoc in its own crate non derive but use DeriveInput to collect field attrs under the hood by @elcoosp ([7484700](https://github.com/elcoosp/rsmack/commit/7484700009288700b654730957338a306aad36bf))
- stable base derive support by @elcoosp ([1bf4675](https://github.com/elcoosp/rsmack/commit/1bf4675521fd90d475de1e47c632ff24e57349a4))
- split get_macro_impl_file_ast & auto gen macro doc with megamac by @elcoosp ([cd0fcbb](https://github.com/elcoosp/rsmack/commit/cd0fcbb050c0784c3e3dd46ec67ae1e305a848da))
- split get_args_fields_doc by @elcoosp ([06c89f8](https://github.com/elcoosp/rsmack/commit/06c89f82a45e188bc3d90a6555eb7a23c6f365ac))
- add way to get the call site source file via unstable, pass env struct to exec by @elcoosp ([c7c80f6](https://github.com/elcoosp/rsmack/commit/c7c80f69ce2c127b31f6d2e2012d52aee072651c))
- add megamac crate to remove remove proc-macro decl boilerplate by @elcoosp ([f742b34](https://github.com/elcoosp/rsmack/commit/f742b34427685ea9b5fb9327c95629fdf1705323))
- complement logr, use own file for exec & use proc-macro-error2 by @elcoosp ([ca09d79](https://github.com/elcoosp/rsmack/commit/ca09d7989eb60fe726156963d29639726f3f168a))
- better error handling for impls using exec via Logr struct by @elcoosp ([b26cb92](https://github.com/elcoosp/rsmack/commit/b26cb92dc0b33cfe5a59b062c5dacf29fd3d8fed))
- create rsmack-utils to reuse call_impls_with_args macro_export by @elcoosp ([2368165](https://github.com/elcoosp/rsmack/commit/236816553fedbd39eb0a6948f55b1696516df708))
- wrap macro to wrap all fields ty of a named struct in the given ty wrapper ident by @elcoosp ([e4a07c1](https://github.com/elcoosp/rsmack/commit/e4a07c1714669d4d247e1d82c0ff5179148977b6))

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

### Performance
- adapt field wrapping benches for unallocated signatures by @elcoosp ([1262512](https://github.com/elcoosp/rsmack/commit/1262512de4ca0ca720db62167bd5a8f92c4dc6fe))
- add criterion expansion benchmarks by @elcoosp ([c74bd05](https://github.com/elcoosp/rsmack/commit/c74bd051d46bf0e71c2bb216d434edc51f9a9b01))

### Refactor
- lift field type wrapping restrictions entirely by @elcoosp ([4558b5d](https://github.com/elcoosp/rsmack/commit/4558b5de183a33d79fd06f3698bbe57e46f2f622))
- update megamac invocation to use readme docs by @elcoosp ([c70cd5b](https://github.com/elcoosp/rsmack/commit/c70cd5bf2bdfbaee44040185f7789047ac15f519))
- clean up macro declaration and imports by @elcoosp ([598549b](https://github.com/elcoosp/rsmack/commit/598549bed498d90da43e374af2f4fb01b39a67af))
- improve error handling for unsupported field types by @elcoosp ([7724413](https://github.com/elcoosp/rsmack/commit/772441324e1c06f9d349fce8521fc80cc8469c97))

### Testing
- add integration tests for the wrap macro by @elcoosp ([608f181](https://github.com/elcoosp/rsmack/commit/608f1815076cddfcf026110548199ca148677cea))

### build
- enable syn full and extra-traits features by @elcoosp ([957224f](https://github.com/elcoosp/rsmack/commit/957224f75cf287f05db291c9eae4944615e474b6))
- migrate to workspace dependencies by @elcoosp ([e6d1b19](https://github.com/elcoosp/rsmack/commit/e6d1b194443682060c29a12002aada737eb8cba3))
**Full Changelog**: https://github.com/elcoosp/rsmack/releases/tag/v1.0.0
