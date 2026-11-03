//! Mini constant-expression evaluator for `edoc`.
//!
//! Evaluates the subset of const expressions users actually put in documentation
//! constants: literals, arithmetic/bitwise/shift/comparison operations, casts,
//! `concat!`, `env!`, `option_env!` and references to other constants of the same
//! file. Anything outside this subset is reported as a collected error instead of
//! aborting the whole expansion.
//!
//! Values are **typed** ([`ConstValue`]): arithmetic operates on machine integers and
//! floats, and string rendering happens once at doc assembly — no string round-trips.
//! References resolve against full module paths (`nested::DEEP`), with a unique
//! bare-name fallback and an explicit ambiguity error.
//!
//! This mirrors what rustc's own `min_const_eval` accepts, without pulling a full
//! interpreter dependency.

use std::collections::HashMap;

use quote::ToTokens;
use syn::parse::Parser;

/// A typed constant value.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ConstValue {
    /// A string literal.
    Str(String),
    /// An integer literal (any rust integer width fits `i128`).
    Int(i128),
    /// A float literal (rendered as `f64`).
    Float(f64),
    /// A boolean literal.
    Bool(bool),
    /// A byte-string literal.
    Bytes(Vec<u8>),
    /// A character literal.
    Char(char),
}

impl std::fmt::Display for ConstValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Str(value) => f.write_str(value),
            Self::Int(value) => write!(f, "{value}"),
            Self::Float(value) => write!(f, "{value}"),
            Self::Bool(value) => write!(f, "{value}"),
            Self::Bytes(value) => write!(f, "{value:?}"),
            Self::Char(value) => write!(f, "{value}"),
        }
    }
}

impl ConstValue {
    /// The user-facing kind name used in diagnostics.
    fn kind(&self) -> &'static str {
        match self {
            Self::Str(_) => "string",
            Self::Int(_) => "integer",
            Self::Float(_) => "float",
            Self::Bool(_) => "bool",
            Self::Bytes(_) => "byte string",
            Self::Char(_) => "char",
        }
    }
}

/// Resolved constants of a source file, keyed by full module path (`nested::DEEP`).
pub(crate) type Consts = HashMap<String, ConstValue>;

/// Evaluation result: the constant value, or a user-facing reason.
pub(crate) type EvalResult = Result<ConstValue, String>;

/// Evaluate an expression, resolving const-to-const references with `resolve_const`.
pub(crate) fn eval(
    expr: &syn::Expr,
    consts: &Consts,
    resolve_const: &mut dyn FnMut(&str) -> EvalResult,
) -> EvalResult {
    match expr {
        syn::Expr::Lit(lit) => eval_lit(&lit.lit),
        syn::Expr::Paren(paren) => eval(&paren.expr, consts, resolve_const),
        syn::Expr::Path(path) => eval_path(path, consts, resolve_const),
        syn::Expr::Unary(unary) => eval_unary(unary, consts, resolve_const),
        syn::Expr::Binary(binary) => eval_binary(binary, consts, resolve_const),
        syn::Expr::Cast(cast) => eval_cast(cast, consts, resolve_const),
        syn::Expr::Macro(mac) => eval_macro(mac, consts, resolve_const),
        other => Err(format!(
            "Unsupported const expression `{}`",
            other.to_token_stream()
        )),
    }
}

/// Normalize a path expression to its `::`-joined form (`nested::DEEP`).
fn normalize_path(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

/// Resolve a reference path against `keys`: exact match first, then a unique
/// bare-name match.
///
/// * `None` — no key matches at all;
/// * `Some(Ok(key))` — the resolved table key;
/// * `Some(Err(reason))` — several keys share the bare name: ambiguous reference.
pub(crate) fn match_key<'a>(
    ref_path: &str,
    keys: impl IntoIterator<Item = &'a String>,
) -> Option<Result<String, String>> {
    let all: Vec<&String> = keys.into_iter().collect();
    if all.iter().any(|key| key.as_str() == ref_path) {
        return Some(Ok(ref_path.to_string()));
    }
    let bare = ref_path.rsplit("::").next().unwrap_or(ref_path);
    // Sorted so diagnostics are deterministic regardless of the underlying table.
    let mut candidates: Vec<&String> = all
        .into_iter()
        .filter(|key| key.rsplit("::").next() == Some(bare))
        .collect();
    candidates.sort();
    match candidates.as_slice() {
        [] => None,
        [only] => Some(Ok((*only).clone())),
        many => Some(Err(format!(
            "Ambiguous const `{bare}`, qualify the path: {}",
            many.iter()
                .map(|candidate| format!("`{candidate}`"))
                .collect::<Vec<_>>()
                .join(", ")
        ))),
    }
}

