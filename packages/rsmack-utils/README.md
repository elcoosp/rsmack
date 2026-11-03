# rsmack-utils

Utils shared between rsmack crates: the reusable proc-macro **entrypoint drivers**
(darling and raw payload flavors), the `ExecEnv`/`Logr` diagnostics pair, the
process-global file-cache helper and filesystem utilities for build-time codegen.

This is also the crate your `megamac`-generated macro depends on at runtime: `exec`
implementations import `ExecEnv` from it, and the generated entrypoints route through
its drivers.

See the [workspace README](https://github.com/elcoosp/rsmack) for the full documentation.
