# ADR 0005: Known warnings — none; the gate and its exceptions

**Status:** Accepted · **Date:** 2026-11-07

## Policy

`pnpm build`, `pnpm check`, and `pnpm lint` logs are grepped for `\b(warn|warning)\b`. Any hit is a release blocker: fix the cause, never filter the output. Third-party warnings that cannot be fixed upstream are the **only** allowable exception and must be recorded here with the exact line and a narrow allowlist entry.

## Current state

**Zero warnings across all three gates** on the final build:

- `pnpm build` — exit 0, zero warning lines.
- `pnpm check` — "0 errors, 0 warnings, 0 hints" (the three initial `'z' is deprecated` hints were fixed by importing `z` from `astro/zod` instead of `astro:content`).
- `pnpm lint` — 23 files, 0 errors.

**The allowlist is empty.** No `grep -v` suppressions exist.

## Warnings that appeared during development and were fixed at the cause

| Warning | Root cause | Fix |
| --- | --- | --- |
| `[sitemap] The Sitemap integration requires the 'site' astro.config option. Skipping.` | Starlight 0.42 always registers `@astrojs/sitemap`; without `site` it warns on every build | Set `site` (see `0006-site-and-plugins.md` for the URL assumption) |
| `[content] Entry docs → 404 was not found.` | Starlight's `get404Route()` probes `getEntry('docs', '404')` for a custom 404 page on every build | Added `src/content/docs/404.mdx` (with `draft: true`, see below) |
| `[build] Could not render '/404' from route '/[...slug]' as it conflicts with higher priority route '/404'.` | The same 404 entry, once added, was also picked up by the catch-all docs route | `draft: true` in its frontmatter: the production catch-all filters drafts out, while `getEntry('docs','404')` still resolves it for Starlight's own `/404` route |
| `[content] The collection "i18n" does not exist or is empty.` | Starlight registers an `i18n` UI-strings collection that must be declared by the project | Declared `i18n: defineCollection({ loader: i18nLoader() })` in `src/content.config.ts` and added `src/content/i18n/en.json` (an empty override object; Starlight's built-in English strings remain active) |
| `InvalidContentEntryDataError: type: Invalid option` | Template example pages predated the required `type` frontmatter schema | Deleted the unused template pages |
