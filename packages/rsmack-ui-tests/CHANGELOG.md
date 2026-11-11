# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [1.0.0] - 2026-11-11

### Documentation
- document signature-driven payloads and facade for v1.0.0 by @elcoosp ([e8ac8d6](https://github.com/elcoosp/rsmack/commit/e8ac8d6d4d8d82908fc10d2563527534b0889185))

### Testing
- adapt tuple struct error test path for inline options by @elcoosp ([d2f7e00](https://github.com/elcoosp/rsmack/commit/d2f7e00788e0a49736349ee00597e15a23e4b2ae))
- deploy trybuild error snapshot for ambiguous bare names by @elcoosp ([06f0c6f](https://github.com/elcoosp/rsmack/commit/06f0c6fe2e3c8fb4134373b77e34c7fbc2c291a1))
- deploy trybuild error snapshot for ambiguous bare names by @elcoosp ([8788997](https://github.com/elcoosp/rsmack/commit/8788997f5b8c43418c024f30175afe1555382b50))
- synchronize scratch file templates for qualified modules by @elcoosp ([df1780f](https://github.com/elcoosp/rsmack/commit/df1780f46ebbe8bab3dc5341b6be10b6548ba2a6))
- add qualified module const fixtures for edoc validation by @elcoosp ([bf12106](https://github.com/elcoosp/rsmack/commit/bf1210612af24b22824b63228aca800857c1969f))
- add stderr snapshot for wrap unsupported type by @elcoosp ([4134a51](https://github.com/elcoosp/rsmack/commit/4134a51d98b9a07947a41ca2cf934b26eaffcc4e))
- add trybuild snapshot for wrap unsupported type by @elcoosp ([77a20e5](https://github.com/elcoosp/rsmack/commit/77a20e50076bc059c91bb6f8abf7fc4f6423f7c0))
- add stderr snapshot for wrap tuple struct by @elcoosp ([1568c58](https://github.com/elcoosp/rsmack/commit/1568c586866e63c0a114a4ec982e2323df173469))
- add trybuild snapshot for wrap tuple struct by @elcoosp ([1d06a6d](https://github.com/elcoosp/rsmack/commit/1d06a6dd4eb24d653d66039350a17679e1dcef96))
- add stderr snapshot for seanum integer non-unit variant by @elcoosp ([e03c903](https://github.com/elcoosp/rsmack/commit/e03c903784000b3d8c46284c3255927e9893a42e))
- add trybuild snapshot for seanum integer non-unit variant by @elcoosp ([6a2e564](https://github.com/elcoosp/rsmack/commit/6a2e564970e30f7fa5c43d46cefb4a01de5cd720))
- add stderr snapshot for megamac missing impl file by @elcoosp ([08f5fc1](https://github.com/elcoosp/rsmack/commit/08f5fc150ea3eaa3f208bdb6ad679b93a8a1c129))
- add trybuild snapshot for megamac missing impl file by @elcoosp ([32b8536](https://github.com/elcoosp/rsmack/commit/32b853635f7f93e18b5a1f329e8b17f1c7c46dd2))
- add stderr snapshot for megamac broken impl by @elcoosp ([49da1a0](https://github.com/elcoosp/rsmack/commit/49da1a07e21dcfa3462fb029bd55a6c5aec37aaf))
- add trybuild snapshot for megamac broken impl by @elcoosp ([80a0fba](https://github.com/elcoosp/rsmack/commit/80a0fba137813afb25919272f320648092a77888))
- add stderr snapshot for megamac bad kind by @elcoosp ([e86350f](https://github.com/elcoosp/rsmack/commit/e86350fb6ee3b63e7dd9cf39d8d4a30d86134972))
- add trybuild snapshot for megamac bad kind by @elcoosp ([81b24d3](https://github.com/elcoosp/rsmack/commit/81b24d31043567094dc159db74c7a8386da26adc))
- add stderr snapshot for fs tuple struct by @elcoosp ([d6351c5](https://github.com/elcoosp/rsmack/commit/d6351c544c5c395a9d21cb7a3b0f0815c4de7407))
- add trybuild snapshot for fs tuple struct by @elcoosp ([2a3683d](https://github.com/elcoosp/rsmack/commit/2a3683d0a592bbe3b2706dee3c21d2c6480e8a26))
- add stderr snapshot for edoc unsupported element by @elcoosp ([3eb53dd](https://github.com/elcoosp/rsmack/commit/3eb53ddf03e27d563355c2063ecc5e1eb9f5edbd))
- add trybuild snapshot for edoc unsupported element by @elcoosp ([4ff8082](https://github.com/elcoosp/rsmack/commit/4ff8082f1e95bfa0b651a77ec2d8ec7d2daa2aaa))
- add stderr snapshot for edoc unresolved const by @elcoosp ([efa3a4a](https://github.com/elcoosp/rsmack/commit/efa3a4a6cad00dec03f7ae21d68b4f0bf44a8dbc))
- add trybuild snapshot for edoc unresolved const by @elcoosp ([1b54de6](https://github.com/elcoosp/rsmack/commit/1b54de6261c372ffc9c222c7d8e9c78fac38f9a3))
- add stderr snapshot for edoc non-tuple expression by @elcoosp ([118cdce](https://github.com/elcoosp/rsmack/commit/118cdcebbba05889ea2bbfd0ef978a18d2185b53))
- add trybuild snapshot for edoc non-tuple expression by @elcoosp ([96b1758](https://github.com/elcoosp/rsmack/commit/96b1758ed1a9be1a80a19fd85c6c3fc258e0436a))
- add stderr snapshot for edoc missing consts file by @elcoosp ([aa0fd38](https://github.com/elcoosp/rsmack/commit/aa0fd38c9b0937351efe570312fa322459a045d5))
- add trybuild snapshot for edoc missing consts file by @elcoosp ([1979879](https://github.com/elcoosp/rsmack/commit/1979879abfc1114d5efcacfdf67a3b7571ce1912))
- add trybuild UI test runner with scratch project seeding by @elcoosp ([01335d7](https://github.com/elcoosp/rsmack/commit/01335d7aa90411dec76f577c677c76de996222ba))
- add placeholder impls fixture for megamac UI cases by @elcoosp ([2c14832](https://github.com/elcoosp/rsmack/commit/2c14832acf2e82de672c7429e4bd82ae54a86bba))
- add fixture constants file for edoc UI cases by @elcoosp ([687cba6](https://github.com/elcoosp/rsmack/commit/687cba69ee6b114e551440b0bea39770dafeeba3))

### build
- add trybuild UI tests crate configuration by @elcoosp ([e8cbeef](https://github.com/elcoosp/rsmack/commit/e8cbeef288efeb2f459c1117bed9fea55ab89d11))
**Full Changelog**: https://github.com/elcoosp/rsmack/releases/tag/v1.0.0
