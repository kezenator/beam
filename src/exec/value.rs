use crate::exec::{ExecError, ExecResult, Function, SourceLocation};

#[derive(Clone)]
pub enum ValueData
{
    Integer(i64),
    Function(Function),
}

#[derive(Clone)]
pub struct Value
{
    source: SourceLocation,
    data: ValueData,
}

impl Value
{
    pub fn new_int(source: SourceLocation, val: i64) -> Value
    {
        Value { source, data: ValueData::Integer(val) }
    }

    pub fn new_function(function: Function) -> Value
    {
        Value { source: function.get_source_location(), data: ValueData::Function(function), }
    }

    pub fn source_location(&self) -> SourceLocation
    {
        self.source
    }
    
    pub fn into_int(self) -> ExecResult<i64>
    {
        match self.data
        {
            ValueData::Integer(val) => Ok(val),
            _ => Err(self.type_error("Integer")),
        }
    }

    pub fn into_function(self) -> ExecResult<Function>
    {
        match self.data
        {
            ValueData::Function(func) => Ok(func),
            _ => Err(self.type_error("Function")),
        }
    }

    fn type_error(&self, expected: &str) -> ExecError
    {
        ExecError::new(self.source, format!("Expected {}", expected))
    }
}
