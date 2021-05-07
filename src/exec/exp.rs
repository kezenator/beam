use crate::vec::Vec3;
use crate::exec::{ActualArgumentExpressions, Context, ExecError, ExecResult, Function, SourceLocation, Value};

//pub mod core;

#[derive(Clone)]
pub enum Expression
{
    Constant{ value: Value },
    Vector{ source: SourceLocation, expressions: Vec<Box<Expression>> },
    Function{ source: SourceLocation, name: String, formal_arguments: Vec<String>, expression: Box<Expression> },
    ReadNamedVar{ source: SourceLocation, name: String },
    WriteNamedVar{ name: String, expression: Box<Expression> },
    Call{ call_site: SourceLocation, function: Box<Expression>, arguments: ActualArgumentExpressions },
    Block { expressions: Vec<Box<Expression>> },
    If{ conditions: Vec<(Box<Expression>, Box<Expression>)>, alternative: Option<Box<Expression>> },
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

    pub fn new_function(source: SourceLocation, name: String, formal_arguments: Vec<String>, expression: Box<Expression>) -> Box<Expression>
    {
        Box::new(Expression::Function{ source, name, formal_arguments, expression })
    }

    pub fn new_read_named_var(source: SourceLocation, name: String) -> Box<Expression>
    {
        Box::new(Expression::ReadNamedVar{ source, name })
    }

    pub fn new_write_named_var(name: String, expression: Box<Expression>) -> Box<Expression>
    {
        Box::new(Expression::WriteNamedVar{ name, expression })
    }

    pub fn new_call(call_site: SourceLocation, function: Box<Expression>, arguments: ActualArgumentExpressions) -> Box<Expression>
    {
        Box::new(Expression::Call{ call_site, function, arguments })
    }

    pub fn new_block(expressions: Vec<Box<Expression>>) -> Box<Expression>
    {
        Box::new(Expression::Block{expressions})
    }

    pub fn new_if(conditions: Vec<(Box<Expression>, Box<Expression>)>, alternative: Option<Box<Expression>>) -> Box<Expression>
    {
        Box::new(Expression::If{ conditions, alternative })
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
            Expression::Function{ source, name, formal_arguments, expression } =>
            {
                Ok(Value::new_function(Function::new_expression(source.clone(), name.clone(), formal_arguments.clone(), context, (*expression).clone())))
            },
            Expression::ReadNamedVar{ source, name } =>
            {
                context.get_var_named(*source, name)
            },
            Expression::WriteNamedVar{ name, expression } =>
            {
                let value = expression.evaluate(context)?;

                context.set_var_named(name, value.clone());

                Ok(value)
            },
            Expression::Call{ call_site, function, arguments } =>
            {
                let function = function.evaluate(context)?.into_function()?;

                let arguments = arguments.evaluate(context)?;

                function.call(context, *call_site, arguments)
            },
            Expression::Block{ expressions } =>
            {
                let mut block_context = context.sub_block();

                expressions.iter()
                    .map(|e| e.evaluate(&mut block_context))
                    .try_fold(Value::new_void(), |_, v| v)
            },
            Expression::If{ conditions, alternative } =>
            {
                let mut block_context = context.sub_block();

                for (cond, exp) in conditions.iter()
                {
                    if cond.evaluate(&mut block_context)?.into_bool()?
                    {
                        return exp.evaluate(&mut block_context);
                    }
                }

                if let Some(alternative) = alternative
                {
                    return alternative.evaluate(&mut block_context);
                }

                // None succeeded

                Ok(Value::new_void())
            }
        }
    }
}
