pub mod event_targets;

/// Logs an expression's string representation and returns the original expression. The format string can also be customised in the second overload (with custom arguments)
///
/// ### Examples:
///
/// `let calculation = log_expr!(do_maths())` prints ```run `do_maths()` ``` and returns whatever value `do_maths()` returned (the expression is directly injected into the generated code, so the expression can return nothing).
/// This form simply calls [log_expr] with `$expression_name=expr` and ```$format_and_args="run `{expr}` ```
///
/// ```let add_two_numbers = log_expr!(f64::from(5+5) * 3.21f64, "Adding numbers: {expr}");```
///
/// prints
///
/// ```Adding numbers: f64::from(5 + 5) * 3.21f64```
#[macro_export]
macro_rules! log_expr {
    ($expression:expr) => {
        log_expr!($expression, expr, "run `{expr}`")
    };
    ($expression:expr, $format_and_args:tt) => {{
        let value = $expression;
        tracing::trace!($format_and_args, expr = stringify!($expression));
        value
    }};
}

#[macro_export]
macro_rules! log_variable {
    ($variable:ident) => {
        tracing::trace!("{}={}", stringify!($variable), $variable);
    };($variable:ident:?) => {
        tracing::trace!("{}={:?}", stringify!($variable), $variable);
    };($variable:ident:#?) => {
        tracing::trace!("{}={:#?}", stringify!($variable), $variable);
    };
}

/// Same as [log_expr] but includes the value of the evaluated expression
///
/// When using a custom format and arguments, the expression is `expr` and the value is `val`
#[macro_export]
macro_rules! log_expr_val {
    ($expression:expr) => {
        log_expr_val!($expression, "eval `{expr}` => {val}")
    };
    (?$expression:expr) => {
        log_expr_val!($expression, "eval `{expr}` => {val:?}")
    }; (#?$expression:expr) => {
        log_expr_val!($expression, "eval `{expr}` => {val:#?}")
    };
    ($expression:expr, $format_and_args:tt) => {{
        let val = $expression;
        tracing::trace!(
            $format_and_args,
            expr = stringify!($expression),
            val = val
        );
        val
    }};
}
