use std::rc::Rc;
use std::collections::HashMap;
use crate::exec::{Context, ExecResult, Expression, SourceLocation, Value};

#[derive(Clone)]
pub enum ActualArgumentExpressions
{
    Positional(Vec<Box<Expression>>),
    Named(Vec<(String, Box<Expression>)>),
}

impl ActualArgumentExpressions
{
    pub fn evaluate(&self, context: &mut Context) -> ExecResult<ActualArguments>
    {
        match self
        {
            ActualArgumentExpressions::Positional(expressions) =>
            {
                let mut values = Vec::new();

                for exp in expressions.iter()
                {
                    values.push(exp.evaluate(context)?);
                }

                Ok(ActualArguments::Positional(values))
            },
            ActualArgumentExpressions::Named(expressions) =>
            {
                let mut values = HashMap::new();

                for (name, exp) in expressions.iter()
                {
                    values.insert(name.clone(), exp.evaluate(context)?);
                }

                Ok(ActualArguments::Named(values))
            },
        }
    }
}

pub enum ActualArguments
{
    Positional(Vec<Value>),
    Named(HashMap<String, Value>),
}

type InbuiltFunction = dyn Fn(&mut Context) -> ExecResult<Value>;

enum FunctionCode
{
    Inbuilt(Box<InbuiltFunction>),
    Expression(Box<Expression>),
}

struct FunctionData
{
    source: SourceLocation,
    name: String,
    formal_arguments: Vec<String>,
    parent_context: Context,
    code: FunctionCode,
}

#[derive(Clone)]
pub struct Function
{
    data: Rc<FunctionData>,
}

impl Function
{
    pub fn new_inbuilt<F>(name: String, formal_arguments: Vec<String>, context: &mut Context, function: F) -> Function
        where F: Fn(&mut Context) -> ExecResult<Value> + Sized + 'static
    {
        let code = FunctionCode::Inbuilt(Box::new(function));

        let data = Rc::new(FunctionData{
            source: SourceLocation::inbuilt(),
            name,
            formal_arguments,
            parent_context: context.clone(),
            code,
        });

        Function{ data }
    }

    pub fn new_expression(source: SourceLocation, name: String, formal_arguments: Vec<String>, context: &mut Context, expression: Box<Expression>) -> Function
    {
        let code = FunctionCode::Expression(expression);

        let data = Rc::new(FunctionData{
            source,
            name,
            formal_arguments,
            parent_context: context.clone(),
            code,
        });

        Function{ data }
    }

    pub fn get_name(&self) -> &str
    {
        &self.data.name
    }

    pub fn get_source_location(&self) -> SourceLocation
    {
        self.data.source
    }

    pub fn call(&self, _context: &mut Context, call_site: SourceLocation, actual_arguments: ActualArguments) -> ExecResult<Value>
    {
        let mut context = self.data.parent_context.sub_frame(call_site, &self.data.formal_arguments, actual_arguments);

        match &self.data.code
        {
            FunctionCode::Inbuilt(inbuilt) => inbuilt(&mut context),
            FunctionCode::Expression(expression) => expression.evaluate(&mut context),
        }
    }
}