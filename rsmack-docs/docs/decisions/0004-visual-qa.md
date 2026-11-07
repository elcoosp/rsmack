# ADR 0004: Visual QA — screenshot every route, then eyeball

**Status:** Accepted · **Date:** 2026-11-07

## Context

A green build is not a working site. The skill's Phase 7 requires rendering **every** built page, screenshotting it in 2 viewports × 2 themes, and eyeballing the results; `pnpm qa` exiting 0 is a release blocker.

## Decision

1. **Route enumeration is mechanical.** `scripts/qa-screenshots.mjs` walks `dist/` for every `index.html` (plus `dist/404.html`, screenshotted once) — no hand-maintained route list, so no page can be skipped. Current shape: **23 routes × 2 viewports (1440×900, 390×844) × 2 themes (light, dark) = 92 screenshots** per run.
2. **Automated probes per load:** theme actually applied (`data-theme`), exactly one `<h1>`, `<main>` text ≥ 200 chars, no horizontal overflow (`scrollWidth > innerWidth + 1`), no console errors, no uncaught page errors, no HTTP ≥ 400 responses, no broken `<img>` elements. Failures are recorded in `.qa/failures.json` and fail the run with a per-item report.
3. **Production preview only:** screenshots run against `pnpm preview`, never the dev server.
4. **Human eyeball pass** over the screenshots (home, one page per Diátaxis section, the longest reference pages, mobile, 404 — in both themes), with native-resolution crops for anything suspicious. Findings are recorded in `docs/qa-report.md`.
5. **axe-core is not added** (kept as documented optional hardening). The probes above plus the manual pass cover the required checks; adding `@axe-core/playwright` remains a welcome follow-up.

## Defects found and fixed during the pass

- The `ansi` code block on the getting-started tutorial rendered visible `␛[1m` control glyphs (Expressive Code tokenizes but displays unstyled sequences). Replaced with a plain `text` block; full QA re-run after the fix.

## Re-run rule

Any change to config, CSS, logo, theme, or content invalidates the QA pass. The final repository state is one where `pnpm build` → `pnpm preview` → `pnpm qa` ran **as the last actions before packaging**, and passed.
