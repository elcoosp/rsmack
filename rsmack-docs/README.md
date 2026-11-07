# rsmack documentation site

Production-grade documentation for [rsmack](https://github.com/elcoosp/rsmack) — the Rust proc-macro factory — built with [Astro](https://astro.build) 7.4 and [Starlight](https://starlight.astro.build) 0.42, organized with the Diátaxis framework (tutorials, how-to guides, reference, explanation).

## Commands

All commands run with pnpm from the project root:

| Command           | Action                                       |
| ----------------- | -------------------------------------------- |
| `pnpm install`    | Install dependencies                          |
| `pnpm dev`        | Start the dev server at `localhost:4321`      |
| `pnpm build`      | Production build to `./dist/` (also generates `llms.txt` / `llms-full.txt` via the `postbuild` hook) |
| `pnpm preview`    | Preview the production build locally          |
| `pnpm check`      | Type-check `.astro`/`.ts` sources (`astro check`) |
| `pnpm lint`       | Lint all Markdown/MDX pages (`markdownlint-cli2`) |
| `pnpm qa`         | Screenshot every built route in 2 viewports × 2 themes against `pnpm preview` and fail on runtime defects |

## Editing content

Pages live in `src/content/docs/` under one folder per Diátaxis type:

```
src/content/docs/
├── index.mdx            # landing page (type: overview)
├── tutorials/           # learning-oriented
├── guides/              # task-oriented how-tos
├── reference/           # information-oriented lookup pages
└── explanation/         # understanding-oriented pages
```

Every page must carry `title`, `description`, and `type` frontmatter (`type` is enforced by the collection schema in `src/content.config.ts`). New sidebar entries go into `astro.config.mjs`.

Writing rules enforced by review and CI: second person, present tense, active voice, imperative instructions; every code block declares a language; no "simply", "just", "easy", "obviously"; each page ends with next-step links.

## Brand assets

Derived from the single upstream asset `docs/logo.png` (512×512) by `scripts/brand_assets.py` in the parent workspace tooling:

- `src/assets/logo.png` — untouched full logo (hero use)
- `src/assets/logo-glyph.png` — glyph cutout with the charcoal background keyed to transparent (header logo)
- `public/favicon.svg` — 256 px PNG embedded as a data URI inside an SVG with a `viewBox`

The accent palette in `src/styles/custom.css` derives from the logo's Ferris-orange gradient; contrast ratios are documented in `docs/decisions/0002-logo-format.md`.

## QA

`pnpm build && pnpm preview --port 4321 &` then `pnpm qa`. The script enumerates routes from `dist/` (nothing hand-maintained), screenshots each route at 1440×900 and 390×844 in light and dark, and fails on console errors, page errors, HTTP ≥ 400, horizontal overflow, missing/duplicate `<h1>`, short `<main>`, theme-application failures, or broken images. Results land in `.qa/` (gitignored, not part of the deliverable). See `docs/qa-report.md` for the human pass over those screenshots.

Project decisions are recorded as ADRs in `docs/decisions/`.
