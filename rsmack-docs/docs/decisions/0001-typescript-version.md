# ADR 0001: Pin TypeScript to 6.x for astro check

**Status:** Accepted · **Date:** 2026-11-07

## Context

The latest stable TypeScript at authoring time is **7.0.2** (`pnpm view typescript version`). Astro 7.4's `astro check` does not support TypeScript 7. The check implemented in `node_modules/astro/dist/cli/check/index.js` reads:

> "astro check does not currently support TypeScript 7.0. To continue using astro check, install TypeScript 6 instead.
>
> astro check will be deprecated in a future Astro release. Experimental support for type-checking Astro files with TypeScript 7.1+ is available through @astrojs/ts-content-mapper. See its README for setup instructions: https://github.com/withastro/astro/tree/main/packages/language-tools/ts-content-mapper#usage"

`@astrojs/check@0.9.12` declares `peerDependencies: { typescript: "^5.0.0 || ^6.0.0" }` — TypeScript 7 is outside every supported range.

## Decision

Pin `typescript` to the latest 6.x release, **6.0.3** (exact version in `package.json`). Use the stable `astro check` path.

## Rejected alternatives

- **TypeScript 7 + `tsc` + `@astrojs/ts-content-mapper`:** explicitly experimental, not the stable default, and not appropriate for a deliverable that must pass `astro check` with zero errors and zero warnings today.
- **Leaving TypeScript floating (`^5 || ^6`):** a future 7.x install would break `pnpm check` silently. The exact pin prevents that.

## Consequences

- `pnpm check` passes with 0 errors, 0 warnings, 0 hints on this pin.
- When Astro ships stable TypeScript 7 support, bump deliberately and update this ADR.