/// Evaluate a path expression: a resolved const lookup or a lazy const reference.
fn eval_path(
    path: &syn::ExprPath,
    consts: &Consts,
    resolve_const: &mut dyn FnMut(&str) -> EvalResult,
) -> EvalResult {
    let ref_path = normalize_path(&path.path);
    if let Some(resolved) = match_key(&ref_path, consts.keys()) {
        let key = resolved?;
        if let Some(value) = consts.get(&key) {
            return Ok(value.clone());
        }
    }
    resolve_const(&ref_path)
}

/// Evaluate a literal into its typed value.
fn eval_lit(lit: &syn::Lit) -> EvalResult {
    match lit {
        syn::Lit::Str(lit) => Ok(ConstValue::Str(lit.value())),
        syn::Lit::Bool(lit) => Ok(ConstValue::Bool(lit.value())),
        syn::Lit::ByteStr(lit) => Ok(ConstValue::Bytes(lit.value())),
        syn::Lit::Int(lit) => lit
            .base10_parse::<i128>()
            .map(ConstValue::Int)
            .map_err(|_| format!("Integer literal `{lit}` out of range")),
        // `f64` parsing saturates to infinity instead of failing, so there is no
        // reachable error branch here.
        syn::Lit::Float(lit) => Ok(ConstValue::Float(
            lit.base10_parse::<f64>().unwrap_or(f64::INFINITY),
        )),
        syn::Lit::Char(lit) => Ok(ConstValue::Char(lit.value())),
        other => Err(format!("Unsupported literal `{}`", other.to_token_stream())),
    }
}

/// Evaluate a unary expression (`-x`, `!x`).
fn eval_unary(
    unary: &syn::ExprUnary,
    consts: &Consts,
    resolve_const: &mut dyn FnMut(&str) -> EvalResult,
) -> EvalResult {
    let operator = unary.op.to_token_stream().to_string();
    // The operator is matched first: an unsupported operator reports itself even when
    // the operand is broken too.
    if operator != "-" && operator != "!" {
        // `*` (dereference) and any future unary operator
        return Err(format!("Unsupported unary operator `{operator}`"));
    }
    let operand = eval(&unary.expr, consts, resolve_const)?;
    match (operator.as_str(), operand) {
        ("-", ConstValue::Int(value)) => value
            .checked_neg()
            .map(ConstValue::Int)
            .ok_or_else(|| format!("Arithmetic overflow negating `{value}`")),
        ("-", ConstValue::Float(value)) => Ok(ConstValue::Float(-value)),
        ("!", ConstValue::Bool(value)) => Ok(ConstValue::Bool(!value)),
        ("!", ConstValue::Int(value)) => Ok(ConstValue::Int(!value)),
        ("-", other) => Err(format!(
            "Unsupported operand for negation: `{}` ({})",
            other,
            other.kind()
        )),
        ("!", other) => Err(format!(
            "Unsupported operand for `!`: `{}` ({})",
            other,
            other.kind()
        )),
        _ => unreachable!("the operator was validated above"),
    }
}

