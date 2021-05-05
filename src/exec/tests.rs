use crate::math::Scalar;
use crate::exec::{parse, Context, ExecResult, Value};

fn eval_exp(input: &str) -> ExecResult<Value>
{
    let expressions = parse(input)?;
    assert_eq!(expressions.len(), 1);

    expressions[0].evaluate(&mut Context::new())
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
}
