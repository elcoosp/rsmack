# ADR 0003: Syntax highlighting — Shiki via Expressive Code (the Starlight default)

**Status:** Accepted · **Date:** 2026-11-07

## Context

Starlight renders code blocks through **Expressive Code**, whose supported highlighter is **Shiki** — TextMate grammars (the same ones VS Code uses), 180+ languages, build-time execution with zero client-side JavaScript, dual light/dark themes, and support for transformers and Twoslash.

An alternative lightweight highlighter, **Rangi**, exists (~13 kB all-in, regex tokenizer, ~46 languages, built-in theme pairs). Its own comparison table positions it for "chat UIs, SSG, small apps", while docs sites with rich code samples are Shiki's territory. Rangi is not integrated with Expressive Code; adopting it would mean writing and maintaining a custom highlighter plugin with no upstream support, and losing transformers, Twoslash, and the language coverage documentation needs.

## Decision

Ship with **Shiki through Starlight's built-in Expressive Code integration** and do not add Rangi or any other custom highlighter. No `expressiveCode` overrides are configured: the default dual-theme setup is already correct for this site, and the skill guidance is to add theme/language pinning only for a concrete need (none exists here — all snippets are `rust`, `shell`, `toml`, or `text`, all covered by the defaults).

## Authoring rules that follow from this

- Every fenced code block declares a language; unlabeled fences render as plain text.
- Terminal output uses `ansi` or plain `text`. Measured during QA: Expressive Code's `ansi` grammar renders unstyled control sequences with visible `␛` glyphs, so for static command output a plain `text` block is the correct choice (adopted on the getting-started page).
- Expressive Code's built-ins — line highlighting (`{1,3-5}`), word wrap, the copy button — stay available to authors; no client-side highlighter is added on top.

## Consequences

- Zero additional dependencies; highlighting is build-time only.
- Dark mode code blocks follow Starlight's dual themes automatically (verified in the eyeball pass: distinct background, multiple token colors in both themes).