/// Evaluate a binary expression (arithmetic, bitwise, shifts, logic, comparisons).
fn eval_binary(
    binary: &syn::ExprBinary,
    consts: &Consts,
    resolve_const: &mut dyn FnMut(&str) -> EvalResult,
) -> EvalResult {
    let lhs = eval(&binary.left, consts, resolve_const)?;
    let rhs = eval(&binary.right, consts, resolve_const)?;
    let op = binary.op.to_token_stream().to_string();
    match &binary.op {
        syn::BinOp::Add(_) => arith(&op, &lhs, &rhs, i128::checked_add, |a, b| a + b),
        syn::BinOp::Sub(_) => arith(&op, &lhs, &rhs, i128::checked_sub, |a, b| a - b),
        syn::BinOp::Mul(_) => arith(&op, &lhs, &rhs, i128::checked_mul, |a, b| a * b),
        syn::BinOp::Div(_) => arith(&op, &lhs, &rhs, i128::checked_div, |a, b| a / b),
        syn::BinOp::Rem(_) if matches!(rhs, ConstValue::Int(0)) => {
            Err(format!("Division by zero: `{lhs} {op} {rhs}`"))
        }
        syn::BinOp::Rem(_) => int_op(&op, &lhs, &rhs, i128::checked_rem),
        syn::BinOp::BitAnd(_) => int_op(&op, &lhs, &rhs, |a, b| Some(a & b)),
        syn::BinOp::BitOr(_) => int_op(&op, &lhs, &rhs, |a, b| Some(a | b)),
        syn::BinOp::BitXor(_) => int_op(&op, &lhs, &rhs, |a, b| Some(a ^ b)),
        syn::BinOp::Shl(_) => shift(&lhs, &rhs, i128::checked_shl),
        syn::BinOp::Shr(_) => shift(&lhs, &rhs, i128::checked_shr),
        syn::BinOp::And(_) => bool_op(&op, &lhs, &rhs, |a, b| a && b),
        syn::BinOp::Or(_) => bool_op(&op, &lhs, &rhs, |a, b| a || b),
        syn::BinOp::Eq(_) => compare(&op, &lhs, &rhs, std::cmp::Ordering::is_eq),
        syn::BinOp::Ne(_) => compare(&op, &lhs, &rhs, std::cmp::Ordering::is_ne),
        syn::BinOp::Lt(_) => compare(&op, &lhs, &rhs, std::cmp::Ordering::is_lt),
        syn::BinOp::Le(_) => compare(&op, &lhs, &rhs, std::cmp::Ordering::is_le),
        syn::BinOp::Gt(_) => compare(&op, &lhs, &rhs, std::cmp::Ordering::is_gt),
        syn::BinOp::Ge(_) => compare(&op, &lhs, &rhs, std::cmp::Ordering::is_ge),
        // `BinOp` is `#[non_exhaustive]`: assignment and future operators never form
        // const expressions.
        other => Err(format!(
            "Unsupported binary operator `{}`",
            other.to_token_stream()
        )),
    }
}

/// Mismatched / unsupported operand pair error.
fn operand_err(op: &str, lhs: &ConstValue, rhs: &ConstValue) -> String {
    format!(
        "Unsupported operands `{lhs}` ({}) and `{rhs}` ({}) for `{op}`",
        lhs.kind(),
        rhs.kind()
    )
}

/// Apply a checked integer operation, falling back to floats.
fn arith(
    op: &str,
    lhs: &ConstValue,
    rhs: &ConstValue,
    int_op: fn(i128, i128) -> Option<i128>,
    float_op: fn(f64, f64) -> f64,
) -> EvalResult {
    match (lhs, rhs) {
        (ConstValue::Int(a), ConstValue::Int(b)) => {
            int_op(*a, *b).map(ConstValue::Int).ok_or_else(|| {
                if (op == "/" || op == "%") && *b == 0 {
                    format!("Division by zero: `{lhs} {op} {rhs}`")
                } else {
                    format!("Arithmetic overflow computing `{lhs} {op} {rhs}`")
                }
            })
        }
        (ConstValue::Float(a), ConstValue::Float(b)) => Ok(ConstValue::Float(float_op(*a, *b))),
        _ => Err(operand_err(op, lhs, rhs)),
    }
}

/// Apply a checked integer-only operation.
fn int_op(
    op: &str,
    lhs: &ConstValue,
    rhs: &ConstValue,
    op_fn: fn(i128, i128) -> Option<i128>,
) -> EvalResult {
    match (lhs, rhs) {
        (ConstValue::Int(a), ConstValue::Int(b)) => op_fn(*a, *b)
            .map(ConstValue::Int)
            .ok_or_else(|| format!("Failed to compute `{lhs} {op} {rhs}`")),
        _ => Err(operand_err(op, lhs, rhs)),
    }
}

/// Apply a shift operation with shift-amount validation.
fn shift(lhs: &ConstValue, rhs: &ConstValue, op: fn(i128, u32) -> Option<i128>) -> EvalResult {
    let (ConstValue::Int(a), ConstValue::Int(b)) = (lhs, rhs) else {
        return Err(operand_err("<<", lhs, rhs));
    };
    let amount = u32::try_from(*b).map_err(|_| format!("Shift amount out of range: `{rhs}`"))?;
    op(*a, amount)
        .map(ConstValue::Int)
        .ok_or_else(|| format!("Arithmetic overflow shifting `{lhs}` by `{rhs}`"))
}

/// Apply a boolean operation.
fn bool_op(
    op: &str,
    lhs: &ConstValue,
    rhs: &ConstValue,
    op_fn: fn(bool, bool) -> bool,
) -> EvalResult {
    match (lhs, rhs) {
        (ConstValue::Bool(a), ConstValue::Bool(b)) => Ok(ConstValue::Bool(op_fn(*a, *b))),
        _ => Err(operand_err(op, lhs, rhs)),
    }
}

