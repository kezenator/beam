use crate::math::Scalar;
use crate::exec::{parse, Context, ExecResult, Value};

fn eval_exp(input: &str) -> ExecResult<Value>
{
    let expressions = parse(input)?;

    let mut context = Context::new();

    expressions.iter()
        .map(|e| e.evaluate(&mut context))
        .try_fold(Value::new_void(), |_, v| v)
}

fn check_parse_error(input: &str)
{
    assert!(parse(input).is_err());
}

fn check_scalar(input: &str, val: Scalar)
{
    assert_eq!(eval_exp(input).and_then(|val| val.into_scalar()), Ok(val));
}

#[test]
fn test_parse()
{
    check_parse_error("add(");
    check_parse_error("1+");
}

#[test]
fn test_uint()
{
    check_scalar("123", 123.0);
    check_scalar(" ( 123)", 123.0);
    check_scalar("1 + 2", 3.0);
    check_scalar("1 + 2 * 3", 7.0);
    check_scalar("(1 + 2) * 3", 9.0);
    check_scalar("1 + (2 * 3)", 7.0);
    check_scalar("1.5 + 2.25", 3.75);
    check_scalar("-1.5", -1.5);
    check_scalar("-1.5 + -2.0", -3.5);

    check_scalar("add(1, mul(2, 3))", 7.0);
    check_scalar("add{ lhs: 1, rhs: mul{ lhs: 2, rhs: 3 }}", 7.0);

    check_scalar("let x = 5; x", 5.0);
    check_scalar("let x = 5; let y = 3; x + y", 8.0);
    check_scalar("if (true) { 12 } else { 13 }", 12.0);
    check_scalar("if (false) { 14 } else { 15 }", 15.0);
    check_scalar("if (1 == 1) { 16 } else { 17 }", 16.0);
    check_scalar("if (1 == 2) { 18 } else { 19 }", 19.0);
    check_scalar("function two() { 2 } two()", 2.0);
    check_scalar("function fib(n) { if (n == 1) { 1 } else { n * fib(n - 1) } } fib(1)", 1.0);
    check_scalar("function fib(n) { if (n == 1) { 1 } else { n * fib(n - 1) } } fib(2)", 2.0);
    check_scalar("function fib(n) { if (n == 1) { 1 } else { n * fib(n - 1) } } fib(3)", 6.0);
    check_scalar("function fib(n) { if (n == 1) { 1 } else { n * fib(n - 1) } } fib(4)", 24.0);
}
