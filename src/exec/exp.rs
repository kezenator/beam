use crate::exec::{ActualArgumentExpressions, Context, ExecResult, SourceLocation, Value};

//pub mod core;

pub enum Expression
{
    Constant{ value: Value },
    ReadNamedVar{ source: SourceLocation, name: String },
    Call{ call_site: SourceLocation, function: Box<Expression>, arguments: ActualArgumentExpressions },
}

impl Expression
{
    pub fn new_constant(value: Value) -> Box<Expression>
    {
        Box::new(Expression::Constant{ value })
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
            Expression::ReadNamedVar{ source, name } =>
            {
                context.get_named_var(*source, name)
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