/// Apply an ordering/equality comparison between two same-kind operands.
fn compare(
    op: &str,
    lhs: &ConstValue,
    rhs: &ConstValue,
    keep: fn(std::cmp::Ordering) -> bool,
) -> EvalResult {
    use ConstValue::{Bool, Char, Float, Int, Str};
    let ordering = match (lhs, rhs) {
        (Int(a), Int(b)) => a.cmp(b),
        (Float(a), Float(b)) => a.total_cmp(b),
        (Str(a), Str(b)) => a.cmp(b),
        (Char(a), Char(b)) => a.cmp(b),
        (Bool(a), Bool(b)) => match op {
            "==" => {
                return Ok(ConstValue::Bool(a == b));
            }
            "!=" => {
                return Ok(ConstValue::Bool(a != b));
            }
            _ => return Err(operand_err(op, lhs, rhs)),
        },
        _ => return Err(operand_err(op, lhs, rhs)),
    };
    Ok(ConstValue::Bool(keep(ordering)))
}

/// Two's-complement wrapping identical to rust's integer `as` casts.
fn wrap_int(int: i128, bits: u32, signed: bool) -> i128 {
    // `bits <= 64` for the narrowing targets so the shift cannot overflow; the wide
    // targets (`i128`, `u128`, `isize`, `usize`) are returned unchanged below.
    let modulus = 1i128 << bits;
    let masked = ((int % modulus) + modulus) % modulus;
    if signed && masked >= modulus / 2 {
        masked - modulus
    } else {
        masked
    }
}

/// Saturating float-to-int cast, mirroring rust's `as` semantics since 1.45.
///
/// The precision-losing and truncating casts below are the feature itself (they mirror
/// rustc's `as`).
#[expect(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn saturate_float_to_int(value: f64, bits: u32, signed: bool) -> i128 {
    if value.is_nan() {
        return 0;
    }
    let (min, max): (i128, i128) = match (bits, signed) {
        (128, true) => (i128::MIN, i128::MAX),
        // `u128::MAX` is not representable as `f64` anyway: saturate at `i128::MAX`.
        (128, false) => (0, i128::MAX),
        _ if signed => (-(1i128 << (bits - 1)), (1i128 << (bits - 1)) - 1),
        _ => (0, (1i128 << bits) - 1),
    };
    let truncated = value.trunc();
    if truncated <= min as f64 {
        min
    } else if max as f64 <= truncated {
        max
    } else {
        truncated as i128
    }
}

/// int -> f64 cast; the precision loss mirrors rustc's `as` (this is the feature).
#[expect(clippy::cast_precision_loss)]
fn int_to_float(int: i128) -> f64 {
    int as f64
}

/// f64 -> f32 -> f64 narrowing, mirroring `expr as f32` promoted back to `f64`.
#[expect(clippy::cast_possible_truncation)]
fn narrow_f32(float: f64) -> f64 {
    f64::from(float as f32)
}

/// Evaluate a cast expression (`expr as usize`), mirroring integer `as` wrapping and
/// float `as` saturation.
fn eval_cast(
    cast: &syn::ExprCast,
    consts: &Consts,
    resolve_const: &mut dyn FnMut(&str) -> EvalResult,
) -> EvalResult {
    let target = cast.ty.to_token_stream().to_string().replace(' ', "");
    let value = eval(&cast.expr, consts, resolve_const)?;
    let int_target = |bits: u32, signed: bool| -> EvalResult {
        match &value {
            ConstValue::Int(int) => Ok(ConstValue::Int(wrap_int(*int, bits, signed))),
            ConstValue::Bool(bool) => {
                Ok(ConstValue::Int(wrap_int(i128::from(*bool), bits, signed)))
            }
            ConstValue::Float(float) => {
                Ok(ConstValue::Int(saturate_float_to_int(*float, bits, signed)))
            }
            other => Err(format!(
                "Unsupported cast operand `{other}` ({}) for `{target}`",
                other.kind()
            )),
        }
    };
    match target.as_str() {
        "i8" => int_target(8, true),
        "i16" => int_target(16, true),
        "i32" => int_target(32, true),
        "i64" => int_target(64, true),
        "u8" => int_target(8, false),
        "u16" => int_target(16, false),
        "u32" => int_target(32, false),
        "u64" => int_target(64, false),
        "i128" | "isize" | "u128" | "usize" => match &value {
            ConstValue::Int(int) => Ok(ConstValue::Int(*int)),
            ConstValue::Bool(bool) => Ok(ConstValue::Int(i128::from(*bool))),
            ConstValue::Float(float) => {
                Ok(ConstValue::Int(saturate_float_to_int(*float, 128, false)))
            }
            other => Err(format!(
                "Unsupported cast operand `{other}` ({}) for `{target}`",
                other.kind()
            )),
        },
        "f32" | "f64" => match &value {
            ConstValue::Int(int) => Ok(ConstValue::Float(int_to_float(*int))),
            ConstValue::Float(float) => Ok(ConstValue::Float(if target == "f32" {
                narrow_f32(*float)
            } else {
                *float
            })),
            ConstValue::Bool(bool) => Ok(ConstValue::Float(f64::from(u8::from(*bool)))),
            other => Err(format!(
                "Unsupported cast operand `{other}` ({}) for `{target}`",
                other.kind()
            )),
        },
        other => Err(format!("Unsupported cast target `{other}`")),
    }
}

