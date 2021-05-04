use crate::vec::Vec3;
use crate::exec::{ActualArgumentExpressions, Context, ExecError, ExecResult, SourceLocation, Value};

//pub mod core;

pub enum Expression
{
    Constant{ value: Value },
    Vector{ source: SourceLocation, expressions: Vec<Box<Expression>> },
    ReadNamedVar{ source: SourceLocation, name: String },
    Call{ call_site: SourceLocation, function: Box<Expression>, arguments: ActualArgumentExpressions },
}

impl Expression
{
    pub fn new_constant(value: Value) -> Box<Expression>
    {
        Box::new(Expression::Constant{ value })
    }

    pub fn new_vector(source: SourceLocation, expressions: Vec<Box<Expression>>) -> Box<Expression>
    {
        Box::new(Expression::Vector{ source, expressions })
    }

    pub fn new_read_named_var(source: SourceLocation, name: String) -> Box<Expression>
    {
        Box::new(Expression::ReadNamedVar{ source, name })
    }

    pub fn new_call(call_site: SourceLocation, function: Box<Expression>, arguments: ActualArgumentExpressions) -> Box<Expression>
    {
        Box::new(Expression::Call{ call_site, function, arguments })
    }

    pub fn evaluate(&self, context: &mut Context) -> ExecResult<Value>
    {
        match self
        {
            Expression::Constant{ value } =>
            {
                Ok(value.clone())
            },
            Expression::Vector{ source, expressions } =>
            {
                if expressions.len() != 3
                {
                    return Err(ExecError::new(*source, "Vector expressions only support three scalar elements"));
                }

                let x = expressions[0].evaluate(context)?.into_scalar()?;
                let y = expressions[1].evaluate(context)?.into_scalar()?;
                let z = expressions[2].evaluate(context)?.into_scalar()?;

                Ok(Value::new_vec3(*source, Vec3::new(x, y, z)))
            },
            Expression::ReadNamedVar{ source, name } =>
            {
                context.get_var_named(*source, name)
            },
            Expression::Call{ call_site, function, arguments } =>
            {
                let function = function.evaluate(context)?.into_function()?;

                let arguments = arguments.evaluate(context)?;

                function.call(context, *call_site, arguments)
            },
        }
    }
}
