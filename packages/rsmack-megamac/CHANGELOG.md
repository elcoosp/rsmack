# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [1.0.0] - 2026-11-14

### Bug Fixes
- readd deps for doc tests by @elcoosp ([c1c180c](https://github.com/elcoosp/rsmack/commit/c1c180cd5ed4b5b28784a4fbe7a1603f822e7484))
- fix invalid documentation by @elcoosp ([05ac0c7](https://github.com/elcoosp/rsmack/commit/05ac0c78150ffdefd9e37a1c165b178e0f71c164))
- update rust crate strum to 0.27.0 by @renovate[bot] in [#12](https://github.com/elcoosp/rsmack/pull/12) ([96a05ed](https://github.com/elcoosp/rsmack/commit/96a05ed7a149db5adeb2ac169e11f535af452096))
- update rust crate stringcase to 0.4 by @renovate[bot] in [#11](https://github.com/elcoosp/rsmack/pull/11) ([660c141](https://github.com/elcoosp/rsmack/commit/660c141ddfbe48a8fdcde178bffbd33b42d5a47d))
- version in cargo.toml by @elcoosp ([59c44b6](https://github.com/elcoosp/rsmack/commit/59c44b6b26f904ae138098a303917de48ead5be8))
- mising descriptio & license by @elcoosp ([0f0455e](https://github.com/elcoosp/rsmack/commit/0f0455eead20df94c53eb2e12cdb838224ee53fe))
- update rust crate darling to 0.21.0 by @renovate[bot] in [#10](https://github.com/elcoosp/rsmack/pull/10) ([213bd28](https://github.com/elcoosp/rsmack/commit/213bd287dd020c76894680e00220e395e68f29ec))
- version in cargo.toml by @elcoosp ([bbe5b6c](https://github.com/elcoosp/rsmack/commit/bbe5b6ce2fbb27fb71b68ab215210464172020df))
- run clippy display fix by @elcoosp ([400bbd3](https://github.com/elcoosp/rsmack/commit/400bbd3b5279f13bf84f27d8b6ea058c8857aade))
- run clippy by @elcoosp ([8cbd0f0](https://github.com/elcoosp/rsmack/commit/8cbd0f0786fa7caed3b638f05fdddf41c8e7a8fd))
- up cargo.toml v by @elcoosp ([725876d](https://github.com/elcoosp/rsmack/commit/725876dbee38eec564780dad7a144f3f88d7f46a))
- up cargo.toml version by @elcoosp ([3b12c3f](https://github.com/elcoosp/rsmack/commit/3b12c3f590f21689b5e4c2792660cfcbc3ba3bb0))
- fix all clippy warns by @elcoosp ([f6b7db0](https://github.com/elcoosp/rsmack/commit/f6b7db0e6f428c585fdc412ac25d04323119b910))
- run clippy cf logr change by @elcoosp ([3d8aac3](https://github.com/elcoosp/rsmack/commit/3d8aac3d65c49d17c3944f1569f607487e53aaef))
- some clippy lints by @elcoosp ([5628444](https://github.com/elcoosp/rsmack/commit/56284440525635c7f72478f8b3eed27720ec7f87))
- revert use of procmacro2_semver_exempt to get sourcefile instead derive package_src_folder from std::env::var (not env! because it is compile time) by @elcoosp ([e855e79](https://github.com/elcoosp/rsmack/commit/e855e79f4846b3ae226de5e58409aef34cd97e7e))

### CI
- test then release by @elcoosp ([5980aa0](https://github.com/elcoosp/rsmack/commit/5980aa05439ca40f2392d397b27a34b7ca55e22f))

### Documentation
- add Vercel documentation link to Cargo.toml by @elcoosp ([0c60871](https://github.com/elcoosp/rsmack/commit/0c60871a81d8f94ee14005b098499c8d2631d321))
- enforce SKILL.md rules (jsDelivr logos, absolute links, remove license sections) by @elcoosp ([3f7e388](https://github.com/elcoosp/rsmack/commit/3f7e38879172dceea5f1402c17156f3dc0e7d129))
- rewrite framework usage around signature-driven protocols by @elcoosp ([3c3d70a](https://github.com/elcoosp/rsmack/commit/3c3d70ab45a6077f08c333b0682e94a6f3596b58))
- rewrite architectural installation tour for v2 protocol by @elcoosp ([3a201a3](https://github.com/elcoosp/rsmack/commit/3a201a3229a0f1c9d688fb60c58eee3d7f1bc081))
- refresh README with new features, benchmarks, and error style guide by @elcoosp ([1b35a83](https://github.com/elcoosp/rsmack/commit/1b35a830fd84a5558450ae2bdca50d3a3ca7c93d))
- add comprehensive rustdoc to the megamac entrypoint by @elcoosp ([55ba968](https://github.com/elcoosp/rsmack/commit/55ba968c2254578a490253eeabe6ac3daf3097e0))
- add dedicated README for the megamac crate by @elcoosp ([30cc3ba](https://github.com/elcoosp/rsmack/commit/30cc3ba3cb7072fa71ebb6b1cc55b8632958e62b))
- completely rewrite README with logo, badges, and improved structure by @elcoosp ([6d622e7](https://github.com/elcoosp/rsmack/commit/6d622e730e9fa2abae8d0a6ae49848e7054cb9d6))
- fix doc versions by @elcoosp ([3233495](https://github.com/elcoosp/rsmack/commit/32334957965c626e2ea91a1237346a1c9599f9fc))
- add seanum doc by @elcoosp ([d85aec4](https://github.com/elcoosp/rsmack/commit/d85aec413fcb6362515b2ecde7d87a701206ccaa))
- megamac readme fix incorrect doc by @elcoosp ([2ebadbc](https://github.com/elcoosp/rsmack/commit/2ebadbcf68ee50addca6d53ba0a7854d77e68697))
- improve megamac readme by @elcoosp ([3f294c7](https://github.com/elcoosp/rsmack/commit/3f294c73dea58eaadd64a3acf39a4040723cb959))
- fix readme install by @elcoosp ([a77f0d9](https://github.com/elcoosp/rsmack/commit/a77f0d9a7307551a7bae34d8b53d064f77be0742))
- add README by @elcoosp ([32456f3](https://github.com/elcoosp/rsmack/commit/32456f341e25b3e3bab04208e114ac7a54ecc694))
- add impls doc by @elcoosp ([4ebfa19](https://github.com/elcoosp/rsmack/commit/4ebfa1914950e28c6eda53433dc6cbc59235940f))
- enhance by @elcoosp ([b9c3a19](https://github.com/elcoosp/rsmack/commit/b9c3a19975d6af085f7c6bb26beaad0e04f6488e))

### Features
- implement process-global cache and raw payload signatures by @elcoosp ([d78a9c1](https://github.com/elcoosp/rsmack/commit/d78a9c1238e1ed8d4dba4d5eda87a24c489a85f2))
- support custom impls/args idents, inline docs, and inferred receivers by @elcoosp ([c35ad7a](https://github.com/elcoosp/rsmack/commit/c35ad7aeded10ed1136864d515dc47df44ae3f1e))
- add CLI playground for macro expansion preview by @elcoosp ([a9b1c7c](https://github.com/elcoosp/rsmack/commit/a9b1c7c0ce04cc63af141f4e87e99582e9c587e1))
- #[doc = include_str!(../README.md)] for each macro by @elcoosp ([5e2f8c6](https://github.com/elcoosp/rsmack/commit/5e2f8c695f52344642a6eaf74fff0b25614efd56))
- add schemars::impl_json_schema by @elcoosp ([82ed388](https://github.com/elcoosp/rsmack/commit/82ed388fa729cb71bceb6442de60afccea0e0b1e))
- stable base derive support by @elcoosp ([1bf4675](https://github.com/elcoosp/rsmack/commit/1bf4675521fd90d475de1e47c632ff24e57349a4))
- split get_macro_impl_file_ast & auto gen macro doc with megamac by @elcoosp ([cd0fcbb](https://github.com/elcoosp/rsmack/commit/cd0fcbb050c0784c3e3dd46ec67ae1e305a848da))
- split get_args_fields_doc by @elcoosp ([06c89f8](https://github.com/elcoosp/rsmack/commit/06c89f82a45e188bc3d90a6555eb7a23c6f365ac))
- add way to get the call site source file via unstable, pass env struct to exec by @elcoosp ([c7c80f6](https://github.com/elcoosp/rsmack/commit/c7c80f69ce2c127b31f6d2e2012d52aee072651c))
- add megamac crate to remove remove proc-macro decl boilerplate by @elcoosp ([f742b34](https://github.com/elcoosp/rsmack/commit/f742b34427685ea9b5fb9327c95629fdf1705323))

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
- comment unused by @elcoosp ([c16465e](https://github.com/elcoosp/rsmack/commit/c16465e6f165332cbcbe4cb55652a9324e1906b0))

### Performance
- adapt generation benches for unallocated env parameters by @elcoosp ([16e773d](https://github.com/elcoosp/rsmack/commit/16e773d25a0c7dc1f8a9646bc380b7b76aa600ea))
- add criterion expansion benchmarks by @elcoosp ([b905a6f](https://github.com/elcoosp/rsmack/commit/b905a6f88375a6f21a024207ff8eaef0c6f6c599))

### Refactor
- enrich crate facade mapping specifications by @elcoosp ([446aef8](https://github.com/elcoosp/rsmack/commit/446aef8ebbd2b8abfb811a0697b142d7798b4e60))
- update playground runner for unallocated environments by @elcoosp ([0fc74e0](https://github.com/elcoosp/rsmack/commit/0fc74e05f9bbf43d52ef5e63ec4041d7915d1b1c))
- route through shared exec driver and update docs by @elcoosp ([6d68b6d](https://github.com/elcoosp/rsmack/commit/6d68b6d876c83174e3e4d0e5017515ac7577daba))
- replace panics with syn::Error and fix doc extraction by @elcoosp ([b29ae49](https://github.com/elcoosp/rsmack/commit/b29ae49a8dfc38824d4f731cbbaf5343de799c72))

### Testing
- add integration tests for playground binary by @elcoosp ([b341608](https://github.com/elcoosp/rsmack/commit/b3416083a041436bdcf680bf37989aac48a4671b))

### build
- add criterion dev-dependency and bench target by @elcoosp ([50ef399](https://github.com/elcoosp/rsmack/commit/50ef39934fcd40934bf3a219d02badb42c44dc1f))
- migrate to workspace dependencies by @elcoosp ([e7c1211](https://github.com/elcoosp/rsmack/commit/e7c121147df3f7409139a6649782944f42b79096))
**Full Changelog**: https://github.com/elcoosp/rsmack/releases/tag/v1.0.0
