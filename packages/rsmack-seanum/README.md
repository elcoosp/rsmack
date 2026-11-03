# rsmack-seanum

`#[seanum]` — `SeaORM` active enums with sane defaults, migrations and discriminants.

```rust,ignore
use rsmack_seanum::seanum;

#[seanum(
    rs_type = "String",
    db_type = "String(sea_orm::sea_query::StringLen::None)",
    derives("::serde::Serialize", "::serde::Deserialize")
)]
#[derive(Default)]
pub enum SwitchAction {
    #[default]
    #[seanum(rename = "send_email")]
    SendEmail,
    SendSms,
}
```

The enum name is snake-cased for `enum_name` (overridable with `enum_name = "..."`),
each variant gets its `string_value` (overridable per variant with `#[seanum(rename =
"...")]`, or case-normalized fleet-wide with `rename_all = "camelCase"` | `"snake_case"`
| `"kebab-case"` | `"PascalCase"` | `"SCREAMING_SNAKE_CASE"` | `"lowercase"` |
`"UPPERCASE"`).

**Minimal default derives**: `Clone, Debug, PartialEq, Eq, Hash, EnumIter,
DeriveActiveEnum` — nothing else, so no hidden `serde`/`fake` dependency. Anything else
is opt-in via `derives(...)`.

**Integer primary keys**: with an integer `rs_type` (`"i32"`, `"u8"`, ...) the macro adds
`#[repr(...)]`, per-variant `#[sea_orm(num_value = N)]` and a value-based
`TryFrom<rs_type>`. Explicit discriminants are honored (`A = 10, B` continues from 11);
duplicates are reported.

**Migrations**: `migration = true` generates a `SwitchAction::migration_column_def()`
helper (requires the [`sea-orm-migration`](https://docs.rs/sea-orm-migration) dependency)
returning a ready `ColumnDef` for the enum's column.

See the [workspace README](https://github.com/elcoosp/rsmack) for the full documentation.
