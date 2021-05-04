use std::rc::Rc;
use std::collections::HashMap;
use crate::exec::{ActualArguments, ExecError, ExecResult, SourceLocation, Value};

pub struct Context
{
    frame: Rc<Frame>,
}

impl Context
{
    pub fn new() -> Context
    {
        let mut frame = Frame::new();

        for func in crate::exec::inbuilt::get_inbuilt_functions()
        {
            frame.named.insert(func.get_name().to_owned(), Value::new_function(func));
        }

        Context
        {
            frame: Rc::new(frame),
        }
    }

    pub fn get_call_site(&self) -> SourceLocation
    {
        self.frame.call_site
    }

    pub fn get_var_named(&mut self, source: SourceLocation, name: &str) -> ExecResult<Value>
    {
        match self.frame.get_var_named(name)
        {
            Some(val) => Ok(val),
            None => Err(ExecError::new(source, format!("Undefined variable \"{}\"", name))),
        }
    }

    pub fn get_param_named(&mut self, name: &str) -> ExecResult<Value>
    {
        match self.frame.get_param_named(name)
        {
            Some(val) => Ok(val),
            None => Err(ExecError::new(self.frame.call_site, format!("No named parameter \"{}\"", name))),
        }
    }

    pub fn get_param_positional(&mut self, index: usize) -> ExecResult<Value>
    {
        match self.frame.get_param_positional(index)
        {
            Some(val) => Ok(val),
            None => Err(ExecError::new(self.frame.call_site, format!("No positional parameter #{}", index + 1))),
        }
    }

    pub fn new_frame(&self, call_site: SourceLocation, formal_arguments: &Vec<String>, actual_arguments: ActualArguments) -> Context
    {
        Context
        {
            frame: Rc::new(Frame::new_sub(self.frame.clone(), call_site, formal_arguments, actual_arguments)),
        }
    }
}

struct Frame
{
    parent: Option<Rc<Frame>>,
    call_site: SourceLocation,
    positional: Vec<Value>,
    named: HashMap<String, Value>,
}

impl Frame
{
    fn new() -> Self
    {
        Frame
        {
            parent: None,
            call_site: SourceLocation::inbuilt(),
            positional: Vec::new(),
            named: HashMap::new(),
        }
    }

    fn new_sub(parent: Rc<Frame>, call_site: SourceLocation, formal_arguments: &Vec<String>, actual_arguments: ActualArguments) -> Self
    {
        let mut result = Frame
        {
            parent: Some(parent),
            call_site: call_site,
            positional: Vec::new(),
            named: HashMap::new(),
        };

        match actual_arguments
        {
            ActualArguments::Positional(vec) =>
            {
                for (index, formal) in formal_arguments.iter().enumerate()
                {
                    result.named.insert(formal.clone(), vec[index].clone());
                }

                result.positional = vec;
            },
            ActualArguments::Named(map) =>
            {
                result.named = map;

                for formal in formal_arguments
                {
                    if let Some(val) = result.named.get(formal)
                    {
                        result.positional.push(val.clone());
                    }
                    else
                    {
                        break;
                    }
                }
            },
        }

        result
    }

    fn get_var_named(&self, name: &str) -> Option<Value>
    {
        if let Some(here) = self.named.get(name)
        {
            return Some((*here).clone())
        }

        if let Some(parent) = &self.parent
        {
            return parent.get_var_named(name);
        }

        None
    }

    fn get_param_named(&self, name: &str) -> Option<Value>
    {
        if let Some(here) = self.named.get(name)
        {
            return Some((*here).clone())
        }

        None
    }

    fn get_param_positional(&self, index: usize) -> Option<Value>
    {
        if index < self.positional.len()
        {
            return Some(self.positional[index].clone());
        }
        None
    }
}
