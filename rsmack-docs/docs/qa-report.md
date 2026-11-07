# QA report — rsmack documentation site

**Build under test:** final production build (`astro build` exit 0, zero warning lines; `astro check` 0 errors / 0 warnings / 0 hints; `markdownlint-cli2` 23 files / 0 errors).

**Automated pass:** `pnpm qa` — **23 routes × 2 viewports × 2 themes = 92 screenshots, 0 failures.** Per-load probes on every screenshot: theme applied, exactly one `<h1>`, `<main>` text ≥ 200 chars, no horizontal overflow, no console errors, no page errors, no HTTP ≥ 400, no broken images. Raw evidence in `.qa/screenshots/` and `.qa/failures.json` (gitignored).

**Human eyeball pass** over the screenshots (native-resolution crops for anything suspicious), per the checklist:

- Logo sharp and correctly sized in both themes (header glyph and hero, inspected at native resolution) — no blur, no halo, no clipping
- Accent is the derived burnt orange, visible on links, active sidebar items, and TOC highlights — not Starlight default blue
- Code blocks syntax-highlighted (multiple token colors), copy button present, horizontal code scroll without page overflow
- Tables fit; message catalogue tables render nested code spans correctly
- `<Aside>`, `<CardGrid>`, `<Tabs>` (cargo first), `<Steps>`, `<FileTree>`, heading badges all render with correct icons, spacing, borders
- Sidebar active state correct everywhere; groups expand; no duplicated or missing items
- On-page TOC present on long pages, current heading highlighted
- Pagination prev/next point at the right neighbors
- Mobile 390 px: hamburger visible and functional, no clipped content, no overlaps, tap targets usable
- Dark mode: no pure-white panels, no invisible text, code backgrounds distinct from page background
- Typography: no overlapping text, no orphaned headings, no text under the fixed header
- Search: opened the modal, typed `seanum` — Pagefind returned 8 titled, sensible results across reference, guide, and roadmap pages

## Per-route results

| Route | Desktop light | Desktop dark | Mobile light | Mobile dark | Notes |
| --- | --- | --- | --- | --- | --- |
| `/` | Pass | Pass | Pass | Pass | Hero logo, CardGrid, aside verified both themes |
| `/404.html` | Pass | Pass | Pass | Pass | Custom branded 404 (`draft: true` entry); screenshotted once per policy |
| `/explanation/architecture/` | Pass | Pass | Pass | Pass | FileTree + feature code block inspected in dark |
| `/explanation/diagnostics-philosophy/` | Pass | Pass | Pass | Pass | |
| `/explanation/payload-protocol/` | Pass | Pass | Pass | Pass | Protocol table fits both viewports |
| `/explanation/performance/` | Pass | Pass | Pass | Pass | |
| `/explanation/roadmap/` | Pass | Pass | Pass | Pass | Badge in body renders |
| `/guides/docs-from-constants/` | Pass | Pass | Pass | Pass | |
| `/guides/handle-errors/` | Pass | Pass | Pass | Pass | Style-guide table with nested code spans verified |
| `/guides/mirror-a-folder/` | Pass | Pass | Pass | Pass | FileTree fixtures render |
| `/guides/preview-with-playground/` | Pass | Pass | Pass | Pass | Flags table fits mobile |
| `/guides/seaorm-enums/` | Pass | Pass | Pass | Pass | |
| `/guides/wrap-field-types/` | Pass | Pass | Pass | Pass | |
| `/guides/write-your-own-macro/` | Pass | Pass | Pass | Pass | Mobile inspected at native resolution: header, hamburger, code blocks clean |
| `/reference/benchmarks/` | Pass | Pass | Pass | Pass | |
| `/reference/edoc/` | Pass | Pass | Pass | Pass | Longest reference page; grammar table fits, no overflow |
| `/reference/error-messages/` | Pass | Pass | Pass | Pass | 11-row catalogue verified in dark |
| `/reference/folder-iso-struct/` | Pass | Pass | Pass | Pass | Heading badges verified |
| `/reference/megamac/` | Pass | Pass | Pass | Pass | Protocol table fits mobile |
| `/reference/rsmack-utils/` | Pass | Pass | Pass | Pass | |
| `/reference/seanum/` | Pass | Pass | Pass | Pass | Required/optional heading badges verified at native resolution |
| `/reference/wrap/` | Pass | Pass | Pass | Pass | |
| `/tutorials/getting-started/` | Pass | Pass | Pass | Pass | Tabs (cargo first), Steps, aside, text output block; ANSI experiment replaced by `text` block after eyeball, full QA re-run |

## Defects found and fixed

| Defect | Fix | Re-verification |
| --- | --- | --- |
| `ansi` output block rendered visible `␛[1m` control glyphs instead of clean styling | Replaced with a plain `text` block (decision in `docs/decisions/0003-syntax-highlighter.md`) | `pnpm build` → `pnpm preview` → `pnpm qa` re-run, native-res crop confirmed clean |

No open visual defects.