/// Evaluate the supported macro invocations: `concat!`, `env!` and `option_env!`.
fn eval_macro(
    mac: &syn::ExprMacro,
    consts: &Consts,
    resolve_const: &mut dyn FnMut(&str) -> EvalResult,
) -> EvalResult {
    let name = mac
        .mac
        .path
        .segments
        .last()
        .map(|s| s.ident.to_string())
        .unwrap_or_default();
    let parser = syn::punctuated::Punctuated::<syn::Expr, syn::token::Comma>::parse_terminated;
    let args: Vec<syn::Expr> = parser
        .parse2(mac.mac.tokens.clone())
        .map(|parsed| parsed.into_iter().collect())
        .map_err(|_| format!("Failed to parse the arguments of `{name}!`"))?;
    match name.as_str() {
        "concat" => {
            let mut joined = String::new();
            for arg in &args {
                joined.push_str(&eval(arg, consts, resolve_const)?.to_string());
            }
            Ok(ConstValue::Str(joined))
        }
        "env" => {
            let key = env_key(&args, &name)?;
            std::env::var(&key)
                .map(ConstValue::Str)
                .map_err(|_| format!("Failed to read env var `{key}`: not set at compile time"))
        }
        "option_env" => {
            let key = env_key(&args, &name)?;
            Ok(ConstValue::Str(std::env::var(&key).unwrap_or_default()))
        }
        other => Err(format!("Unsupported const macro `{other}!`")),
    }
}

