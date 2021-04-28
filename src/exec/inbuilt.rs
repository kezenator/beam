use crate::exec::{Context, Function, Value};

pub fn get_inbuilt_functions() -> Vec<Function>
{
    let mut result = Vec::new();

    for name in ["+", "add"].iter()
    {
        result.push(Function::new_inbuilt(
            name.to_string(),
            vec!["lhs".to_owned(), "rhs".to_owned()],
            |context: &mut Context|
            {
                let lhs = context.get_positional_var(0)?.into_int()?;
                let rhs = context.get_positional_var(1)?.into_int()?;

                Ok(Value::new_int(context.get_call_site(), lhs + rhs))
            }
        ));
    }

    for name in ["*", "mul"].iter()
    {
        result.push(Function::new_inbuilt(
            name.to_string(),
            vec!["lhs".to_owned(), "rhs".to_owned()],
            |context: &mut Context|
            {
                let lhs = context.get_positional_var(0)?.into_int()?;
                let rhs = context.get_positional_var(1)?.into_int()?;

                Ok(Value::new_int(context.get_call_site(), lhs * rhs))
            }
        ));
    }

    result
}