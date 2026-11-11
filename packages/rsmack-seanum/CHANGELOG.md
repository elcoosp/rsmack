# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [1.0.0] - 2026-11-11

### Bug Fixes
- readd deps for doc tests by @elcoosp ([c1c180c](https://github.com/elcoosp/rsmack/commit/c1c180cd5ed4b5b28784a4fbe7a1603f822e7484))
- rm sea-orm serde & fake dep, ref from root inside derive by @elcoosp ([81bf4d8](https://github.com/elcoosp/rsmack/commit/81bf4d81d57e40e5ca787287a4570b73966f61e0))
- derive Hash for seanum by @elcoosp ([69f1cfe](https://github.com/elcoosp/rsmack/commit/69f1cfebc126882d2e7f44550ad8bb3da759aecb))
- fix invalid documentation by @elcoosp ([05ac0c7](https://github.com/elcoosp/rsmack/commit/05ac0c78150ffdefd9e37a1c165b178e0f71c164))

### Documentation
- rewrite reference documentation for minimal derives by @elcoosp ([149eba5](https://github.com/elcoosp/rsmack/commit/149eba5ed2dbbc298cf391203ad915202ded4c75))
- update README for SeaORM 2 compatibility by @elcoosp ([5e5091b](https://github.com/elcoosp/rsmack/commit/5e5091ba792ea099256e0d72677186c0ab206643))
- add seanum doc by @elcoosp ([d85aec4](https://github.com/elcoosp/rsmack/commit/d85aec413fcb6362515b2ecde7d87a701206ccaa))

### Features
- support explicit integer discriminants and rename-all cases by @elcoosp ([cccefad](https://github.com/elcoosp/rsmack/commit/cccefad8fa3f3f4dba08cebe468c69ee8d35a28e))
- add integer rs_type support, custom derives, variant renames, and migration helpers by @elcoosp ([d0fd4dd](https://github.com/elcoosp/rsmack/commit/d0fd4dd1c274de6cd7c91a3f2a09b4f17f420d73))
- collect existing attributes by @elcoosp ([8f1c3b7](https://github.com/elcoosp/rsmack/commit/8f1c3b775b09153c7bcf8e38e6ac7faa0e981d4f))
- implement seanum, generate a Clone, Dummy, Debug, PartialEq, EnumIter, DeriveActiveEnum, Eq, Serialize, Deserialize enum wired with correct attrs mirroring the enum by @elcoosp ([2a20614](https://github.com/elcoosp/rsmack/commit/2a2061425bb58e7392fb251045a694ac584d9584))

### Miscellaneous
- 0.17.0 by @elcoosp ([a4a8d3b](https://github.com/elcoosp/rsmack/commit/a4a8d3b0be71c0822baae42f8ed46e186ac2e24c))
- 0.16.0 by @elcoosp ([7df3982](https://github.com/elcoosp/rsmack/commit/7df39828d2d98876275bfde91ad46d08c380178b))
- v0.15.0 by @elcoosp ([90758ae](https://github.com/elcoosp/rsmack/commit/90758ae5cfa4b1bd66f7af00650dd0254d5bd469))
- 0.14.0 by @elcoosp ([fb54b73](https://github.com/elcoosp/rsmack/commit/fb54b73cc8bd9b23e5d6f425e1483d19407f0305))
- 0.13.0 by @elcoosp ([0c56d5c](https://github.com/elcoosp/rsmack/commit/0c56d5c4194ee3861b9c54792715dc19e47e1f13))

### Performance
- adapt active enum benches for unallocated formats by @elcoosp ([0f49cbb](https://github.com/elcoosp/rsmack/commit/0f49cbb5880b0905063b3f157deefa69160f7d4c))
- add criterion expansion benchmarks by @elcoosp ([b60f8be](https://github.com/elcoosp/rsmack/commit/b60f8be8bda812063945d264339ed99cc232f4cb))

### Refactor
- update megamac invocation to use readme docs by @elcoosp ([255d806](https://github.com/elcoosp/rsmack/commit/255d80690de16ed7084eb7d482e9d27b0d5c1d48))
- clean up macro declaration and imports by @elcoosp ([9e25e03](https://github.com/elcoosp/rsmack/commit/9e25e03a46293f16e881811bbb1353f999cc1464))
- use stringcase for snake_case conversion and preserve attrs by @elcoosp ([019b2df](https://github.com/elcoosp/rsmack/commit/019b2df3ae5f0d4ce1c006189a1e8521e4f801d9))

### Testing
- deploy regression checks validating minimal default derives by @elcoosp ([e0deac4](https://github.com/elcoosp/rsmack/commit/e0deac4df4fb2de1657c576b8b3144db74a32f5d))
- add integration tests for seanum macro with SeaORM 2 by @elcoosp ([d4e7f89](https://github.com/elcoosp/rsmack/commit/d4e7f89ef5370cd8b518c9ac9ea11006a464484e))

### build
- add criterion dev-dependency and move sea-orm-migration to dev-deps by @elcoosp ([2bb8a2d](https://github.com/elcoosp/rsmack/commit/2bb8a2d86fb75b507234f08358dbf39384d8caf7))
- migrate to workspace dependencies and update sea-orm by @elcoosp ([007810a](https://github.com/elcoosp/rsmack/commit/007810a1b3be5f718c2e3759353c21a94ab2cfcb))
**Full Changelog**: https://github.com/elcoosp/rsmack/releases/tag/v1.0.0
