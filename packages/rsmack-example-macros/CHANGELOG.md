# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [1.0.0] - 2026-11-11

### Documentation
- add README for the example macros crate by @elcoosp ([26202b5](https://github.com/elcoosp/rsmack/commit/26202b5655d603fd4a61360d97bf5dd21f46e2ad))

### Features
- write raw payload function-like macro mock by @elcoosp ([d009193](https://github.com/elcoosp/rsmack/commit/d009193d024ae97e0f55bca6d108021b3ea504b8))
- declare generated macros in lib.rs by @elcoosp ([618770b](https://github.com/elcoosp/rsmack/commit/618770bb7be8a219c71fb5b02468eb2b3f886952))
- implement type_name derive macro by @elcoosp ([5ba809a](https://github.com/elcoosp/rsmack/commit/5ba809a667e3fce224809ebcf9319e4f651bc4e6))
- declare impls module by @elcoosp ([cfa0c8f](https://github.com/elcoosp/rsmack/commit/cfa0c8f4d6491c751f74904b3cb77ac4a6833bab))
- implement make_public attribute macro by @elcoosp ([a5eefd5](https://github.com/elcoosp/rsmack/commit/a5eefd54ee163f48608b740051033a452e09302f))
- implement greeting function-like macro by @elcoosp ([c1d4660](https://github.com/elcoosp/rsmack/commit/c1d46603b8a0c1aa87601678b1fef07a775d07a6))
- add rsmack-example-macros crate for living macro examples by @elcoosp ([ae95bf0](https://github.com/elcoosp/rsmack/commit/ae95bf09e22178339c2e62956ee0a47ff95fdf3f))

### Refactor
- declare the shout raw payload test macro by @elcoosp ([fb4d3a7](https://github.com/elcoosp/rsmack/commit/fb4d3a759c2daca03db0f92a3767957296f1a54a))
- strip out-of-date args parameters from type-name derive by @elcoosp ([1a5df10](https://github.com/elcoosp/rsmack/commit/1a5df10a806b3320aa067d2134d719b71aec1df3))
- register shout submodule inside workspace examples by @elcoosp ([e5f6180](https://github.com/elcoosp/rsmack/commit/e5f61803b4ba7337220fbb866210a3ef5560ae35))
- remove receiver arg from megamac invocations by @elcoosp ([7b56b8a](https://github.com/elcoosp/rsmack/commit/7b56b8a46bb15c405383b9e8dbf6c8b6de63606d))
- update exec signature and use expect for dead_code by @elcoosp ([1dba8d9](https://github.com/elcoosp/rsmack/commit/1dba8d92ebffc6cea796b8ae92976afcf7fdf18a))
- update exec signature and derive Default for Args by @elcoosp ([706f408](https://github.com/elcoosp/rsmack/commit/706f4082e581abf7a128b13d8ab64d5cbafafba3))
- update exec signature to take ExecEnv by reference by @elcoosp ([f664d0c](https://github.com/elcoosp/rsmack/commit/f664d0ca837caeec8f122ff16c4cf004f9db4120))

### Testing
- deploy end-to-end integration tests for raw payloads by @elcoosp ([a8b63af](https://github.com/elcoosp/rsmack/commit/a8b63af8caec8534a4308876145afd84b6fa0d5e))
- add end-to-end tests for all example macros by @elcoosp ([34424fb](https://github.com/elcoosp/rsmack/commit/34424fb4e57ef74587973001e694a234369eefd3))
**Full Changelog**: https://github.com/elcoosp/rsmack/releases/tag/v1.0.0
