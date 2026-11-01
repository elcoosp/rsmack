//! Mini constant-expression evaluator for `edoc`.
//!
//! Evaluates the subset of const expressions users actually put in documentation
//! constants: literals, arithmetic/bitwise/shift/comparison operations, casts,
//! `concat!`, `env!`, `option_env!` and references to other constants of the same
//! file. Anything outside this subset is reported as a collected error instead of
//! aborting the whole expansion.
//!
//! This mirrors what rustc's own `min_const_eval` accepts, without pulling a full
//! interpreter dependency.

use std::collections::HashMap;

use syn::parse::Parser;

use quote::ToTokens;

/// Resolved constants of a source file, keyed by const name.
pub(crate) type Consts = HashMap<String, String>;

/// Evaluation result: the documentation string, or a user-facing reason.
pub(crate) type EvalResult = Result<String, String>;

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

/// Evaluate a path expression: a resolved const lookup or a lazy const reference.
fn eval_path(
    path: &syn::ExprPath,
    consts: &Consts,
    resolve_const: &mut dyn FnMut(&str) -> EvalResult,
) -> EvalResult {
    let Some(ident) = path.path.segments.last().map(|s| &s.ident) else {
        return Err("Empty path expression".to_string());
    };
    let ident = ident.to_string();
    if let Some(value) = consts.get(&ident) {
        return Ok(value.clone());
    }
    resolve_const(&ident)
}

