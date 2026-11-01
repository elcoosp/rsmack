# Error message style guide

Every user-facing diagnostic in rsmack flows through [`Logr`], so all of them share the
same `#[<crate>::<macro>]` prefix. Keep the message text itself uniform with these
rules — grep-able, stable snapshots in `packages/rsmack-ui-tests/tests/ui/` enforce them.

## Rules

1. **Start with the failure, not a decoration.** No `Error:` prefixes (the `Logr`
   prefix already carries the macro name), no emoji.
2. **Verbs, consistently:**
   - Reading/parsing inputs: `Failed to <verb> ...`
     (`Failed to read constants file ...`, `Failed to parse edoc expression: ...`)
   - Constraint violations: `Only <thing> supported`
     (`Only named struct supported`, `Only Tuple supported, maybe ...`)
   - Unknown variants/enums: `Unsupported <thing> \`<value>\``
     (`Unsupported MacroKind \`Nope\`, expected \`Attr\`, \`Func\` or \`Derive\``)
   - Value construction problems: `Invalid <thing> ...`
     (`Invalid \`vis\` value ...`, `Invalid \`derives\` entry: ...`)
3. **Quote identifiers, paths and values with backticks**; never quote whole sentences.
4. **No trailing period** — rustc convention.
5. **Fatal vs collected.** Fatal errors (the whole expansion is meaningless: unreadable
   file, unparsable item) return `syn::Error` through `ExecEnv::finish_result`.
   Everything that still leaves a usable output (one broken field among many) is
   *collected* with `Logr::emit_error` so **all** problems of one item are reported in
   a single pass (multi-error accumulation).
6. **Never panic.** `expect`/`unwrap`/`panic!` in macro code paths are forbidden: an
   arbitrary user input must turn into a `compile_error!`, not an ICE. Fuzz targets
   (`fuzz/`) enforce this for the parsers; trybuild snapshots (`tests/ui/*.stderr`)
   pin the exact wording.
