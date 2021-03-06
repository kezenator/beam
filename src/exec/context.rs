use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use crate::exec::{ActualArguments, ExecError, ExecResult, SourceLocation, Value};

#[derive(Clone)]
pub struct Context
{
    frame: Rc<RefCell<Frame>>,
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
            frame: Rc::new(RefCell::new(frame)),
        }
    }

    pub fn new_root_frame(call_site: SourceLocation, formal_arguments: &Vec<String>, actual_arguments: ActualArguments) -> Context
    {
        Context
        {
            frame: Rc::new(RefCell::new(Frame::new_frame(None, call_site, formal_arguments, actual_arguments))),
        }
    }

    pub fn get_call_site(&self) -> SourceLocation
    {
        self.frame.borrow().call_site
    }

    pub fn get_var_named(&mut self, source: SourceLocation, name: &str) -> ExecResult<Value>
    {
        match self.frame.borrow().get_var_named(name)
        {
            Some(val) => Ok(val),
            None => Err(ExecError::new(source, format!("Undefined variable \"{}\"", name))),
        }
    }

    pub fn set_var_named(&mut self, name: &str, value: Value)
    {
        self.frame.borrow_mut().set_var_named(name, value);
    }

    pub fn get_param_named(&mut self, name: &str) -> ExecResult<Value>
    {
        match self.frame.borrow().get_param_named(name)
        {
            Some(val) => Ok(val),
            None => Err(ExecError::new(self.frame.borrow().call_site, format!("No named parameter \"{}\"", name))),
        }
    }

    pub fn get_param_positional(&mut self, index: usize) -> ExecResult<Value>
    {
        match self.frame.borrow().get_param_positional(index)
        {
            Some(val) => Ok(val),
            None => Err(ExecError::new(self.frame.borrow().call_site, format!("No positional parameter #{}", index + 1))),
        }
    }

    pub fn get_param_all_positional(&mut self) -> Vec<Value>
    {
        self.frame.borrow().get_param_all_positional()
    }

    pub fn sub_frame(&self, call_site: SourceLocation, formal_arguments: &Vec<String>, actual_arguments: ActualArguments) -> Context
    {
        Context
        {
            frame: Rc::new(RefCell::new(Frame::new_frame(Some(self.frame.clone()), call_site, formal_arguments, actual_arguments))),
        }
    }

    pub fn sub_block(&self) -> Context
    {
        Context
        {
            frame: Rc::new(RefCell::new(Frame::new_block(self.frame.clone()))),
        }
    }
}

struct Frame
{
    parent: Option<Rc<RefCell<Frame>>>,
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

    fn new_block(parent: Rc<RefCell<Frame>>) -> Self
    {
        let call_site = parent.borrow().call_site;

        Frame
        {
            parent: Some(parent.clone()),
            call_site,
            positional: Vec::new(),
            named: HashMap::new(),
        }
    }

    fn new_frame(parent: Option<Rc<RefCell<Frame>>>, call_site: SourceLocation, formal_arguments: &Vec<String>, actual_arguments: ActualArguments) -> Self
    {
        let mut result = Frame
        {
            parent,
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
            return parent.borrow().get_var_named(name);
        }

        None
    }

    pub fn set_var_named(&mut self, name: &str, value: Value)
    {
        self.named.insert(name.to_owned(), value);
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

    fn get_param_all_positional(&self) -> Vec<Value>
    {
        self.positional.clone()
    }
}
