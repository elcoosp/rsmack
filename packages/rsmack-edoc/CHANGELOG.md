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
- fix all clippy warns by @elcoosp ([f6b7db0](https://github.com/elcoosp/rsmack/commit/f6b7db0e6f428c585fdc412ac25d04323119b910))
- unused vars by @elcoosp ([191379c](https://github.com/elcoosp/rsmack/commit/191379cd39d8f4a3d03ca751c124b95c9dcb24ac))
- run clippy cf logr change by @elcoosp ([3d8aac3](https://github.com/elcoosp/rsmack/commit/3d8aac3d65c49d17c3944f1569f607487e53aaef))

### CI
- test then release by @elcoosp ([5980aa0](https://github.com/elcoosp/rsmack/commit/5980aa05439ca40f2392d397b27a34b7ca55e22f))

### Documentation
- add Vercel documentation link to Cargo.toml by @elcoosp ([0c60871](https://github.com/elcoosp/rsmack/commit/0c60871a81d8f94ee14005b098499c8d2631d321))
- author standalone crate documentation reference by @elcoosp ([fe00750](https://github.com/elcoosp/rsmack/commit/fe0075092d75d148b6354dd13c7ddf336de575b7))
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
- enhance by @elcoosp ([b9c3a19](https://github.com/elcoosp/rsmack/commit/b9c3a19975d6af085f7c6bb26beaad0e04f6488e))

### Features
- implement item-level and enum variant edoc attributes by @elcoosp ([709b8e7](https://github.com/elcoosp/rsmack/commit/709b8e7819f777f025ce2fcc8411c3653a342ff8))
- build typed ConstValue interpreter and path matches by @elcoosp ([7c1cf16](https://github.com/elcoosp/rsmack/commit/7c1cf166361454732e61ac8605e08247d736bc55))
- support const-eval, multi-error reporting, and memoized consts cache by @elcoosp ([3e34004](https://github.com/elcoosp/rsmack/commit/3e34004db19cebc66caec82d7fcffa599f27419b))
- implement mini const-expression evaluator by @elcoosp ([dbc344d](https://github.com/elcoosp/rsmack/commit/dbc344df5b65ba35e07cf30cab65cc1d1550cdef))
- #[doc = include_str!(../README.md)] for each macro by @elcoosp ([5e2f8c6](https://github.com/elcoosp/rsmack/commit/5e2f8c695f52344642a6eaf74fff0b25614efd56))
- add schemars::impl_json_schema by @elcoosp ([82ed388](https://github.com/elcoosp/rsmack/commit/82ed388fa729cb71bceb6442de60afccea0e0b1e))
- use cl macros in logr by @elcoosp ([d49d3ba](https://github.com/elcoosp/rsmack/commit/d49d3ba09bd9cc4e5745c320024095ef1b34b05c))
- working edoc in its own crate non derive but use DeriveInput to collect field attrs under the hood by @elcoosp ([7484700](https://github.com/elcoosp/rsmack/commit/7484700009288700b654730957338a306aad36bf))

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
- adapt criterion benches for unallocated env signatures by @elcoosp ([e6a1c74](https://github.com/elcoosp/rsmack/commit/e6a1c7409c6c53c85ec2ccf31e1807ea86a26a40))
- add criterion expansion benchmarks by @elcoosp ([bbb0fc8](https://github.com/elcoosp/rsmack/commit/bbb0fc84cbcfd0283c254319465159fa7af59bf9))

### Refactor
- enrich crate facade programmatic documentation by @elcoosp ([2e360c7](https://github.com/elcoosp/rsmack/commit/2e360c7a3ed8d2be0e9d343262e80df396ca0c99))
- update megamac invocation to use inline docs by @elcoosp ([b4b9a28](https://github.com/elcoosp/rsmack/commit/b4b9a28c123fee370f9aaca9d336ec1592a7b948))
- expose const_eval module by @elcoosp ([3cf4a14](https://github.com/elcoosp/rsmack/commit/3cf4a14fdb2e95904ff7f5e3a581a7e06b9e4aa2))
- clean up macro declaration and imports by @elcoosp ([681d787](https://github.com/elcoosp/rsmack/commit/681d787955e65c7a295aec63ddf41f371737bddc))
- parse constants file as AST and improve error handling by @elcoosp ([5ad884d](https://github.com/elcoosp/rsmack/commit/5ad884d80cd1fb8c31626fe34d04959c137139d2))

### Testing
- add impls module mirror for benchmarks by @elcoosp ([e032eaf](https://github.com/elcoosp/rsmack/commit/e032eaf2f6844640d6cffea4e09f19f0265ff88a))
- add end-to-end integration tests for the edoc macro by @elcoosp ([68a7335](https://github.com/elcoosp/rsmack/commit/68a73357f5b79af3acac0dc8a16095982dcf33e4))
- add dummy constants file for integration tests by @elcoosp ([ca954cb](https://github.com/elcoosp/rsmack/commit/ca954cb56632c99a82f99ffb4c276f8d90cd6fd5))

### build
- add criterion dev-dependency and bench target by @elcoosp ([7eef2fb](https://github.com/elcoosp/rsmack/commit/7eef2fb743eb6513acab9686567066ebc9ffdc11))
- migrate to workspace dependencies and edition 2024 by @elcoosp ([3bccaf8](https://github.com/elcoosp/rsmack/commit/3bccaf8a4e798605f1827f80cd3b180418239f18))
**Full Changelog**: https://github.com/elcoosp/rsmack/releases/tag/v1.0.0