/// Extract the environment variable key (a single string literal) from macro args.
fn env_key(args: &[syn::Expr], name: &str) -> Result<String, String> {
    let [
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(lit),
            ..
        }),
    ] = args
    else {
        return Err(format!("`{name}!` expects a single string literal key"));
    };
    Ok(lit.value())
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use std::str::FromStr;

    /// Shared resolver used by the helper tests: always reports the ident as unresolved.
    fn unresolved(ident: &str) -> EvalResult {
        Err(format!("Unresolved const ident `{ident}`"))
    }

    fn eval_ok(tokens: proc_macro2::TokenStream) -> String {
        let expr: syn::Expr = syn::parse2(tokens).expect("valid expr");
        eval(&expr, &Consts::new(), &mut unresolved)
            .expect("evaluation should succeed")
            .to_string()
    }

    fn eval_err(tokens: proc_macro2::TokenStream) -> String {
        let expr: syn::Expr = syn::parse2(tokens).expect("valid expr");
        eval(&expr, &Consts::new(), &mut unresolved).expect_err("evaluation should fail")
    }

    #[test]
    fn literals_render_as_strings() {
        assert_eq!(eval_ok(quote! { "hello" }), "hello");
        assert_eq!(eval_ok(quote! { true }), "true");
        assert_eq!(eval_ok(quote! { b"ab" }), "[97, 98]");
        assert_eq!(eval_ok(quote! { 42 }), "42");
        assert_eq!(eval_ok(quote! { 1.5 }), "1.5");
        assert_eq!(eval_ok(quote! { 'x' }), "x");
    }

    #[test]
    fn arithmetic_and_shifts_work() {
        assert_eq!(eval_ok(quote! { 1 + 2 }), "3");
        assert_eq!(eval_ok(quote! { (1 + 2) * 4 }), "12");
        assert_eq!(eval_ok(quote! { 7 / 2 }), "3");
        assert_eq!(eval_ok(quote! { 7 % 2 }), "1");
        assert_eq!(eval_ok(quote! { -5 }), "-5");
        assert_eq!(eval_ok(quote! { 1 << 3 }), "8");
        assert_eq!(eval_ok(quote! { 16 >> 2 }), "4");
        assert_eq!(eval_ok(quote! { 6 & 3 }), "2");
        assert_eq!(eval_ok(quote! { 6 | 3 }), "7");
        assert_eq!(eval_ok(quote! { 6 ^ 3 }), "5");
        assert_eq!(eval_ok(quote! { 1.5 + 1.0 }), "2.5");
        assert_eq!(eval_ok(quote! { !true }), "false");
        assert_eq!(eval_ok(quote! { !0 }), "-1");
    }

    #[test]
    fn comparisons_and_logic_work() {
        assert_eq!(eval_ok(quote! { 1 < 2 }), "true");
        assert_eq!(eval_ok(quote! { 2.0 >= 2.5 }), "false");
        assert_eq!(eval_ok(quote! { "a" == "a" }), "true");
        assert_eq!(eval_ok(quote! { "a" != "b" }), "true");
        assert_eq!(eval_ok(quote! { "a" < "b" }), "true");
        assert_eq!(eval_ok(quote! { 'a' < 'b' }), "true");
        assert_eq!(eval_ok(quote! { true && false }), "false");
        assert_eq!(eval_ok(quote! { true || false }), "true");
        assert_eq!(eval_ok(quote! { true == false }), "false");
        assert_eq!(eval_ok(quote! { true != false }), "true");
    }

    #[test]
    fn mismatched_and_disallowed_comparisons_are_errors() {
        // Mixed-kind comparison
        let err = eval_err(quote! { "a" == 1 });
        assert!(err.contains("Unsupported operands"));
        // `bool` is not ordered
        let err = eval_err(quote! { true < false });
        assert!(err.contains("Unsupported operands"));
        // `bool` arithmetic
        let err = eval_err(quote! { true + false });
        assert!(err.contains("Unsupported operands"));
        // string arithmetic
        let err = eval_err(quote! { "a" + "b" });
        assert!(err.contains("Unsupported operands"));
    }

    #[test]
    fn casts_wrap_like_rust() {
        assert_eq!(eval_ok(quote! { 300 as u8 }), "44");
        assert_eq!(eval_ok(quote! { 5 as i64 }), "5");
    }

    #[test]
    fn concat_env_and_option_env_work() {
        assert_eq!(eval_ok(quote! { concat!("a", "b", "c") }), "abc");
        // `concat!` renders non-string values like rustc does
        assert_eq!(eval_ok(quote! { concat!("v", 2, ".", 1) }), "v2.1");
        // SAFETY: test binary serializes env access
        unsafe { std::env::set_var("RSMACK_TEST_VAR", "loaded") };
        assert_eq!(eval_ok(quote! { env!("RSMACK_TEST_VAR") }), "loaded");
        assert_eq!(
            eval_ok(quote! { option_env!("RSMACK_TEST_MISSING_VAR") }),
            ""
        );
        let err = eval_err(quote! { env!("RSMACK_TEST_MISSING_VAR") });
        assert!(err.contains("Failed to read env var"));
    }

    #[test]
    fn unsupported_expressions_report_a_reason() {
        assert!(eval_err(quote! { Some(3) }).contains("Unsupported const expression"));
        assert!(eval_err(quote! { vec![1] }).contains("Unsupported const macro"));
        assert!(eval_err(quote! { 1.0 as FaCollection }).contains("Unsupported cast target"));
    }

    #[test]
    fn unresolved_paths_report_a_reason() {
        let expr: syn::Expr = syn::parse2(quote! { MISSING }).expect("expr");
        let err = eval(&expr, &Consts::new(), &mut unresolved).expect_err("should fail");
        assert!(err.contains("Unresolved const ident `MISSING`"));
    }

    #[test]
    fn missing_ops_and_wrapping_casts_are_covered() {
        assert_eq!(eval_ok(quote! { 5 - 2 }), "3");
        assert_eq!(eval_ok(quote! { 1 <= 1 }), "true");
        assert_eq!(eval_ok(quote! { 2 > 1 }), "true");
        assert_eq!(eval_ok(quote! { -1.5 }), "-1.5");
        // two's-complement wrap below zero
        assert_eq!(eval_ok(quote! { 200 as i8 }), "-56");
        // negation of a non-numeric operand
        let err = eval_err(quote! { -true });
        assert!(err.contains("Unsupported operand for negation"));
        // dereference is not const-evaluable
        let err = eval_err(quote! { *MISSING });
        assert!(err.contains("Unsupported unary operator `*`"));
        // `!` on a non-numeric non-bool operand
        let err = eval_err(quote! { !"x" });
        assert!(err.contains("Unsupported operand for `!`"));
    }

    #[test]
    fn non_exhaustive_operator_variants_are_reported() {
        // `BinOp` is `#[non_exhaustive]`: hand-built hidden variants (e.g. `+=`, which
        // the parser would reject in a const expression) must yield an error.
        let assignment = syn::Expr::Binary(syn::ExprBinary {
            attrs: Vec::new(),
            left: Box::new(syn::parse2(quote! { 1 }).expect("lhs")),
            // syn token structs default their spans to the call site.
            op: syn::BinOp::AddAssign(syn::token::PlusEq {
                spans: [
                    proc_macro2::Span::call_site(),
                    proc_macro2::Span::call_site(),
                ],
            }),
            right: Box::new(syn::parse2(quote! { 2 }).expect("rhs")),
        });
        let err =
            eval(&assignment, &Consts::new(), &mut unresolved).expect_err("assignment op fails");
        assert!(err.contains("Unsupported binary operator"));
    }

    #[test]
    fn hand_built_edge_expressions_are_reported() {
        // An `ExprPath` with no segments cannot be produced by the parser but can be
        // built programmatically: it must yield an error, never a panic.
        let empty_path = syn::Expr::Path(syn::ExprPath {
            attrs: Vec::new(),
            qself: None,
            path: syn::Path {
                leading_colon: None,
                segments: syn::punctuated::Punctuated::new(),
            },
        });
        let err = eval(&empty_path, &Consts::new(), &mut unresolved).expect_err("empty path fails");
        assert!(err.contains("Unresolved const ident"));
        assert_eq!(
            normalize_path(&syn::Path {
                leading_colon: None,
                segments: syn::punctuated::Punctuated::new(),
            }),
            ""
        );

        // Same for a verbatim literal token.
        let verbatim_lit = syn::Expr::Lit(syn::ExprLit {
            attrs: Vec::new(),
            lit: syn::Lit::Verbatim(
                proc_macro2::Literal::from_str("\"verbatim\"").expect("literal"),
            ),
        });
        let err = eval(&verbatim_lit, &Consts::new(), &mut unresolved)
            .expect_err("verbatim literal fails");
        assert!(err.contains("Unsupported literal"));
    }

    #[test]
    fn float_ops_and_error_paths_are_covered() {
        assert_eq!(eval_ok(quote! { 2.5 - 1.0 }), "1.5");
        assert_eq!(eval_ok(quote! { 2.5 * 2.0 }), "5");
        assert_eq!(eval_ok(quote! { 2.5 / 0.5 }), "5");
        // integer-only operations on non-integer operands
        let err = eval_err(quote! { true & false });
        assert!(err.contains("Unsupported operands"));
        let err = eval_err(quote! { true | false });
        assert!(err.contains("Unsupported operands"));
        let err = eval_err(quote! { true ^ false });
        assert!(err.contains("Unsupported operands"));
        let err = eval_err(quote! { 7 % 0 });
        assert!(err.contains("Division by zero"));
        let err = eval_err(quote! { 7 / 0 });
        assert!(err.contains("Division by zero"));
        // shift errors: negative amount, oversized amount, non-integer operand
        let err = eval_err(quote! { 1 << -1 });
        assert!(err.contains("Shift amount out of range"));
        let err = eval_err(quote! { 1 << 200 });
        assert!(err.contains("Arithmetic overflow shifting"));
        let err = eval_err(quote! { 1.5 << 1 });
        assert!(err.contains("Unsupported operands"));
        // boolean operations on non-boolean operands
        let err = eval_err(quote! { 1 && 2 });
        assert!(err.contains("Unsupported operands"));
        let err = eval_err(quote! { true || 2 });
        assert!(err.contains("Unsupported operands"));
    }

    #[test]
    fn cast_targets_are_all_covered() {
        assert_eq!(eval_ok(quote! { 5 as i8 }), "5");
        assert_eq!(eval_ok(quote! { 5 as i16 }), "5");
        assert_eq!(eval_ok(quote! { 5 as i32 }), "5");
        assert_eq!(eval_ok(quote! { 5 as i64 }), "5");
        assert_eq!(eval_ok(quote! { 5 as i128 }), "5");
        assert_eq!(eval_ok(quote! { 5 as isize }), "5");
        assert_eq!(eval_ok(quote! { 5 as u8 }), "5");
        assert_eq!(eval_ok(quote! { 5 as u16 }), "5");
        assert_eq!(eval_ok(quote! { 5 as u32 }), "5");
        assert_eq!(eval_ok(quote! { 5 as u64 }), "5");
        assert_eq!(eval_ok(quote! { 5 as u128 }), "5");
        assert_eq!(eval_ok(quote! { 5 as usize }), "5");
        assert_eq!(eval_ok(quote! { -1 as u8 }), "255");
    }

    #[test]
    fn typed_casts_cover_bool_float_and_float_targets() {
        // bool as int (rust allows `true as u8`)
        assert_eq!(eval_ok(quote! { true as u8 }), "1");
        assert_eq!(eval_ok(quote! { false as i32 }), "0");
        // float -> int saturates like rustc
        assert_eq!(eval_ok(quote! { 300.7 as u8 }), "255");
        assert_eq!(eval_ok(quote! { -300.7 as i8 }), "-128");
        assert_eq!(eval_ok(quote! { 1.9 as i32 }), "1");
        assert_eq!(eval_ok(quote! { -1.9 as i32 }), "-1");
        // int/bool -> float
        assert_eq!(eval_ok(quote! { 3 as f64 }), "3");
        assert_eq!(eval_ok(quote! { true as f64 }), "1");
        assert_eq!(eval_ok(quote! { 2 as f32 }), "2");
        // float -> float narrows through f32
        assert_eq!(eval_ok(quote! { 1.5 as f32 }), "1.5");
        // unsupported cast operands
        let err = eval_err(quote! { "x" as u8 });
        assert!(err.contains("Unsupported cast operand"));
        let err = eval_err(quote! { "x" as f64 });
        assert!(err.contains("Unsupported cast operand"));
        // b"ab" as u8 is not a rust cast either
        let err = eval_err(quote! { b"ab" as u8 });
        assert!(err.contains("Unsupported cast operand"));
    }

    #[test]
    fn macro_evaluation_error_paths_are_covered() {
        let err = eval_err(quote! { concat!(1 +) });
        assert!(err.contains("Failed to parse the arguments of `concat!`"));
        let err = eval_err(quote! { concat!(1, MISSING) });
        assert!(err.contains("Unresolved const ident `MISSING`"));
        let err = eval_err(quote! { env!() });
        assert!(err.contains("expects a single string literal key"));
        // integer literal out of range
        let err = eval_err(quote! { 99999999999999999999999999999999999999999999999999 });
        assert!(err.contains("out of range"));
    }

    #[test]
    fn overflow_reports_a_reason() {
        let err = eval_err(quote! { 170141183460469231731687303715884105727 + 1 });
        assert!(err.contains("Arithmetic overflow"), "got: {err}");
        // The bare `i128::MIN` magnitude literal is out of the i128 range (rustc
        // special-cases the negated form; the mini evaluator reports the literal).
        let err = eval_err(quote! { -170141183460469231731687303715884105728 - 1 });
        assert!(err.contains("out of range"), "got: {err}");
    }

    #[test]
    fn env_macro_requires_string_key() {
        let expr: syn::Expr = syn::parse2(quote! { env!(1) }).expect("expr");
        let err = eval(&expr, &Consts::new(), &mut unresolved).expect_err("should fail");
        assert!(err.contains("expects a single string literal key"));
    }

    #[test]
    fn match_key_resolves_exact_unique_and_ambiguous() {
        let keys: Vec<String> = vec![
            "nested::DEEP".to_string(),
            "other::DEEP".to_string(),
            "plain".to_string(),
        ];
        // Exact match wins even when bare names collide.
        assert_eq!(
            match_key("nested::DEEP", keys.iter()).expect("match"),
            Ok("nested::DEEP".to_string())
        );
        // Unique bare name
        assert_eq!(
            match_key("plain", keys.iter()).expect("match"),
            Ok("plain".to_string())
        );
        // Ambiguous bare name
        let err = match_key("DEEP", keys.iter())
            .expect("ambiguous names match")
            .expect_err("ambiguous");
        assert!(err.contains("Ambiguous const `DEEP`"));
        assert!(err.contains("`nested::DEEP`"));
        // Missing
        assert!(match_key("nope", keys.iter()).is_none());
        // Empty key list
        assert!(match_key("anything", Vec::<String>::new().iter()).is_none());
    }
}