/// Evaluate a literal into its string representation.
fn eval_lit(lit: &syn::Lit) -> EvalResult {
    match lit {
        syn::Lit::Str(lit) => Ok(lit.value()),
        syn::Lit::Bool(lit) => Ok(lit.value().to_string()),
        syn::Lit::ByteStr(lit) => Ok(format!("{:?}", lit.value())),
        syn::Lit::Int(lit) => lit
            .base10_parse::<i128>()
            .map(|value| value.to_string())
            .map_err(|_| format!("Integer literal `{lit}` out of range")),
        // `f64` parsing saturates to infinity instead of failing, so there is no
        // reachable error branch here.
        syn::Lit::Float(lit) => Ok(lit
            .base10_parse::<f64>()
            .unwrap_or(f64::INFINITY)
            .to_string()),
        syn::Lit::Char(lit) => Ok(lit.value().to_string()),
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
    match operator.as_str() {
        "-" => {
            let operand = eval(&unary.expr, consts, resolve_const)?;
            negate(&operand)
        }
        "!" => {
            let operand = eval(&unary.expr, consts, resolve_const)?;
            match operand.as_str() {
                "true" => Ok("false".to_string()),
                "false" => Ok("true".to_string()),
                other => parse_int(other).map(|v| (!v).to_string()),
            }
        }
        // `*` (dereference) and any future unary operator
        other => Err(format!("Unsupported unary operator `{other}`")),
    }
}

/// Negate an integer or float operand rendered as a string.
fn negate(operand: &str) -> EvalResult {
    if let Ok(value) = operand.parse::<i128>() {
        return Ok((-value).to_string());
    }
    operand
        .parse::<f64>()
        .map(|value| (-value).to_string())
        .map_err(|_| format!("Unsupported operand for negation: `{operand}`"))
}

/// Parse an operand string as an integer.
fn parse_int(operand: &str) -> Result<i128, String> {
    operand
        .parse::<i128>()
        .map_err(|_| format!("Unsupported integer operand: `{operand}`"))
}

/// Evaluate a binary expression (arithmetic, bitwise, shifts, logic, comparisons).
fn eval_binary(
    binary: &syn::ExprBinary,
    consts: &Consts,
    resolve_const: &mut dyn FnMut(&str) -> EvalResult,
) -> EvalResult {
    let lhs = eval(&binary.left, consts, resolve_const)?;
    let rhs = eval(&binary.right, consts, resolve_const)?;
    match &binary.op {
        syn::BinOp::Add(_) => arith(&lhs, &rhs, i128::checked_add, |a, b| a + b),
        syn::BinOp::Sub(_) => arith(&lhs, &rhs, i128::checked_sub, |a, b| a - b),
        syn::BinOp::Mul(_) => arith(&lhs, &rhs, i128::checked_mul, |a, b| a * b),
        syn::BinOp::Div(_) => arith(&lhs, &rhs, i128::checked_div, |a, b| a / b),
        syn::BinOp::Rem(_) => int_op(&lhs, &rhs, i128::checked_rem)
            .ok_or_else(|| format!("Failed to compute the remainder of `{lhs}` and `{rhs}`")),
        syn::BinOp::BitAnd(_) => int_op(&lhs, &rhs, |a, b| Some(a & b))
            .ok_or_else(|| format!("Failed to apply bitwise and to `{lhs}` and `{rhs}`")),
        syn::BinOp::BitOr(_) => int_op(&lhs, &rhs, |a, b| Some(a | b))
            .ok_or_else(|| format!("Failed to apply bitwise or to `{lhs}` and `{rhs}`")),
        syn::BinOp::BitXor(_) => int_op(&lhs, &rhs, |a, b| Some(a ^ b))
            .ok_or_else(|| format!("Failed to apply bitwise xor to `{lhs}` and `{rhs}`")),
        syn::BinOp::Shl(_) => shift(&lhs, &rhs, i128::checked_shl),
        syn::BinOp::Shr(_) => shift(&lhs, &rhs, i128::checked_shr),
        syn::BinOp::And(_) => bool_op(&lhs, &rhs, |a, b| a && b),
        syn::BinOp::Or(_) => bool_op(&lhs, &rhs, |a, b| a || b),
        syn::BinOp::Eq(_) => Ok(compare(&lhs, &rhs, std::cmp::Ordering::is_eq)),
        syn::BinOp::Ne(_) => Ok(compare(&lhs, &rhs, std::cmp::Ordering::is_ne)),
        syn::BinOp::Lt(_) => Ok(compare(&lhs, &rhs, std::cmp::Ordering::is_lt)),
        syn::BinOp::Le(_) => Ok(compare(&lhs, &rhs, std::cmp::Ordering::is_le)),
        syn::BinOp::Gt(_) => Ok(compare(&lhs, &rhs, std::cmp::Ordering::is_gt)),
        syn::BinOp::Ge(_) => Ok(compare(&lhs, &rhs, std::cmp::Ordering::is_ge)),
        // `BinOp` is `#[non_exhaustive]`: assignment and future operators never form
        // const expressions.
        other => Err(format!(
            "Unsupported binary operator `{}`",
            other.to_token_stream()
        )),
    }
}

/// Apply a checked integer operation, falling back to floats.
fn arith(
    lhs: &str,
    rhs: &str,
    int_op: fn(i128, i128) -> Option<i128>,
    float_op: fn(f64, f64) -> f64,
) -> EvalResult {
    if let (Ok(a), Ok(b)) = (lhs.parse::<i128>(), rhs.parse::<i128>()) {
        return int_op(a, b)
            .map(|value| value.to_string())
            .ok_or_else(|| format!("Arithmetic overflow computing `{lhs}` and `{rhs}`"));
    }
    let (a, b) = (
        lhs.parse::<f64>()
            .map_err(|_| format!("Unsupported arithmetic operand: `{lhs}`"))?,
        rhs.parse::<f64>()
            .map_err(|_| format!("Unsupported arithmetic operand: `{rhs}`"))?,
    );
    Ok(float_op(a, b).to_string())
}

/// Apply a checked integer-only operation.
fn int_op(lhs: &str, rhs: &str, op: fn(i128, i128) -> Option<i128>) -> Option<String> {
    let (a, b) = (lhs.parse::<i128>().ok()?, rhs.parse::<i128>().ok()?);
    op(a, b).map(|value| value.to_string())
}

/// Apply a shift operation with shift-amount validation.
fn shift(lhs: &str, rhs: &str, op: fn(i128, u32) -> Option<i128>) -> EvalResult {
    let a = parse_int(lhs)?;
    let b = u32::try_from(parse_int(rhs)?)
        .map_err(|_| format!("Shift amount out of range: `{rhs}`"))?;
    op(a, b)
        .map(|value| value.to_string())
        .ok_or_else(|| format!("Arithmetic overflow shifting `{lhs}` by `{rhs}`"))
}

/// Apply a boolean operation.
fn bool_op(lhs: &str, rhs: &str, op: fn(bool, bool) -> bool) -> EvalResult {
    let a = lhs
        .parse::<bool>()
        .map_err(|_| format!("Unsupported boolean operand: `{lhs}`"))?;
    let b = rhs
        .parse::<bool>()
        .map_err(|_| format!("Unsupported boolean operand: `{rhs}`"))?;
    Ok(op(a, b).to_string())
}

/// Apply an ordering comparison between two scalar operands.
fn compare(lhs: &str, rhs: &str, keep: fn(std::cmp::Ordering) -> bool) -> String {
    let ordering = lhs
        .parse::<i128>()
        .ok()
        .zip(rhs.parse::<i128>().ok())
        .map(|(a, b)| a.cmp(&b))
        .or_else(|| {
            lhs.parse::<f64>()
                .ok()
                .zip(rhs.parse::<f64>().ok())
                .map(|(a, b)| a.total_cmp(&b))
        })
        .unwrap_or_else(|| lhs.cmp(rhs));
    keep(ordering).to_string()
}

/// Evaluate a cast expression (`expr as usize`), mirroring integer `as` wrapping.
fn eval_cast(
    cast: &syn::ExprCast,
    consts: &Consts,
    resolve_const: &mut dyn FnMut(&str) -> EvalResult,
) -> EvalResult {
    let target = cast.ty.to_token_stream().to_string().replace(' ', "");
    let value = eval(&cast.expr, consts, resolve_const)?;
    let int = parse_int(&value)?;
    let (bits, signed) = match target.as_str() {
        "i8" => (8, true),
        "i16" => (16, true),
        "i32" => (32, true),
        "i64" => (64, true),
        "i128" | "isize" | "u128" | "usize" => return Ok(int.to_string()),
        "u8" => (8, false),
        "u16" => (16, false),
        "u32" => (32, false),
        "u64" => (64, false),
        other => return Err(format!("Unsupported cast target `{other}`")),
    };
    // Two's-complement wrapping identical to rust's integer `as` casts, expressed with
    // modular arithmetic so no truncating casts are involved. `bits <= 64` so the shift
    // cannot overflow.
    let modulus = 1i128 << bits;
    let masked = ((int % modulus) + modulus) % modulus;
    let casted = if signed && masked >= modulus / 2 {
        masked - modulus
    } else {
        masked
    };
    Ok(casted.to_string())
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
                joined.push_str(&eval(arg, consts, resolve_const)?);
            }
            Ok(joined)
        }
        "env" => {
            let key = env_key(&args, &name)?;
            std::env::var(&key)
                .map_err(|_| format!("Failed to read env var `{key}`: not set at compile time"))
        }
        "option_env" => {
            let key = env_key(&args, &name)?;
            Ok(std::env::var(&key).unwrap_or_default())
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
        eval(&expr, &Consts::new(), &mut unresolved).expect("evaluation should succeed")
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
        assert_eq!(eval_ok(quote! { true && false }), "false");
        assert_eq!(eval_ok(quote! { true || false }), "true");
    }

    #[test]
    fn casts_wrap_like_rust() {
        assert_eq!(eval_ok(quote! { 300 as u8 }), "44");
        assert_eq!(eval_ok(quote! { 5 as i64 }), "5");
    }

    #[test]
    fn concat_env_and_option_env_work() {
        assert_eq!(eval_ok(quote! { concat!("a", "b", "c") }), "abc");
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
    }

    #[test]
    fn non_exhaustive_operator_variants_are_reported() {
        // `BinOp` is `#[non_exhaustive]`: hand-built hidden variants (e.g. `+=`, which
        // the parser would reject in a const expression) must yield an error.
        let assignment = syn::Expr::Binary(syn::ExprBinary {
            attrs: Vec::new(),
            left: Box::new(syn::parse2(quote! { 1 }).expect("lhs")),
            // syn token types default their spans to the call site.
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
        assert!(err.contains("Empty path expression"));

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
        // non-numeric operands fall through to the arithmetic error path
        let err = eval_err(quote! { "a" + "b" });
        assert!(err.contains("Unsupported arithmetic operand"));
        let err = eval_err(quote! { 1.5 + "a" });
        assert!(err.contains("Unsupported arithmetic operand"));
        // integer-only operations on non-integer operands
        let err = eval_err(quote! { true & false });
        assert!(err.contains("Failed to apply bitwise and"));
        let err = eval_err(quote! { true | false });
        assert!(err.contains("Failed to apply bitwise or"));
        let err = eval_err(quote! { true ^ false });
        assert!(err.contains("Failed to apply bitwise xor"));
        let err = eval_err(quote! { 7 % 0 });
        assert!(err.contains("Failed to compute the remainder"));
        // shift errors: negative amount, oversized amount, non-integer operand
        let err = eval_err(quote! { 1 << -1 });
        assert!(err.contains("Shift amount out of range"));
        let err = eval_err(quote! { 1 << 200 });
        assert!(err.contains("Arithmetic overflow shifting"));
        let err = eval_err(quote! { 1.5 << 1 });
        assert!(err.contains("Unsupported integer operand"));
        // boolean operations on non-boolean operands
        let err = eval_err(quote! { 1 && 2 });
        assert!(err.contains("Unsupported boolean operand"));
        let err = eval_err(quote! { true || 2 });
        assert!(err.contains("Unsupported boolean operand"));
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
    }

    #[test]
    fn env_macro_requires_string_key() {
        let expr: syn::Expr = syn::parse2(quote! { env!(1) }).expect("expr");
        let err = eval(&expr, &Consts::new(), &mut unresolved).expect_err("should fail");
        assert!(err.contains("expects a single string literal key"));
    }
}
