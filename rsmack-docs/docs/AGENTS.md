# AGENTS.md — working on the rsmack docs site

Guidance for humans and AI agents modifying this documentation codebase.

## Stack baselines (do not downgrade)

- Astro **7.4.x** floor, Starlight **0.42.x** floor, pnpm **10.x** as the package manager of record.
- TypeScript is pinned to **6.0.3**. `astro check` does not support TypeScript 7 — see `docs/decisions/0001-typescript-version.md` before changing this.
- Node 22.23+ required by Astro 7.4.

## Non-negotiable quality gates

Run all four in this order; every one must be clean before you commit:

```shell
pnpm build   # exit 0, ZERO warning lines (grep the log for \bwarn(ing)?\b)
pnpm check   # 0 errors, 0 warnings (hints: fix or document in docs/decisions/)
pnpm lint    # markdownlint-cli2 over src/content/docs/**/*.{md,mdx}
pnpm preview --port 4321 &   # then:
pnpm qa      # every built route × 2 viewports × 2 themes must pass
```

Any change to config, CSS, logo, theme, or content invalidates a previous QA pass — re-run `pnpm build` → `pnpm preview` → `pnpm qa` as your **last action** before handing off.

## Content rules

- One Diátaxis type per page, declared in frontmatter (`type: tutorial | how-to | reference | explanation | overview`) and enforced by the schema in `src/content.config.ts`. Do not mix types on one page.
- Every page: `title`, `description`, a one-sentence summary after the heading, prerequisites when relevant, and next-step `<LinkCard>`s at the end.
- Second person, present tense, active voice, imperative instructions. Never write "simply", "just", "easy", or "obviously".
- Every fenced code block declares a language (`rust`, `shell`, `toml`, `text`). Show expected output for commands.
- Only document behavior that exists in the rsmack sources. Verify option names, defaults, and error strings against `packages/*/src/impls/*.rs` and `packages/rsmack-ui-tests` snapshots before writing them.
- MDX: `{` and `<` in prose must live inside code spans; code spans that themselves contain backticks use double-backtick delimiters (`` ``like `this` `` ``).
- The sidebar is explicit in `astro.config.mjs` — add new pages there.

## Brand and theme

- Logo assets are derived from the single upstream `docs/logo.png`; regenerate them with the asset pipeline rather than editing pixels by hand. Never upscale the raster.
- The accent palette lives in `src/styles/custom.css`, derived from the logo; if you touch it, re-verify WCAG AA contrast in both themes (method in `docs/decisions/0002-logo-format.md`).
- Dark mode is a first-class theme: never disable the Starlight theme toggle.
- Search is Pagefind (built in). Do not remove it.

## llms.txt

`pnpm build` triggers `scripts/generate-llms.mjs` (postbuild), which regenerates `dist/llms.txt` and `dist/llms-full.txt` from the built routes. Do not hand-edit them.

## Deliverable

The zip artifact contains this docs codebase only — no `node_modules/`, `dist/`, `.astro/`, `.qa/`, or the upstream Rust sources.
