# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [1.0.0] - 2026-11-11

### Bug Fixes
- readd deps for doc tests by @elcoosp ([c1c180c](https://github.com/elcoosp/rsmack/commit/c1c180cd5ed4b5b28784a4fbe7a1603f822e7484))
- update rust crate build-print to v1 by @renovate[bot] in [#15](https://github.com/elcoosp/rsmack/pull/15) ([f6c68a1](https://github.com/elcoosp/rsmack/commit/f6c68a19b132119c02dd6ddedf1bc3765efcc71f))
- update rust crate stringcase to 0.4 by @renovate[bot] in [#11](https://github.com/elcoosp/rsmack/pull/11) ([660c141](https://github.com/elcoosp/rsmack/commit/660c141ddfbe48a8fdcde178bffbd33b42d5a47d))
- version in cargo.toml by @elcoosp ([59c44b6](https://github.com/elcoosp/rsmack/commit/59c44b6b26f904ae138098a303917de48ead5be8))
- mising descriptio & license by @elcoosp ([0f0455e](https://github.com/elcoosp/rsmack/commit/0f0455eead20df94c53eb2e12cdb838224ee53fe))
- update rust crate darling to 0.21.0 by @renovate[bot] in [#10](https://github.com/elcoosp/rsmack/pull/10) ([213bd28](https://github.com/elcoosp/rsmack/commit/213bd287dd020c76894680e00220e395e68f29ec))
- version in cargo.toml by @elcoosp ([bbe5b6c](https://github.com/elcoosp/rsmack/commit/bbe5b6ce2fbb27fb71b68ab215210464172020df))
- run clippy by @elcoosp ([8cbd0f0](https://github.com/elcoosp/rsmack/commit/8cbd0f0786fa7caed3b638f05fdddf41c8e7a8fd))
- up cargo.toml v by @elcoosp ([725876d](https://github.com/elcoosp/rsmack/commit/725876dbee38eec564780dad7a144f3f88d7f46a))
- up cargo.toml version by @elcoosp ([3b12c3f](https://github.com/elcoosp/rsmack/commit/3b12c3f590f21689b5e4c2792660cfcbc3ba3bb0))
- split build_env macro by @elcoosp ([b4d0be5](https://github.com/elcoosp/rsmack/commit/b4d0be51799fd38c86447de70430eb4da0300df3))
- make logr take M: AsRef<str> by @elcoosp ([484e5a2](https://github.com/elcoosp/rsmack/commit/484e5a20631c3db65c5fe11ff2cf8a04f93a3ea2))
- some clippy lints by @elcoosp ([5628444](https://github.com/elcoosp/rsmack/commit/56284440525635c7f72478f8b3eed27720ec7f87))
- split fs mod by @elcoosp ([6c11672](https://github.com/elcoosp/rsmack/commit/6c11672da36befa1ee13af261a03c8db6aba4793))

### Documentation
- document raw payload entrypoint drivers facings by @elcoosp ([6688b4f](https://github.com/elcoosp/rsmack/commit/6688b4f943a2cdabbd1539bab4611108f9388d2f))
- add dedicated README for the utils crate by @elcoosp ([b7ec9e4](https://github.com/elcoosp/rsmack/commit/b7ec9e448d5ad9e420b3ee6328b0547b24d64b15))
- enhance by @elcoosp ([b9c3a19](https://github.com/elcoosp/rsmack/commit/b9c3a19975d6af085f7c6bb26beaad0e04f6488e))
- fs mod by @elcoosp ([443cd68](https://github.com/elcoosp/rsmack/commit/443cd68ab0c7e710166e3aa842e265d9b968e3af))

### Features
- export fallible module scanners and staleness keys by @elcoosp ([bc509cb](https://github.com/elcoosp/rsmack/commit/bc509cbefa2ae04054331330b51e5ca4970135ce))
- implement raw attribute and function macro entrypoint drivers by @elcoosp ([479b24f](https://github.com/elcoosp/rsmack/commit/479b24fa036718f03a360e8d0c5014cdcc78a1fc))
- support recursive folder scanning and visibility in folder_iso_struct by @elcoosp ([746b460](https://github.com/elcoosp/rsmack/commit/746b4600e53e505a2eb7f509a5b6e5f0a351317e))
- add finish method to ExecEnv for flushing Logr errors by @elcoosp ([a78cc2e](https://github.com/elcoosp/rsmack/commit/a78cc2e63e30603db901d4296ef7f5fae53077ab))
- support empty attribute/macro arguments via darling from_word by @elcoosp ([d46757f](https://github.com/elcoosp/rsmack/commit/d46757f2d8108c0686906bf9af7186baaefbe8e5))
- implement seanum, generate a Clone, Dummy, Debug, PartialEq, EnumIter, DeriveActiveEnum, Eq, Serialize, Deserialize enum wired with correct attrs mirroring the enum by @elcoosp ([2a20614](https://github.com/elcoosp/rsmack/commit/2a2061425bb58e7392fb251045a694ac584d9584))
- add schemars::impl_json_schema by @elcoosp ([82ed388](https://github.com/elcoosp/rsmack/commit/82ed388fa729cb71bceb6442de60afccea0e0b1e))
- use cl macros in logr by @elcoosp ([d49d3ba](https://github.com/elcoosp/rsmack/commit/d49d3ba09bd9cc4e5745c320024095ef1b34b05c))
- stable base derive support by @elcoosp ([1bf4675](https://github.com/elcoosp/rsmack/commit/1bf4675521fd90d475de1e47c632ff24e57349a4))
- add way to get the call site source file via unstable, pass env struct to exec by @elcoosp ([c7c80f6](https://github.com/elcoosp/rsmack/commit/c7c80f69ce2c127b31f6d2e2012d52aee072651c))
- add megamac crate to remove remove proc-macro decl boilerplate by @elcoosp ([f742b34](https://github.com/elcoosp/rsmack/commit/f742b34427685ea9b5fb9327c95629fdf1705323))
- complement logr, use own file for exec & use proc-macro-error2 by @elcoosp ([ca09d79](https://github.com/elcoosp/rsmack/commit/ca09d7989eb60fe726156963d29639726f3f168a))
- add Logr::new_module_path by @elcoosp ([849e111](https://github.com/elcoosp/rsmack/commit/849e1118aaac4e63a642957833f362b2c70ebba6))
- better error handling for impls using exec via Logr struct by @elcoosp ([b26cb92](https://github.com/elcoosp/rsmack/commit/b26cb92dc0b33cfe5a59b062c5dacf29fd3d8fed))
- folder_iso_struct builder proc_macro do nothing for now by @elcoosp ([112998c](https://github.com/elcoosp/rsmack/commit/112998ce7687212ddad49b1626820c1e3b430cd7))
- create rsmack-utils to reuse call_impls_with_args macro_export by @elcoosp ([2368165](https://github.com/elcoosp/rsmack/commit/236816553fedbd39eb0a6948f55b1696516df708))

### Miscellaneous
- 0.17.0 by @elcoosp ([a4a8d3b](https://github.com/elcoosp/rsmack/commit/a4a8d3b0be71c0822baae42f8ed46e186ac2e24c))
- 0.16.0 by @elcoosp ([7df3982](https://github.com/elcoosp/rsmack/commit/7df39828d2d98876275bfde91ad46d08c380178b))
- v0.15.0 by @elcoosp ([90758ae](https://github.com/elcoosp/rsmack/commit/90758ae5cfa4b1bd66f7af00650dd0254d5bd469))
- 0.14.0 by @elcoosp ([fb54b73](https://github.com/elcoosp/rsmack/commit/fb54b73cc8bd9b23e5d6f425e1483d19407f0305))
- 0.13.0 by @elcoosp ([0c56d5c](https://github.com/elcoosp/rsmack/commit/0c56d5c4194ee3861b9c54792715dc19e47e1f13))
- v0.3.0 by @elcoosp ([20696ab](https://github.com/elcoosp/rsmack/commit/20696ab5fc50e8ff141acfd168a6d2fd499fc095))
- rename by @elcoosp ([4a7a620](https://github.com/elcoosp/rsmack/commit/4a7a6204e4d8e22103393c4015767c45d4ef3074))
- run clippy pedantic by @elcoosp ([99ff717](https://github.com/elcoosp/rsmack/commit/99ff71755d68f709c392dbdec10c5955e99dddb2))

### Performance
- adapt utilities benches for unallocated builders by @elcoosp ([de712ff](https://github.com/elcoosp/rsmack/commit/de712ffab9bdd70b8a5949f72e5df50cbf13059b))
- add criterion benchmarks for Logr and exec drivers by @elcoosp ([59f4970](https://github.com/elcoosp/rsmack/commit/59f4970fd151c3d963281c562fa7ac72aa4c3783))

### Refactor
- drop builder machinery from the ExecEnv loops by @elcoosp ([3354bd1](https://github.com/elcoosp/rsmack/commit/3354bd1123c5742f7adff56cab90583e89165064))
- drop builder machinery from the Logr diagnostics loop by @elcoosp ([41cf48a](https://github.com/elcoosp/rsmack/commit/41cf48a868d59dffa1c1894f79a317ca51beda79))
- simplify ExecEnv to use a single prefix and add finish_result by @elcoosp ([bd296f9](https://github.com/elcoosp/rsmack/commit/bd296f9a6ae6d2015a0d810a45bea061145a8349))
- add has_errors helper and must_use attributes by @elcoosp ([f05c94a](https://github.com/elcoosp/rsmack/commit/f05c94a86db4d0175c5531d288c8478fc1ecf515))
- replace macro-based entrypoints with shared generic drivers by @elcoosp ([8e895d0](https://github.com/elcoosp/rsmack/commit/8e895d0930886152e896be52c341898d18f913f8))
- use absolute paths in impl_json_schema macro by @elcoosp ([0a098a9](https://github.com/elcoosp/rsmack/commit/0a098a9a736e19efb2f642e9e26a329946c2bb6b))
- replace proc-macro-error2 with syn::Error based Logr by @elcoosp ([647cf27](https://github.com/elcoosp/rsmack/commit/647cf27c61e5fed4f2c8ab4ccbaf5fe4305cc194))
- robustly find workspace root and refactor folder mirroring by @elcoosp ([e8bcb0d](https://github.com/elcoosp/rsmack/commit/e8bcb0dcd29ca519ca9bc1b4212c85b6d67038d2))
- update cl! macro doc examples by @elcoosp ([5920ad3](https://github.com/elcoosp/rsmack/commit/5920ad35fff29f7c2c4c4cee16c8d5f7baa2454c))

### Testing
- add integration test for folder_iso_struct helper by @elcoosp ([763a673](https://github.com/elcoosp/rsmack/commit/763a6733458dc5d909aad76943942360807de416))

### build
- add criterion dev-dependency and bench target by @elcoosp ([9307851](https://github.com/elcoosp/rsmack/commit/9307851b0118f9e19f2901a7bd7707d0297296b9))
- migrate to workspace dependencies by @elcoosp ([f9a5abf](https://github.com/elcoosp/rsmack/commit/f9a5abf54132b2cba6ae62abc908cb280d7784cc))
**Full Changelog**: https://github.com/elcoosp/rsmack/releases/tag/v1.0.0
