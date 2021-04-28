use crate::exec::{parse, Context, ExecResult, Value};

fn eval_exp(input: &str) -> ExecResult<Value>
{
    parse(input).and_then(|exp| exp.evaluate(&mut Context::new()))
}

fn check_parse_error(input: &str)
{
    assert!(parse(input).is_err());
}
fn check_uint(input: &str, val: i64)
{
    assert_eq!(eval_exp(input).and_then(|val| val.into_int()), Ok(val));
}

#[test]
fn test_parse()
{
    check_parse_error("");
}

#[test]
fn test_uint()
{
    check_uint("123", 123);
    check_uint(" ( 123)", 123);
    check_uint("1 + 2", 3);
    check_uint("1 + 2 * 3", 7);
    check_uint("(1 + 2) * 3", 9);
    check_uint("1 + (2 * 3)", 7);

    check_uint("add(1, mul(2, 3))", 7);
    check_uint("add{ lhs: 1, rhs: mul{ lhs: 2, rhs: 3 }}", 7);
}
